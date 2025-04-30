//! Pagination Related Types and Functions

use std::fmt::Debug;

use crate::traits::FormatSql;

/// An SQL Tool that supports pagination
pub trait PaginationSupportingTool {
    /// Set the limit for the query
    fn limit(&mut self, limit: i32) -> &mut Self;
    /// Set the offset for the query
    fn offset(&mut self, offset: i32) -> &mut Self;
    /// Set the page parameters for the query
    fn page_params(&mut self, page_params: impl PageParamsType) -> &mut Self {
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
    fn page_params(self, page_params: impl PageParamsType) -> Self
    where
        Self: Sized,
    {
        self.limit(page_params.limit()).offset(page_params.offset())
    }
}
pub trait PageParamsType {
    /// The page size
    fn page_size(&self) -> i32;
    /// The page number
    fn page_number(&self) -> i32;

    /// Calculate the number of pages based on the total number of items
    #[inline]
    fn number_of_pages(&self, total: i64) -> i32 {
        (total as f64 / self.page_size() as f64).ceil() as i32
    }
    #[inline]
    fn limit(&self) -> i32 {
        self.page_size()
    }
    /// Requests start at 1.
    /// However, offset starts at 0.
    ///
    /// This function returns the index of the page.
    #[inline]
    fn page_index(&self) -> i32 {
        (self.page_number() - 1).max(0)
    }
    /// Requests start at 1.
    #[inline]
    fn offset(&self) -> i32 {
        self.page_size() * self.page_index()
    }
}
impl<T> PageParamsType for &T
where
    T: PageParamsType,
{
    fn page_size(&self) -> i32 {
        (*self).page_size()
    }
    fn page_number(&self) -> i32 {
        (*self).page_number()
    }
    fn number_of_pages(&self, total: i64) -> i32 {
        (*self).number_of_pages(total)
    }
    fn limit(&self) -> i32 {
        (*self).limit()
    }
    fn page_index(&self) -> i32 {
        (*self).page_index()
    }
    fn offset(&self) -> i32 {
        (*self).offset()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatLimitOffset {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}
impl FormatLimitOffset {
    pub fn new_optional(limit: Option<i32>, offset: Option<i32>) -> Option<Self> {
        if limit.is_none() && offset.is_none() {
            None
        } else {
            Some(Self { limit, offset })
        }
    }
}
impl FormatSql for FormatLimitOffset {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        let mut sql = String::new();
        if let Some(limit) = self.limit.as_ref() {
            sql.push_str("LIMIT ");
            sql.push_str(&limit.to_string());
        }
        if let Some(offset) = self.offset {
            if !sql.is_empty() {
                sql.push(' ');
            }
            sql.push_str("OFFSET ");
            sql.push_str(&offset.to_string());
        }
        sql.into()
    }
}
