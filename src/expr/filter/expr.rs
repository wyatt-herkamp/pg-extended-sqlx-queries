use crate::{ExprType, SQLComparison};

use super::FilterConditionBuilder;

pub trait FilterExpr<'args>: ExprType<'args> {
    fn equals<E>(self, value: E) -> FilterConditionBuilder<'args>
    where
        Self: Sized + 'args,
        E: ExprType<'args> + 'args,
    {
        self.compare(SQLComparison::Equals, value)
    }
    fn like<E>(self, value: E) -> FilterConditionBuilder<'args>
    where
        Self: Sized + 'args,
        E: ExprType<'args> + 'args,
    {
        self.compare(SQLComparison::Like, value)
    }
    fn is_not_null(self) -> FilterConditionBuilder<'args>
    where
        Self: Sized + 'args,
    {
        FilterConditionBuilder::NotNull(Box::new(self))
    }
    fn is_null(self) -> FilterConditionBuilder<'args>
    where
        Self: Sized + 'args,
    {
        FilterConditionBuilder::Null(Box::new(self))
    }

    fn less_than<E>(self, value: E) -> FilterConditionBuilder<'args>
    where
        Self: Sized + 'args,
        E: ExprType<'args> + 'args,
    {
        self.compare(SQLComparison::LessThan, value)
    }
    fn less_than_or_equals<E>(self, value: E) -> FilterConditionBuilder<'args>
    where
        Self: Sized + 'args,
        E: ExprType<'args> + 'args,
    {
        self.compare(SQLComparison::LessThanOrEqualTo, value)
    }

    fn greater_than<E>(self, value: E) -> FilterConditionBuilder<'args>
    where
        Self: Sized + 'args,
        E: ExprType<'args> + 'args,
    {
        self.compare(SQLComparison::GreaterThan, value)
    }
    fn greater_than_or_equals<E>(self, value: E) -> FilterConditionBuilder<'args>
    where
        Self: Sized + 'args,
        E: ExprType<'args> + 'args,
    {
        self.compare(SQLComparison::GreaterThanOrEqualTo, value)
    }
    fn between<L, R>(self, start: L, end: R) -> FilterConditionBuilder<'args>
    where
        Self: Sized + 'args,
        L: ExprType<'args> + 'args,
        R: ExprType<'args> + 'args,
    {
        FilterConditionBuilder::Between {
            start: Box::new(start),
            end: Box::new(end),
        }
    }
    /// Compares the current Self with a provided Binary Comparison to the right
    fn compare<L>(self, comparison: SQLComparison, value: L) -> FilterConditionBuilder<'args>
    where
        Self: Sized + 'args,
        L: ExprType<'args> + 'args,
    {
        FilterConditionBuilder::CompareValue {
            left: Box::new(self),
            comparison,
            right: Box::new(value),
        }
    }
}

impl<'args, E> FilterExpr<'args> for E where E: ExprType<'args> {}
