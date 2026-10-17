use serde::Deserialize;
use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::content::Form;
use topcoat::router::{href, route};

use crate::forms::{back_within_this_site, done};
use crate::theme::{Theme, remember};

#[derive(Deserialize)]
struct ThemeChoice {
    theme: String,
    back: String,
}

#[route(POST)]
pub(crate) async fn change_theme(cx: &Cx, Form(choice): Form<ThemeChoice>) -> Result<()> {
    if let Ok(theme) = choice.theme.parse::<Theme>() {
        remember(cx, theme);
    }
    done(back_within_this_site(&choice.back).unwrap_or_else(|| href!(super::home).resolve(cx)))
}
