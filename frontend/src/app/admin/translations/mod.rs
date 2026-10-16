pub(crate) mod api;
pub(crate) mod translation_key;

use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::{href, page};
use topcoat::runtime::{Event, Signal, shard, signal};
use topcoat::view::{View, attributes, component, view};

use self::api::{self as catalog, Translation};
use self::translation_key::TranslationKey;
use crate::auth::{Permission, require_permission};
use crate::backend::BackendError;
use crate::components::badge::{BadgeVariant, badge};
use crate::components::icon::{Icon, IconSize, icon};
use crate::components::input::input;
use crate::components::page_header::page_header;
use crate::components::pagination::{DEFAULT_PAGE_SIZE, PageInfo, load_page, page_request, pagination};
use crate::components::row_actions::{icon_link, icon_submit};
use crate::i18n::{Lang, texts};

#[page]
pub(crate) async fn translations(cx: &Cx) -> Result<impl View> {
    require_permission(cx, Permission::TranslationsView).await?;
    let texts = texts(cx).await?;
    let query = signal(cx, String::new);
    let page = signal(cx, || 1_usize);
    let page_size = signal(cx, || DEFAULT_PAGE_SIZE);

    Ok(view! {
        page_header(
            title: texts.t("admin.translations.title"),
            description: Some(texts.t("admin.translations.description"))
        )
        <div class="mb-6 rounded-lg border border-border bg-card px-4 py-3">
            <label for="filter" class="mb-2 block text-sm font-medium">
                (texts.t("admin.translations.filter"))
            </label>
            <div class="relative max-w-md">
                <span
                    class="pointer-events-none absolute top-1/2 left-3 -translate-y-1/2 text-muted-foreground"
                >
                    icon(name: Icon::MagnifyingGlass, size: IconSize::Sm)
                </span>
                input(
                    attrs: attributes! {
                        id="filter"
                        type="search"
                        autocomplete="off"
                        class="pl-9"
                        @input=$(|e: Event| {
                            query.set(e.target.value);
                            page.set(1);
                        })
                    }
                )
            </div>
        </div>
        message_list(query: $(query.get()), page: page, page_size: page_size)
    })
}

/// The messages that match `query`, one page at a time. It is an endpoint of
/// its own, so it checks the reader's permission and takes its arguments as
/// they come: any text, and a page number kept within the pages there are.
#[shard]
pub(crate) async fn message_list(
    cx: &Cx,
    query: String,
    page: Signal<usize>,
    page_size: Signal<usize>,
) -> Result<impl View> {
    let current = require_permission(cx, Permission::TranslationsView).await?;
    let texts = texts(cx).await?;
    let may_edit = current.may(Permission::TranslationsEdit);
    let request = page_request(page.get(), page_size.get());
    let paged = load_page(request, |request| catalog::list(cx, &query, request))
        .await
        .map_err(BackendError::into_page_error)?;

    Ok(view! {
        if paged.total_elements == 0 {
            <p
                class="rounded-lg border border-dashed border-border px-4 py-12 text-center text-sm text-muted-foreground"
            >
                (texts.t("admin.translations.empty"))
            </p>
        }
        <ul class="flex flex-col gap-3">
            #[key(message.key.clone())]
            for message in paged.content.clone() {
                message_card(message: message, may_edit: may_edit)
            }
        </ul>
        pagination(page: &page, page_size: &page_size, shown: PageInfo::from(&paged))
    })
}

#[component]
async fn message_card(cx: &Cx, message: Translation, may_edit: bool) -> Result<impl View> {
    let texts = texts(cx).await?;
    let edit = href!(translation_key::translation, TranslationKey(message.key.clone())).resolve(cx);
    let reset = href!(
        translation_key::reset::reset_translation,
        TranslationKey(message.key.clone())
    )
    .resolve(cx);
    Ok(view! {
        <li
            id=(format!("message-{}", message.key))
            class="rounded-lg border border-border bg-card p-4"
        >
            <div class="mb-3 flex items-center justify-between gap-2">
                <span class="min-w-0 font-mono text-sm break-all">
                    (message.key.as_str())
                </span>
                <div class="flex shrink-0 items-center gap-1">
                    if message.customized {
                        badge(
                            variant: BadgeVariant::Secondary,
                            (texts.t("admin.translations.customized"))
                        )
                        if may_edit {
                            icon_submit(
                                action: reset,
                                name: Icon::ArrowPath,
                                label: texts.t("admin.translations.edit.reset")
                            )
                        }
                    }
                    if may_edit {
                        icon_link(
                            href: edit,
                            name: Icon::Pencil,
                            label: texts.t("common.edit")
                        )
                    } else {
                        icon_link(
                            href: edit,
                            name: Icon::Eye,
                            label: texts.t("common.details")
                        )
                    }
                </div>
            </div>
            <dl class="grid grid-cols-[2.5rem_1fr] gap-x-3 gap-y-2 text-sm">
                <dt class="text-xs font-semibold text-muted-foreground uppercase">
                    (Lang::Pl.code())
                </dt>
                <dd lang="pl" class="break-words">(message.message.pl.as_str())</dd>
                <dt class="text-xs font-semibold text-muted-foreground uppercase">
                    (Lang::En.code())
                </dt>
                <dd lang="en" class="break-words">(message.message.en.as_str())</dd>
            </dl>
        </li>
    })
}
