use serde::Deserialize;
use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::content::Form;
use topcoat::router::{href, route};

use crate::forms::{back_within_this_site, done};
use crate::i18n::{Lang, remember};

#[derive(Deserialize)]
struct LanguageChoice {
    lang: String,
    back: String,
}

#[route(POST)]
pub(crate) async fn change_language(cx: &Cx, Form(choice): Form<LanguageChoice>) -> Result<()> {
    if let Ok(lang) = choice.lang.parse::<Lang>() {
        remember(cx, lang);
    }
    done(back_within_this_site(&choice.back).unwrap_or_else(|| href!(super::home).resolve(cx)))
}
