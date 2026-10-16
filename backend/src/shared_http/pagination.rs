//! How a paged collection travels over HTTP: `?page=0&size=20` in, and the
//! page with its totals out.

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::shared_kernel::pagination::{MAX_PAGE_SIZE, PageRequest, PageResult};

const DEFAULT_PAGE_SIZE: u32 = 20;

#[derive(Debug, Deserialize, Validate)]
pub struct PageQuery {
    #[serde(default)]
    pub page: u32,
    #[serde(default = "default_size")]
    #[validate(range(min = 1, max = MAX_PAGE_SIZE, message = "pagination.validators.size.range"))]
    pub size: u32,
}

fn default_size() -> u32 {
    DEFAULT_PAGE_SIZE
}

impl PageQuery {
    /// The query of an endpoint that also takes other parameters, which
    /// cannot be flattened into it from a query string.
    #[must_use]
    pub fn new(page: Option<u32>, size: Option<u32>) -> Self {
        Self {
            page: page.unwrap_or_default(),
            size: size.unwrap_or(DEFAULT_PAGE_SIZE),
        }
    }

    /// The request, once [`Validate::validate`] has accepted the query.
    ///
    /// # Panics
    ///
    /// If the query was not validated first and its size is out of range.
    #[must_use]
    pub fn request(&self) -> PageRequest {
        PageRequest::new(self.page, self.size).expect("the query was validated")
    }
}

#[derive(Debug, Serialize)]
pub struct PageResponse<T> {
    pub content: Vec<T>,
    pub page: u32,
    pub size: u32,
    pub total_elements: u64,
    pub total_pages: u64,
}

impl<D, T: From<D>> From<PageResult<D>> for PageResponse<T> {
    fn from(page: PageResult<D>) -> Self {
        let total_pages = page.total_pages();
        Self {
            page: page.page,
            size: page.size,
            total_elements: page.total,
            total_pages,
            content: page.items.into_iter().map(T::from).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use validator::Validate;

    use super::PageQuery;

    #[test]
    fn a_missing_page_and_size_mean_the_first_twenty() {
        let query: PageQuery = serde_json::from_str("{}").unwrap();

        assert!(query.validate().is_ok());
        assert_eq!((query.page, query.size), (0, 20));
    }

    #[test]
    fn a_size_over_a_hundred_is_refused() {
        let query = PageQuery { page: 0, size: 500 };

        assert!(query.validate().is_err());
    }
}
