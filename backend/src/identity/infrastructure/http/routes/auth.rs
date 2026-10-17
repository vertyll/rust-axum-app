//! Sign-in through Keycloak's hosted pages: `/authorize` starts the
//! authorization code flow with PKCE, `/callback` finishes it and opens the
//! session, `/session` answers who is signed in, `/logout` ends it.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use tower_sessions::Session;

use super::SignInSettings;
use crate::identity::application::AuthSession;
use crate::identity::application::ports::IdentityPorts;
use crate::identity::application::service::IdentityService;
use crate::identity::infrastructure::http::error::ApiError;
use crate::identity::infrastructure::http::extract::{SESSION_KEY, sent_from_this_origin, store};
use crate::identity::infrastructure::http::responses::SessionResponse;
use crate::identity::infrastructure::keycloak::pkce::{self, SignInTransaction};

const TRANSACTION_KEY: &str = "identity.sign_in";
const SCOPE: &str = "openid profile email";
const SIGN_IN_FAILED: &str = "sign_in_failed";
const STATE_MISMATCH: &str = "state_mismatch";
const ALLOWED_ACTIONS: [&str; 2] = ["CONFIGURE_TOTP", "UPDATE_PASSWORD"];
/// Keycloak's action removing one credential, named by its id: `delete_credential:<id>`.
const DELETE_CREDENTIAL: &str = "delete_credential:";
const UI_LOCALES: [&str; 2] = ["pl", "en"];
const UI_THEMES: [&str; 3] = ["system", "light", "dark"];

struct AuthState<P: IdentityPorts> {
    identity: Arc<IdentityService<P>>,
    settings: Arc<SignInSettings>,
}

impl<P: IdentityPorts> Clone for AuthState<P> {
    fn clone(&self) -> Self {
        Self {
            identity: self.identity.clone(),
            settings: self.settings.clone(),
        }
    }
}

pub fn auth_router<P: IdentityPorts>(identity: Arc<IdentityService<P>>, settings: SignInSettings) -> Router {
    Router::new()
        .route("/authorize", get(authorize::<P>))
        .route("/callback", get(callback::<P>))
        .route("/session", get(current_session))
        .route("/logout", post(logout::<P>))
        .with_state(AuthState {
            identity,
            settings: Arc::new(settings),
        })
}

#[derive(Deserialize)]
struct AuthorizeQuery {
    kc_action: Option<String>,
    #[serde(default)]
    register: bool,
    /// The color theme the reader chose in the client, which Keycloak's pages open in.
    theme: Option<String>,
}

async fn authorize<P: IdentityPorts>(
    State(state): State<AuthState<P>>,
    session: Session,
    headers: HeaderMap,
    Query(query): Query<AuthorizeQuery>,
) -> Result<Redirect, ApiError> {
    let transaction = SignInTransaction::new();
    session.insert(TRANSACTION_KEY, &transaction).await.map_err(store)?;

    let keycloak = &state.settings.keycloak;
    let challenge = pkce::challenge_of(&transaction.code_verifier);
    let mut params: Vec<(&str, &str)> = vec![
        ("client_id", &keycloak.client_id),
        ("redirect_uri", &keycloak.callback_url),
        ("response_type", "code"),
        ("scope", SCOPE),
        ("state", &transaction.state),
        ("code_challenge", &challenge),
        ("code_challenge_method", pkce::CHALLENGE_METHOD),
    ];
    let language = preferred_language(&headers);
    if let Some(language) = language {
        params.push(("ui_locales", language));
    }
    if let Some(action) = query.kc_action.as_deref().filter(|action| is_allowed_action(action)) {
        if let (Some(language), Some(signed_in)) =
            (language, session.get::<AuthSession>(SESSION_KEY).await.map_err(store)?)
        {
            state.identity.keep_account_language(&signed_in, language).await;
        }
        params.push(("kc_action", action));
    }
    if query.register {
        params.push(("prompt", "create"));
    }
    if let Some(theme) = supported_theme(query.theme.as_deref()) {
        params.push(("ui_theme", theme));
    }
    Ok(Redirect::to(&with_query(&keycloak.endpoint("auth"), &params)))
}

#[derive(Deserialize)]
struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

async fn callback<P: IdentityPorts>(
    State(state): State<AuthState<P>>,
    session: Session,
    Query(query): Query<CallbackQuery>,
) -> Result<Redirect, ApiError> {
    let transaction = session
        .remove::<SignInTransaction>(TRANSACTION_KEY)
        .await
        .map_err(store)?;
    let post_login = &state.settings.post_login_url;

    if let Some(error) = query.error {
        tracing::debug!("Keycloak returned an authorization error: {error}");
        return Ok(Redirect::to(&with_query(post_login, &[("error", SIGN_IN_FAILED)])));
    }
    let (Some(code), Some(returned_state), Some(transaction)) = (query.code, query.state, transaction) else {
        tracing::warn!("rejecting a sign-in callback whose state was not issued to this browser");
        return Ok(Redirect::to(&with_query(post_login, &[("error", STATE_MISMATCH)])));
    };
    if !pkce::same_state(&transaction.state, &returned_state) {
        tracing::warn!("rejecting a sign-in callback whose state was not issued to this browser");
        return Ok(Redirect::to(&with_query(post_login, &[("error", STATE_MISMATCH)])));
    }

    match state.identity.sign_in(&code, &transaction.code_verifier).await {
        Ok(signed_in) => {
            session.cycle_id().await.map_err(store)?;
            session.insert(SESSION_KEY, &signed_in).await.map_err(store)?;
            Ok(Redirect::to(post_login))
        }
        Err(err) => {
            tracing::warn!("sign-in could not be completed: {err}");
            Ok(Redirect::to(&with_query(post_login, &[("error", SIGN_IN_FAILED)])))
        }
    }
}

async fn current_session(session: Session) -> Result<Response, ApiError> {
    Ok(match session.get::<AuthSession>(SESSION_KEY).await.map_err(store)? {
        Some(signed_in) => Json(SessionResponse::from(&signed_in)).into_response(),
        None => StatusCode::NO_CONTENT.into_response(),
    })
}

async fn logout<P: IdentityPorts>(
    State(state): State<AuthState<P>>,
    method: Method,
    headers: HeaderMap,
    session: Session,
) -> Result<StatusCode, ApiError> {
    if !sent_from_this_origin(&method, &headers) {
        return Ok(StatusCode::FORBIDDEN);
    }
    if let Some(signed_in) = session.get::<AuthSession>(SESSION_KEY).await.map_err(store)? {
        state.identity.sign_out(&signed_in).await;
    }
    session.flush().await.map_err(store)?;
    Ok(StatusCode::NO_CONTENT)
}

/// An action Keycloak may run for a signed-in person: setting up the second factor,
/// changing the password, or removing a credential named by its UUID, which Keycloak
/// removes only when it is the person's own.
fn is_allowed_action(action: &str) -> bool {
    ALLOWED_ACTIONS.contains(&action)
        || action
            .strip_prefix(DELETE_CREDENTIAL)
            .is_some_and(|id| uuid::Uuid::try_parse(id).is_ok())
}

fn preferred_language(headers: &HeaderMap) -> Option<&'static str> {
    let accepted = headers.get(header::ACCEPT_LANGUAGE)?.to_str().ok()?;
    accepted
        .split(',')
        .filter_map(|part| part.split(';').next())
        .map(|tag| tag.trim().split('-').next().unwrap_or_default().to_ascii_lowercase())
        .find_map(|language| UI_LOCALES.into_iter().find(|supported| *supported == language))
}

fn supported_theme(theme: Option<&str>) -> Option<&'static str> {
    UI_THEMES.into_iter().find(|supported| Some(*supported) == theme)
}

fn with_query(base: &str, params: &[(&str, &str)]) -> String {
    let query = params
        .iter()
        .map(|(name, value)| format!("{}={}", encode(name), encode(value)))
        .collect::<Vec<_>>()
        .join("&");
    let separator = if base.contains('?') { '&' } else { '?' };
    format!("{base}{separator}{query}")
}

fn encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (byte as char).to_string(),
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue, header};

    use super::{is_allowed_action, preferred_language, supported_theme, with_query};

    #[test]
    fn passes_on_only_the_actions_of_the_account_page() {
        assert!(is_allowed_action("CONFIGURE_TOTP"));
        assert!(is_allowed_action("UPDATE_PASSWORD"));
        assert!(is_allowed_action(
            "delete_credential:90062308-97cf-410a-afe5-06680a07a349"
        ));
        assert!(!is_allowed_action("delete_credential"));
        assert!(!is_allowed_action("delete_credential:otp&prompt=none"));
        assert!(!is_allowed_action("delete_account"));
    }

    #[test]
    fn picks_the_first_supported_language() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::ACCEPT_LANGUAGE,
            HeaderValue::from_static("de-DE,pl;q=0.9,en;q=0.8"),
        );
        assert_eq!(preferred_language(&headers), Some("pl"));
        assert_eq!(preferred_language(&HeaderMap::new()), None);
    }

    #[test]
    fn passes_on_only_a_theme_keycloak_knows() {
        assert_eq!(supported_theme(Some("dark")), Some("dark"));
        assert_eq!(supported_theme(Some("sepia")), None);
        assert_eq!(supported_theme(None), None);
    }

    #[test]
    fn encodes_query_values() {
        assert_eq!(
            with_query(
                "http://k/auth",
                &[("scope", "openid profile"), ("redirect_uri", "http://a/b?c=d")]
            ),
            "http://k/auth?scope=openid%20profile&redirect_uri=http%3A%2F%2Fa%2Fb%3Fc%3Dd"
        );
    }
}
