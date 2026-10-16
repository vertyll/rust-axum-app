//! Lists shown a page at a time, as fastdo's paginator: the first, previous,
//! next and last page, the number of items per page, where the reader is and
//! how many items there are. The page and its size are signals a shard reads,
//! so changing either re-renders only that shard.

use topcoat::Result;
use topcoat::context::Cx;
use topcoat::runtime::{Event, Signal};
use topcoat::view::{View, component, view};

use super::dropdown::{DROPDOWN_ITEM, DROPDOWN_ITEM_ACTIVE, dropdown};
use super::icon::{Icon, IconSize, icon};
use crate::backend::{BackendError, Page, PageRequest};
use crate::i18n::texts;

pub const PAGE_SIZES: [usize; 5] = [5, 10, 25, 50, 100];
pub const DEFAULT_PAGE_SIZE: usize = 10;
/// The largest page the back-end gives, for a short list read whole, such as
/// the roles to choose from.
pub const MAX_PAGE_SIZE: u32 = 100;

/// The request for the page the reader asked for. Both numbers come from the
/// browser, so a size not offered falls back to the default, and the page,
/// which counts from 1 here, from 0 at the back-end.
#[must_use]
pub fn page_request(page: usize, size: usize) -> PageRequest {
    let size = if PAGE_SIZES.contains(&size) {
        size
    } else {
        DEFAULT_PAGE_SIZE
    };
    PageRequest {
        page: u32::try_from(page.max(1) - 1).unwrap_or(u32::MAX),
        size: u32::try_from(size).expect("the page sizes offered are small"),
    }
}

/// Reads the page asked for, or the last page when the list has shrunk below
/// it since the reader turned to it.
pub async fn load_page<T, F, Fut>(request: PageRequest, fetch: F) -> Result<Page<T>, BackendError>
where
    F: Fn(PageRequest) -> Fut,
    Fut: Future<Output = Result<Page<T>, BackendError>>,
{
    let page = fetch(request).await?;
    match u32::try_from(page.total_pages) {
        Ok(pages) if pages > 0 && request.page >= pages => {
            fetch(PageRequest {
                page: pages - 1,
                ..request
            })
            .await
        }
        _ => Ok(page),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageInfo {
    /// The page shown, from 1.
    pub current: usize,
    pub pages: usize,
    pub size: usize,
    pub total: usize,
}

impl<T> From<&Page<T>> for PageInfo {
    fn from(page: &Page<T>) -> Self {
        let count = |value: u64| usize::try_from(value).unwrap_or(usize::MAX);
        Self {
            current: page.number as usize + 1,
            pages: count(page.total_pages).max(1),
            size: page.size as usize,
            total: count(page.total_elements),
        }
    }
}

const STEP: &str = "inline-flex size-8 items-center justify-center rounded-md border border-border bg-background \
                    transition-colors hover:bg-foreground/5 disabled:pointer-events-none disabled:opacity-50 md:size-9";

#[component]
pub async fn pagination(
    cx: &Cx,
    page: &Signal<usize>,
    page_size: &Signal<usize>,
    shown: PageInfo,
) -> Result<impl View> {
    let texts = texts(cx).await?;
    let PageInfo {
        current,
        pages,
        size,
        total,
    } = shown;
    let number = |value: usize| i64::try_from(value).expect("a page number or count is small");
    let position = texts.t_with(
        "common.pagination.position",
        vec![("page", number(current).into()), ("pages", number(pages).into())],
    );
    let total_label = texts.t_with("common.pagination.total", vec![("count", number(total).into())]);
    let previous = current.saturating_sub(1).max(1);
    let next = (current + 1).min(pages);
    let at_start = current <= 1;
    let at_end = current >= pages;

    Ok(view! {
        <nav
            class="mt-6 flex flex-col items-start gap-3 md:flex-row md:flex-wrap md:items-center"
            aria-label=(texts.t("common.pagination.label"))
        >
            <div class="flex items-center gap-1.5 md:gap-2">
                <button
                    type="button"
                    class=(STEP)
                    disabled=(at_start)
                    title=(texts.t("common.pagination.first"))
                    aria-label=(texts.t("common.pagination.first"))
                    @click=$(|_e: Event| page.set(1))
                >
                    icon(name: Icon::ChevronDoubleLeft, size: IconSize::Sm)
                </button>
                <button
                    type="button"
                    class=(STEP)
                    disabled=(at_start)
                    title=(texts.t("common.pagination.previous"))
                    aria-label=(texts.t("common.pagination.previous"))
                    @click=$(|_e: Event| page.set(previous))
                >
                    icon(name: Icon::ChevronLeft, size: IconSize::Sm)
                </button>
                <button
                    type="button"
                    class=(STEP)
                    disabled=(at_end)
                    title=(texts.t("common.pagination.next"))
                    aria-label=(texts.t("common.pagination.next"))
                    @click=$(|_e: Event| page.set(next))
                >
                    icon(name: Icon::ChevronRight, size: IconSize::Sm)
                </button>
                <button
                    type="button"
                    class=(STEP)
                    disabled=(at_end)
                    title=(texts.t("common.pagination.last"))
                    aria-label=(texts.t("common.pagination.last"))
                    @click=$(|_e: Event| page.set(pages))
                >
                    icon(name: Icon::ChevronDoubleRight, size: IconSize::Sm)
                </button>
            </div>
            <div
                class="flex flex-wrap items-center gap-x-4 gap-y-2 text-sm text-muted-foreground"
            >
                <div class="flex items-center gap-2">
                    <span>(texts.t("common.pagination.per_page"))</span>
                    dropdown(
                        label: texts.t("common.pagination.per_page"),
                        trigger: view! { <span class="text-foreground">(size)</span> }.into(

                        ),
                        align: super::dropdown::DropdownAlign::Start,
                        for option in PAGE_SIZES {
                            <button
                                type="button"
                                class=(if option == size {
                                    DROPDOWN_ITEM_ACTIVE
                                } else {
                                    DROPDOWN_ITEM
                                })
                                @click=$(|_e: Event| {
                                    page_size.set(option);
                                    page.set(1);
                                })
                            >
                                (option)
                            </button>
                        }
                    )
                </div>
                <span class="whitespace-nowrap">(position)</span>
                <span class="whitespace-nowrap">(total_label)</span>
            </div>
        </nav>
    })
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_PAGE_SIZE, page_request};
    use crate::backend::PageRequest;

    #[test]
    fn the_first_page_here_is_page_zero_at_the_back_end() {
        assert_eq!(page_request(3, 25), PageRequest { page: 2, size: 25 });
        assert_eq!(page_request(0, 25).page, 0);
    }

    #[test]
    fn a_size_not_offered_falls_back_to_the_default() {
        assert_eq!(page_request(1, 7).size as usize, DEFAULT_PAGE_SIZE);
    }
}
