use crate::expr::{ExprType, SQLComparison};

use super::{FilterConditionBuilder, FilterConditionBuilderInner, OneSidedFilterConditionExprType};
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
        FilterConditionBuilderInner::NotNull(self).into()
    }
    fn is_null(self) -> FilterConditionBuilder<'args, Self, OneSidedFilterConditionExprType>
    where
        Self: Sized + 'args,
    {
        FilterConditionBuilderInner::Null(self).into()
    }

    fn between<R>(self, start: R, end: R) -> FilterConditionBuilder<'args, Self, R>
    where
        Self: Sized + 'args,
        R: ExprType<'args> + 'args,
    {
        FilterConditionBuilderInner::Between {
            value: self,
            start,
            end,
        }
        .into()
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
        FilterConditionBuilderInner::CompareValue {
            left: self,
            comparison,
            right: value,
        }
        .into()
    }
}

impl<'args, E> FilterExpr<'args> for E where E: ExprType<'args> {}
