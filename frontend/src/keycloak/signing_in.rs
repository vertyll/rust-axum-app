//! Keycloak's sign-in, reached through the back-end, which the browser starts
//! here so Keycloak shows its pages in the theme and the language the reader
//! chose here, and the account actions that go through it.

use reqwest::header::ACCEPT_LANGUAGE;
use serde::Serialize;
use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::header::HeaderValue;
use topcoat::router::request::uri;
use topcoat::router::response::Response;
use topcoat::router::{Body, href, route};

use super::account::{AccountAction, keep_account_language};
use crate::backend::proxy;
use crate::i18n::lang;
use crate::theme::{Theme, theme};

/// The sign-in, passed on to the back-end with the reader's language and
/// theme. It is matched before the proxy's catch-all `/api` route. Keycloak
/// shows a signed-in person their account's language, so before an account
/// action the reader's is stored on the account.
#[route(GET "/api/auth/authorize")]
pub async fn authorize(cx: &Cx, body: Body) -> Result<Response> {
    let language = lang(cx);
    let chosen_theme = theme(cx);
    // A language that cannot be stored leaves the account's as it is, and the action still opens.
    if asks_for_an_account_action(uri(cx).query().unwrap_or_default())
        && let Err(err) = keep_account_language(cx, language.code()).await
    {
        tracing::warn!("could not keep the account's language before an account action: {err:?}");
    }
    proxy::forward_changed(cx, body, |path_and_query, headers| {
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static(language.code()));
        *path_and_query = with_theme(path_and_query, chosen_theme);
    })
    .await
}

pub fn sign_in(cx: &Cx) -> String {
    href!(authorize).resolve(cx)
}

pub fn register(cx: &Cx) -> String {
    #[derive(Serialize)]
    struct Register {
        register: bool,
    }
    href!(authorize).query(Register { register: true }).resolve(cx)
}

/// Where the browser goes for an account action: the sign-in, asking Keycloak
/// for that one page.
pub fn account_action(cx: &Cx, action: &AccountAction) -> String {
    #[derive(Serialize)]
    struct Action {
        kc_action: String,
    }
    let kc_action = match action {
        AccountAction::UpdatePassword => "UPDATE_PASSWORD".to_owned(),
        AccountAction::ConfigureTotp => "CONFIGURE_TOTP".to_owned(),
        AccountAction::DeleteCredential(id) => format!("delete_credential:{id}"),
    };
    href!(authorize).query(Action { kc_action }).resolve(cx)
}

fn asks_for_an_account_action(query: &str) -> bool {
    form_urlencoded::parse(query.as_bytes()).any(|(name, _)| name == "kc_action")
}

/// The path and query with `theme` set to the reader's, in place of any the request carried.
fn with_theme(path_and_query: &str, theme: Theme) -> String {
    let (path, query) = path_and_query.split_once('?').unwrap_or((path_and_query, ""));
    let mut pairs = form_urlencoded::Serializer::new(String::new());
    for (name, value) in form_urlencoded::parse(query.as_bytes()).filter(|(name, _)| name != "theme") {
        pairs.append_pair(&name, &value);
    }
    pairs.append_pair("theme", theme.code());
    format!("{path}?{}", pairs.finish())
}

#[cfg(test)]
mod tests {
    use super::{asks_for_an_account_action, with_theme};
    use crate::theme::Theme;

    #[test]
    fn an_account_action_is_told_by_its_parameter() {
        assert!(asks_for_an_account_action("kc_action=UPDATE_PASSWORD&theme=dark"));
        assert!(!asks_for_an_account_action("register=true"));
        assert!(!asks_for_an_account_action(""));
    }

    #[test]
    fn the_theme_of_the_reader_replaces_any_the_request_carried() {
        assert_eq!(
            with_theme("/api/auth/authorize", Theme::Dark),
            "/api/auth/authorize?theme=dark"
        );
        assert_eq!(
            with_theme("/api/auth/authorize?register=true&theme=light", Theme::System),
            "/api/auth/authorize?register=true&theme=system"
        );
    }
}
