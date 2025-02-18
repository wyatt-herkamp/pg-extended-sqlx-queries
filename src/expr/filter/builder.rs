use std::marker::PhantomData;

pub use crate::prelude::*;

use super::SQLCondition;

pub(crate) enum FilterConditionBuilderInner<
    'args,
    L: ExprType<'args> + 'args,
    R: ExprType<'args> + 'args,
> {
    CompareValue {
        left: L,
        comparison: SQLComparison,
        right: R,
    },
    Between {
        value: L,
        start: R,
        end: R,
    },
    NotNull(L),
    Not(L),
    Null(L),

    Then {
        left: L,
        and_or: AndOr,
        right: R,
    },
    /// Exists just to make the compiler happy
    #[allow(dead_code)]
    Hidden(PhantomData<&'args ()>),
}
impl<'args, L: ExprType<'args> + 'args, R: ExprType<'args> + 'args>
    FilterConditionBuilderInner<'args, L, R>
{
    pub(crate) fn dyn_expression(
        self,
    ) -> FilterConditionBuilderInner<'args, DynExpr<'args>, DynExpr<'args>> {
        match self {
            Self::CompareValue {
                left,
                comparison,
                right,
            } => FilterConditionBuilderInner::CompareValue {
                left: DynExpr::new(left),
                comparison,
                right: DynExpr::new(right),
            },
            Self::Between { start, end, value } => FilterConditionBuilderInner::Between {
                value: DynExpr::new(value),
                start: DynExpr::new(start),
                end: DynExpr::new(end),
            },
            Self::Not(expr) => FilterConditionBuilderInner::Not(DynExpr::new(expr)),
            Self::NotNull(expr) => FilterConditionBuilderInner::NotNull(DynExpr::new(expr)),
            Self::Null(expr) => FilterConditionBuilderInner::Null(DynExpr::new(expr)),
            Self::Then {
                left,
                and_or,
                right,
            } => FilterConditionBuilderInner::Then {
                left: DynExpr::new(left),
                and_or,
                right: DynExpr::new(right),
            },
            Self::Hidden(_) => unreachable!(),
        }
    }

    pub(crate) fn process_inner(self, args: &mut ArgumentHolder<'args>) -> SQLCondition {
        match self {
            FilterConditionBuilderInner::CompareValue {
                left,
                comparison,
                right,
            } => SQLCondition::CompareValue {
                left: left.process_unboxed(args),
                comparison,
                right: right.process_unboxed(args),
            },
            FilterConditionBuilderInner::Between { value, start, end } => SQLCondition::Between {
                value: value.process_unboxed(args),
                start: start.process_unboxed(args),
                end: end.process_unboxed(args),
            },
            FilterConditionBuilderInner::NotNull(expr) => {
                SQLCondition::NotNull(expr.process_unboxed(args))
            }
            FilterConditionBuilderInner::Null(expr) => {
                SQLCondition::Null(expr.process_unboxed(args))
            }
            FilterConditionBuilderInner::Then {
                left,
                and_or,
                right,
            } => SQLCondition::Then {
                left: left.process_unboxed(args),
                and_or,
                right: right.process_unboxed(args),
            },
            FilterConditionBuilderInner::Not(expr) => SQLCondition::Not(expr.process_unboxed(args)),
            FilterConditionBuilderInner::Hidden(_) => unreachable!(),
        }
    }
}
