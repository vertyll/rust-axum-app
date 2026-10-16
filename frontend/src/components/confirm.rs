use topcoat::Result;
use topcoat::context::Cx;
use topcoat::runtime::{Event, signal};
use topcoat::view::{View, attributes, component, view};

use crate::components::button::{ButtonVariant, button};
use crate::components::icon::{Icon, IconSize, icon};
use crate::i18n::texts;

/// A destructive action behind a second click: the first shows the
/// confirmation, which posts to `action`. The open state lives only in the
/// browser.
#[component]
pub async fn confirm_action(
    cx: &Cx,
    #[into] action: String,
    #[into] label: String,
    #[into] question: String,
) -> Result<impl View> {
    let texts = texts(cx).await?;
    let asking = signal(cx, || false);

    Ok(view! {
        <div class="flex flex-wrap items-center gap-2">
            <span :hidden=$(asking.get())>
                button(
                    variant: ButtonVariant::Destructive,
                    attrs: attributes! { type="button" @click=$(|_e: Event| asking.set(true)) },
                    icon(name: Icon::Trash, size: IconSize::Sm)
                    (label.as_str())
                )
            </span>
            <form
                method="post"
                action=(action)
                class="flex flex-wrap items-center gap-2"
                :hidden=$(!asking.get())
            >
                <span class="text-sm">(question)</span>
                button(
                    variant: ButtonVariant::Destructive,
                    attrs: attributes! { type="submit" },
                    icon(name: Icon::Trash, size: IconSize::Sm)
                    (texts.t("common.confirm"))
                )
                button(
                    variant: ButtonVariant::Outline,
                    attrs: attributes! { type="button" @click=$(|_e: Event| asking.set(false)) },
                    icon(name: Icon::XMark, size: IconSize::Sm)
                    (texts.t("common.cancel"))
                )
            </form>
        </div>
    })
}
