#![allow(dead_code)]
//! Testing Utilities for the Database Tooling
use std::fmt::Debug;

use chrono::{DateTime, FixedOffset};
use sqlformat::{FormatOptions, QueryParams};

use crate::{prelude::*, table_layout::Relation};
#[derive(Debug, Clone, TableType)]
#[table(name = "test_table")]
pub struct TestTable {
    #[column(primary_key)]
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub age: i32,
    pub email: String,
    pub phone: String,
    pub another_table_id: Option<i32>,
    pub updated_at: DateTime<FixedOffset>,
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Debug, Clone, TableType)]
#[table(name = "another_table")]
pub struct AnotherTable {
    #[column(primary_key)]
    pub id: i32,
    pub email: String,
    pub phone: String,
    pub age: i32,
    pub updated_at: DateTime<FixedOffset>,
    pub created_at: DateTime<FixedOffset>,
}
impl Relation<AnotherTable> for TestTable {
    fn from_column() -> Self::Columns {
        TestTableColumn::AnotherTableId
    }
    fn to_column() -> <AnotherTable as TableType>::Columns {
        AnotherTableColumn::Id
    }
}
#[cfg(test)]
mod tests {
    use crate::{ColumnType, testing::TestTableColumn};
    #[test]
    fn test_table_columns() {
        assert!(TestTableColumn::Id.column_name() == "id");
        assert!(TestTableColumn::Id.full_name() == "test_table.id");
    }
}

pub fn print_query(query: &str, test_name: &'static str) {
    let sql = sqlformat::format(query, &QueryParams::None, &FormatOptions::default());
    println!("Test: {}", test_name);

    println!("{}", sql);
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TestPageParams {
    /// The number of items per page
    pub page_size: i32,
    /// The page number
    pub page_number: i32,
}
impl Debug for TestPageParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestPageParams")
            .field("page_size", &self.page_size)
            .field("page_number", &self.page_number)
            .field("offset", &self.offset())
            .field("limit", &self.limit())
            .finish()
    }
}
impl TestPageParams {
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
impl Default for TestPageParams {
    fn default() -> Self {
        Self {
            page_size: 10,
            page_number: 1,
        }
    }
}
