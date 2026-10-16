use topcoat::Result;
use topcoat::runtime::link;
use topcoat::view::{View, attributes, component, view};

use super::button::{ButtonSize, ButtonVariant, button_variants};
use super::icon::{Icon, IconSize, icon};

/// The way back to the list a page was opened from.
#[component]
pub async fn back_link(#[into] href: String, #[into] label: String) -> Result<impl View> {
    Ok(view! {
        link(
            href: href,
            attrs: attributes! {
                class=(button_variants(ButtonVariant::Outline, ButtonSize::Md))
            },
            icon(name: Icon::ChevronLeft, size: IconSize::Sm)
            (label)
        )
    })
}
