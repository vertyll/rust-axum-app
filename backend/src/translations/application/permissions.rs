//! What the translations module checks. Reading a language's catalogue
//! needs no account.

use crate::identity::contracts::{Catalogue, Declared, Permission};

pub struct TranslationsView;

impl Permission for TranslationsView {
    const NAME: &'static str = "TRANSLATIONS_VIEW";
    const DESCRIPTION: &'static str = "See every message with its default";
}

pub struct TranslationsEdit;

impl Permission for TranslationsEdit {
    const NAME: &'static str = "TRANSLATIONS_EDIT";
    const DESCRIPTION: &'static str = "Change messages and reset them to their defaults";
}

pub const CATALOGUE: Catalogue = Catalogue {
    module: "translations",
    permissions: &[Declared::of::<TranslationsView>(), Declared::of::<TranslationsEdit>()],
    stock_roles: &[],
};
