use std::marker::PhantomData;

use crate::{AndOr, FormatSql, SQLComparison};

use super::{arguments::ArgumentHolder, DynExpr, Expr, ExprType};
mod expr;
pub use expr::*;
pub struct OneSidedFilterConditionExprType(PhantomData<()>);

impl<'args> ExprType<'args> for OneSidedFilterConditionExprType {
    fn process(self: Box<Self>, _: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        unimplemented!("OneSidedFilterConditionExprType cannot be processed")
    }

    fn process_unboxed(self, _: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        unimplemented!("OneSidedFilterConditionExprType cannot be processed")
    }
}
pub enum FilterConditionBuilder<'args, L: ExprType<'args> + 'args, R: ExprType<'args> + 'args> {
    CompareValue {
        left: L,
        comparison: SQLComparison,
        right: R,
    },
    Between {
        start: L,
        end: R,
    },
    NotNull(L),
    Null(L),

    Then {
        left: L,
        and_or: AndOr,
        right: R,
    },
    Hidden(PhantomData<&'args ()>),
}

impl<'args, L: ExprType<'args> + 'args, R: ExprType<'args> + 'args>
    FilterConditionBuilder<'args, L, R>
{
    pub fn and<RL: ExprType<'args>, RR: ExprType<'args>>(
        self,
        right: FilterConditionBuilder<'args, RL, RR>,
    ) -> FilterConditionBuilder<'args, Self, FilterConditionBuilder<'args, RL, RR>> {
        FilterConditionBuilder::Then {
            left: self,
            and_or: AndOr::And,
            right: right,
        }
    }
    pub fn or<RL: ExprType<'args>, RR: ExprType<'args>>(
        self,
        right: FilterConditionBuilder<'args, RL, RR>,
    ) -> FilterConditionBuilder<'args, Self, FilterConditionBuilder<'args, RL, RR>> {
        FilterConditionBuilder::Then {
            left: self,
            and_or: AndOr::Or,
            right: right,
        }
    }
    pub fn dyn_expression(self) -> FilterConditionBuilder<'args, DynExpr<'args>, DynExpr<'args>> {
        match self {
            FilterConditionBuilder::CompareValue {
                left,
                comparison,
                right,
            } => FilterConditionBuilder::CompareValue {
                left: DynExpr::new(left),
                comparison,
                right: DynExpr::new(right),
            },
            FilterConditionBuilder::Between { start, end } => FilterConditionBuilder::Between {
                start: DynExpr::new(start),
                end: DynExpr::new(end),
            },
            FilterConditionBuilder::NotNull(expr) => {
                FilterConditionBuilder::NotNull(DynExpr::new(expr))
            }
            FilterConditionBuilder::Null(expr) => FilterConditionBuilder::Null(DynExpr::new(expr)),
            FilterConditionBuilder::Then {
                left,
                and_or,
                right,
            } => FilterConditionBuilder::Then {
                left: DynExpr::new(left),
                and_or,
                right: DynExpr::new(right),
            },
            FilterConditionBuilder::Hidden(_) => unreachable!(),
        }
    }

    pub(crate) fn process_inner(self, args: &mut ArgumentHolder<'args>) -> SQLCondition {
        match self {
            FilterConditionBuilder::CompareValue {
                left,
                comparison,
                right,
            } => SQLCondition::CompareValue {
                left: left.process_unboxed(args),
                comparison,
                right: right.process_unboxed(args),
            },
            FilterConditionBuilder::Between { start, end } => SQLCondition::Between {
                start: start.process_unboxed(args),
                end: end.process_unboxed(args),
            },
            FilterConditionBuilder::NotNull(expr) => {
                SQLCondition::NotNull(expr.process_unboxed(args))
            }
            FilterConditionBuilder::Null(expr) => SQLCondition::Null(expr.process_unboxed(args)),
            FilterConditionBuilder::Then {
                left,
                and_or,
                right,
            } => SQLCondition::Then {
                left: left.process_unboxed(args),
                and_or,
                right: right.process_unboxed(args),
            },
            FilterConditionBuilder::Hidden(_) => unreachable!(),
        }
    }
}

impl<'args, L: ExprType<'args> + 'args, R: ExprType<'args> + 'args> ExprType<'args>
    for FilterConditionBuilder<'args, L, R>
{
    fn process(self: Box<Self>, args: &mut crate::arguments::ArgumentHolder<'args>) -> crate::Expr
    where
        Self: 'args,
    {
        self.process_unboxed(args)
    }

    fn process_unboxed(self, args: &mut crate::arguments::ArgumentHolder<'args>) -> crate::Expr
    where
        Self: 'args,
    {
        crate::Expr::Condition(Box::new(self.process_inner(args)))
    }
}
#[derive(Debug)]
pub enum SQLCondition {
    /// Generic comparison between two values
    CompareValue {
        left: Expr,
        comparison: SQLComparison,
        right: Expr,
    },
    /// Represents a Postgres `BETWEEN` condition
    /// [Postgres Documentation](https://www.postgresql.org/docs/current/functions-comparison.html)
    Between { start: Expr, end: Expr },
    /// Represents a Postgres `IS NOT NULL` condition
    NotNull(Expr),

    /// Represents a Postgres `IS NULL` condition
    Null(Expr),

    /// Represents a Postgres AND or OR condition
    Then {
        left: Expr,
        and_or: AndOr,
        right: Expr,
    },
}
impl FormatSql for SQLCondition {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        match self {
            SQLCondition::CompareValue {
                left,
                comparison,
                right,
            } => format!(
                "{} {} {}",
                left.format_sql(),
                comparison.format_sql(),
                right.format_sql()
            )
            .into(),
            SQLCondition::Between { start, end } => {
                format!("BETWEEN {} AND {}", start.format_sql(), end.format_sql()).into()
            }
            SQLCondition::NotNull(expr) => format!("{} IS NOT NULL", expr.format_sql()).into(),
            SQLCondition::Null(expr) => format!("{} IS NULL", expr.format_sql()).into(),
            SQLCondition::Then {
                left,
                and_or,
                right,
            } => format!(
                "{} {} {}",
                left.format_sql(),
                and_or.format_sql(),
                right.format_sql()
            )
            .into(),
        }
    }
}
