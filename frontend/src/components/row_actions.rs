//! The actions column of a table, as fastdo's: it stays at the right edge
//! while the other columns scroll beneath it, and each action is an icon with
//! its name for screen readers and as a tooltip.

use super::icon::{Icon, IconSize, icon};
use topcoat::Result;
use topcoat::view::{Child, View, component, view};

const STICKY: &str = "sticky right-0 bg-background shadow-[-8px_0_8px_-8px_rgb(0_0_0/0.15)]";
const ICON_ACTION: &str = "inline-flex size-8 items-center justify-center rounded-md text-muted-foreground transition-colors \
                           hover:bg-foreground/5 hover:text-foreground";

#[component]
pub async fn actions_head(#[into] label: String) -> Result<impl View> {
    Ok(view! {
        <th
            class=(format!(
                "{STICKY} h-10 px-3 text-right align-middle font-medium whitespace-nowrap text-muted-foreground",
            ))
        >
            (label)
        </th>
    })
}

#[component]
pub async fn actions_cell(#[default] child: Child<'_>) -> Result<impl View> {
    Ok(view! {
        <td class=(format!("{STICKY} p-2 align-middle"))>
            <div class="flex items-center justify-end gap-1">(child)</div>
        </td>
    })
}

#[component]
pub async fn icon_link(#[into] href: String, name: Icon, #[into] label: String) -> Result<impl View> {
    Ok(view! {
        topcoat::runtime::link(
            href: href,
            attrs: topcoat::view::attributes! {
                class=(ICON_ACTION)
                title=(label.clone())
                aria-label=(label)
            },
            icon(name: name, size: IconSize::Md)
        )
    })
}

#[component]
pub async fn icon_download(#[into] href: String, name: Icon, #[into] label: String) -> Result<impl View> {
    Ok(view! {
        <a
            href=(href)
            download=""
            class=(ICON_ACTION)
            title=(label.clone())
            aria-label=(label)
        >
            icon(name: name, size: IconSize::Md)
        </a>
    })
}

/// An action that posts a form, such as restoring a default; the children
/// are the form's hidden fields, if it has any.
#[component]
pub async fn icon_submit(
    #[into] action: String,
    name: Icon,
    #[into] label: String,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <form method="post" action=(action)>
            (child)
            <button
                type="submit"
                class=(ICON_ACTION)
                title=(label.clone())
                aria-label=(label)
            >
                icon(name: name, size: IconSize::Md)
            </button>
        </form>
    })
}
