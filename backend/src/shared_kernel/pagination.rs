//! A collection read a page at a time.

pub const MAX_PAGE_SIZE: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("a page holds 1 to {MAX_PAGE_SIZE} items")]
pub struct InvalidPageSize;

/// Which page to read: `page` counts from 0, `size` is 1 to [`MAX_PAGE_SIZE`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageRequest {
    page: u32,
    size: u32,
}

impl PageRequest {
    pub fn new(page: u32, size: u32) -> Result<Self, InvalidPageSize> {
        if (1..=MAX_PAGE_SIZE).contains(&size) {
            Ok(Self { page, size })
        } else {
            Err(InvalidPageSize)
        }
    }

    #[must_use]
    pub fn page(self) -> u32 {
        self.page
    }

    #[must_use]
    pub fn size(self) -> u32 {
        self.size
    }

    #[must_use]
    pub fn offset(self) -> usize {
        self.page as usize * self.size as usize
    }

    #[must_use]
    pub fn limit(self) -> usize {
        self.size as usize
    }

    /// The page of a collection already in memory, for one bounded by the
    /// code rather than by its data, such as the message catalog.
    #[must_use]
    pub fn slice<T>(self, items: Vec<T>) -> PageResult<T> {
        let total = items.len() as u64;
        let items = items.into_iter().skip(self.offset()).take(self.limit()).collect();
        PageResult::new(items, self, total)
    }
}

/// One page of a collection, and how many items the whole holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageResult<T> {
    pub items: Vec<T>,
    pub page: u32,
    pub size: u32,
    pub total: u64,
}

impl<T> PageResult<T> {
    #[must_use]
    pub fn new(items: Vec<T>, request: PageRequest, total: u64) -> Self {
        Self {
            items,
            page: request.page,
            size: request.size,
            total,
        }
    }

    #[must_use]
    pub fn total_pages(&self) -> u64 {
        self.total.div_ceil(u64::from(self.size))
    }

    #[must_use]
    pub fn map<U>(self, f: impl FnMut(T) -> U) -> PageResult<U> {
        PageResult {
            items: self.items.into_iter().map(f).collect(),
            page: self.page,
            size: self.size,
            total: self.total,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{InvalidPageSize, PageRequest};

    #[test]
    fn a_page_holds_one_to_a_hundred_items() {
        assert_eq!(PageRequest::new(0, 0), Err(InvalidPageSize));
        assert_eq!(PageRequest::new(0, 101), Err(InvalidPageSize));
        assert!(PageRequest::new(3, 100).is_ok());
    }

    #[test]
    fn slices_the_requested_page_and_counts_the_whole() {
        let page = PageRequest::new(2, 10).unwrap().slice((1..=23).collect::<Vec<_>>());

        assert_eq!(page.items, [21, 22, 23]);
        assert_eq!((page.total, page.total_pages()), (23, 3));
    }

    #[test]
    fn a_page_past_the_end_is_empty_but_still_counts_the_whole() {
        let page = PageRequest::new(9, 10).unwrap().slice((1..=23).collect::<Vec<_>>());

        assert_eq!(page.items, Vec::<i32>::new());
        assert_eq!(page.total, 23);
    }
}
