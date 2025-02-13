#![allow(dead_code)]
//! Testing Utilities for the Database Tooling
use chrono::{DateTime, FixedOffset};
use sqlx::Postgres;

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

pub struct TestParentQuery<'args> {
    pub(crate) arguments: Option<<Postgres as sqlx::Database>::Arguments<'args>>,
    pub(crate) conditions: Vec<SQLCondition>,
}
impl Default for TestParentQuery<'_> {
    fn default() -> Self {
        Self {
            arguments: Some(Default::default()),
            conditions: Vec::new(),
        }
    }
}
impl<'args> HasArguments<'args> for TestParentQuery<'args> {
    fn take_arguments_or_error(&mut self) -> <Postgres as sqlx::Database>::Arguments<'args> {
        self.arguments.take().expect("Arguments already taken")
    }

    fn borrow_arguments_or_error(&mut self) -> &mut <Postgres as sqlx::Database>::Arguments<'args> {
        self.arguments.as_mut().expect("Arguments already taken")
    }
}
impl<'args> WhereableTool<'args> for TestParentQuery<'args> {
    fn push_where_comparison(&mut self, comparison: SQLCondition) {
        self.conditions.push(comparison);
    }
}
impl<'args> FormatWhere for TestParentQuery<'args> {
    fn get_conditions(&self) -> &[SQLCondition] {
        &self.conditions
    }
}
#[cfg(test)]
mod tests {
    use crate::{testing::TestTableColumn, ColumnType};
    #[test]
    fn test_table_columns() {
        assert!(TestTableColumn::Id.column_name() == "id");
        assert!(TestTableColumn::Id.formatted_column() == "test_table.id");
    }
}
