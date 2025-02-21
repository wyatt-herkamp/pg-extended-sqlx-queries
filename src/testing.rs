#![allow(dead_code)]
//! Testing Utilities for the Database Tooling
use chrono::{DateTime, FixedOffset};
use sqlformat::{FormatOptions, QueryParams};

use crate::prelude::*;
#[derive(Debug, Clone, TableType)]
#[table(name = "test_table")]
pub struct TestTable {
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
    pub id: i32,
    pub email: String,
    pub phone: String,
    pub age: i32,
    pub updated_at: DateTime<FixedOffset>,
    pub created_at: DateTime<FixedOffset>,
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
