use serde::Deserialize;
use topcoat::Result;
use topcoat::context::{Cx, app_context};
use topcoat::cookie::{Cookie, Cookies, SameSite, cookies};
use topcoat::router::content::Form;
use topcoat::router::{href, route};

use crate::config::AppConfig;
use crate::forms::done;
use crate::i18n::{LANG_COOKIE, Lang};

#[derive(Deserialize)]
struct LanguageChoice {
    lang: String,
    back: String,
}

#[route(POST)]
pub(crate) async fn change_language(cx: &Cx, Form(choice): Form<LanguageChoice>) -> Result<()> {
    if let Ok(lang) = choice.lang.parse::<Lang>() {
        let cookie = Cookie::build((LANG_COOKIE, lang.code()))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(app_context::<AppConfig>(cx).secure_cookies())
            .permanent()
            .build();
        cookies(cx).add(cookie);
    }
    done(back_within_this_site(&choice.back).unwrap_or_else(|| href!(super::home).resolve(cx)))
}

/// The page to return to, only when it is a path on this site, so the form
/// cannot send the reader elsewhere.
fn back_within_this_site(back: &str) -> Option<String> {
    let on_this_site = back.starts_with('/') && !back.starts_with("//") && !back.contains('\\');
    on_this_site.then(|| back.to_owned())
}

#[cfg(test)]
mod tests {
    use super::back_within_this_site;

    #[test]
    fn returns_only_to_paths_on_this_site() {
        assert_eq!(back_within_this_site("/files?x=1").as_deref(), Some("/files?x=1"));
        assert_eq!(back_within_this_site("//evil.example"), None);
        assert_eq!(back_within_this_site("/\\evil.example"), None);
        assert_eq!(back_within_this_site("https://evil.example"), None);
    }
}
