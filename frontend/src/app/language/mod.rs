use serde::Deserialize;
use topcoat::Result;
use topcoat::context::{Cx, app_context};
use topcoat::cookie::{Cookie, Cookies, SameSite, cookies};
use topcoat::router::content::Form;
use topcoat::router::{href, route};

use crate::config::AppConfig;
use crate::forms::{back_within_this_site, done};
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
