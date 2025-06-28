//! Utilities for testing the SQL Expressions
//!
//! This is used to test the SQL Expressions
//!
//! This is NOT STABLE API and any usage will break at any time

use crate::{
    expr::{ArgumentHolder, HasArguments, SQLCondition},
    traits::{FormatWhere, WhereableTool},
};

/// A utility struct for testing different SQL Queries
#[derive(Default)]
pub struct FakeQuery<'args> {
    pub arguments: ArgumentHolder<'args>,
    pub conditions: Vec<SQLCondition>,
}
impl<'args> HasArguments<'args> for FakeQuery<'args> {
    fn holder(&mut self) -> &mut ArgumentHolder<'args> {
        &mut self.arguments
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
