use topcoat::Result;
use topcoat::view::{Child, View, class, component, view};

use super::icon::{Icon, IconSize, icon};

/// Opening one dropdown closes the others, as each is a `<details>` of its own,
/// and opens its panel upwards when it does not fit below its trigger on the
/// screen but does above, as near the end of a page.
const ON_TOGGLE: &str = "if (!this.open) return; \
    document.querySelectorAll('details[data-dropdown]').forEach(menu => menu === this || (menu.open = false)); \
    const panel = this.lastElementChild; \
    panel.classList.remove('bottom-full', 'mb-2'); \
    panel.classList.add('mt-2'); \
    const trigger = this.getBoundingClientRect(); \
    const below = window.innerHeight - trigger.bottom; \
    if (panel.offsetHeight > below && trigger.top > below) { \
        panel.classList.remove('mt-2'); \
        panel.classList.add('bottom-full', 'mb-2'); \
    }";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DropdownAlign {
    Start,
    #[default]
    End,
}

/// A button that opens a panel of items, as fastdo's `app-dropdown`. It is a
/// `<details>`, so it works without scripts. `label` names the trigger for
/// screen readers; `trigger` is what it shows, and the children are the
/// panel's items, such as [`dropdown_item`]s.
#[component]
pub async fn dropdown(
    #[into] label: String,
    trigger: Child<'_>,
    #[default] align: DropdownAlign,
    #[default] class: Option<&'static str>,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <details
            class=(class!("relative", class))
            data-dropdown=""
            ontoggle=(ON_TOGGLE)
        >
            <summary
                class="flex cursor-pointer list-none items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors hover:bg-foreground/5"
                aria-label=(label)
            >
                (trigger)
                icon(name: Icon::ChevronDown, size: IconSize::Sm)
            </summary>
            <div
                class=(class!(
                    "absolute z-50 mt-2 flex min-w-44 flex-col rounded-lg border border-border bg-popover p-1 text-popover-foreground shadow-sm",
                    "right-0" if align == DropdownAlign::End,
                    "left-0" if align == DropdownAlign::Start,
                ))
            >
                (child)
            </div>
        </details>
    })
}

/// The classes of an item in a dropdown's panel, a link or a button.
pub const DROPDOWN_ITEM: &str =
    "flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-sm transition-colors hover:bg-foreground/5";
pub const DROPDOWN_ITEM_ACTIVE: &str =
    "flex w-full items-center gap-2 rounded-md bg-foreground/5 px-3 py-2 text-left text-sm font-medium";
/// An item whose action cannot be undone, such as signing out.
pub const DROPDOWN_ITEM_DESTRUCTIVE: &str = "flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-sm \
     text-destructive transition-colors hover:bg-foreground/5";
