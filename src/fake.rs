//! Utilities for testing the SQL Expressions
//!
//! This is used to test the SQL Expressions
//!
//! This is NOT STABLE API and any usage will break at any time
use sqlx::Postgres;

use crate::{FormatWhere, HasArguments, SQLCondition, WhereableTool};
/// A utility struct for testing different SQL Queries
pub struct FakeQuery<'args> {
    pub(crate) arguments: Option<<Postgres as sqlx::Database>::Arguments<'args>>,
    pub(crate) conditions: Vec<SQLCondition>,
}
impl Default for FakeQuery<'_> {
    fn default() -> Self {
        Self {
            arguments: Some(Default::default()),
            conditions: Vec::new(),
        }
    }
}
impl<'args> HasArguments<'args> for FakeQuery<'args> {
    fn take_arguments_or_error(&mut self) -> <Postgres as sqlx::Database>::Arguments<'args> {
        self.arguments.take().expect("Arguments already taken")
    }

    fn borrow_arguments_or_error(&mut self) -> &mut <Postgres as sqlx::Database>::Arguments<'args> {
        self.arguments.as_mut().expect("Arguments already taken")
    }
}
impl<'args> WhereableTool<'args> for FakeQuery<'args> {
    fn push_where_comparison(&mut self, comparison: SQLCondition) {
        self.conditions.push(comparison);
    }
}
impl<'args> FormatWhere for FakeQuery<'args> {
    fn get_conditions(&self) -> &[SQLCondition] {
        &self.conditions
    }
}
