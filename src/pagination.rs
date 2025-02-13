//! Pagination Related Types and Functions

use std::fmt::Debug;

/// An SQL Tool that supports pagination
pub trait PaginationSupportingTool {
    /// Set the limit for the query
    fn limit(&mut self, limit: i32) -> &mut Self;
    /// Set the offset for the query
    fn offset(&mut self, offset: i32) -> &mut Self;
    /// Set the page parameters for the query
    fn page_params(&mut self, page_params: impl Into<PageParams>) -> &mut Self {
        let page_params = page_params.into();
        self.limit(page_params.limit()).offset(page_params.offset())
    }
}
/// An SQL Tool that supports pagination
pub trait PaginationOwnedSupportingTool {
    /// Set the limit for the query
    fn limit(self, limit: i32) -> Self;
    /// Set the offset for the query
    fn offset(self, offset: i32) -> Self;
    /// Set the page parameters for the query
    fn page_params(self, page_params: impl Into<PageParams>) -> Self
    where
        Self: Sized,
    {
        let page_params = page_params.into();
        self.limit(page_params.limit()).offset(page_params.offset())
    }
}
/// Parameters for pagination
///
/// Includes the page size and the page number
///
/// # Note
/// Passing a page number less than 1 or equal to I32::MAX might result in all items being returned
/// This is dependent on the request handler
#[derive(Clone, Copy, PartialEq, Eq)]

pub struct PageParams {
    /// The number of items per page
    pub page_size: i32,
    /// The page number
    pub page_number: i32,
}
impl Debug for PageParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PageParams")
            .field("page_size", &self.page_size)
            .field("page_number", &self.page_number)
            .field("offset", &self.offset())
            .field("limit", &self.limit())
            .finish()
    }
}
impl PageParams {
    /// If the page size is greater than the max argument it is set to the max argument
    pub fn max_page_size(&mut self, max: i32) {
        self.page_size = self.page_size.min(max);
    }
    /// Calculate the number of pages based on the total number of items
    #[inline]
    pub fn number_of_pages(&self, total: i64) -> i32 {
        (total as f64 / self.page_size as f64).ceil() as i32
    }
    #[inline]
    pub fn limit(&self) -> i32 {
        self.page_size
    }
    /// Requests start at 1.
    /// However, offset starts at 0.
    ///
    /// This function returns the index of the page.
    #[inline]
    pub fn page_index(&self) -> i32 {
        (self.page_number - 1).max(0)
    }
    /// Requests start at 1.
    #[inline]
    pub fn offset(&self) -> i32 {
        self.page_size * self.page_index()
    }
}
impl Default for PageParams {
    fn default() -> Self {
        Self {
            page_size: 10,
            page_number: 1,
        }
    }
}
