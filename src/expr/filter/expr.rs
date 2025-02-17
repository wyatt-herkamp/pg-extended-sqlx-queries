use crate::{ExprType, SQLComparison};

use super::{FilterConditionBuilder, OneSidedFilterConditionExprType};
macro_rules! compairson {
    (
        $(
            $name:ident: $value:ident
        ),*
    ) => {
        $(
        fn $name<E>(self, value: E) -> FilterConditionBuilder<'args, Self, E>
        where
            Self: Sized + 'args,
            E: ExprType<'args> + 'args,
        {
            self.compare(SQLComparison::$value, value)
        }
        )*
    };
}
pub trait FilterExpr<'args>: ExprType<'args> {
    compairson! {
        equals: Equals,
        like: Like,
        less_than: LessThan,
        less_than_or_equals: LessThanOrEqualTo,
        greater_than: GreaterThan,
        greater_than_or_equals: GreaterThanOrEqualTo,
        not_equals: NotEquals
    }
    fn is_not_null(self) -> FilterConditionBuilder<'args, Self, OneSidedFilterConditionExprType>
    where
        Self: Sized + 'args,
    {
        FilterConditionBuilder::NotNull(self)
    }
    fn is_null(self) -> FilterConditionBuilder<'args, Self, OneSidedFilterConditionExprType>
    where
        Self: Sized + 'args,
    {
        FilterConditionBuilder::Null(self)
    }

    fn between<L, R>(self, start: L, end: R) -> FilterConditionBuilder<'args, L, R>
    where
        Self: Sized + 'args,
        L: ExprType<'args> + 'args,
        R: ExprType<'args> + 'args,
    {
        FilterConditionBuilder::Between {
            start: start,
            end: end,
        }
    }
    /// Compares the current Self with a provided Binary Comparison to the right
    fn compare<L>(
        self,
        comparison: SQLComparison,
        value: L,
    ) -> FilterConditionBuilder<'args, Self, L>
    where
        Self: Sized + 'args,
        L: ExprType<'args> + 'args,
    {
        FilterConditionBuilder::CompareValue {
            left: self,
            comparison,
            right: value,
        }
    }
}

impl<'args, E> FilterExpr<'args> for E where E: ExprType<'args> {}
