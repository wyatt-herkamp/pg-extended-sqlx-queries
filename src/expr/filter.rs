mod builder;
use crate::traits::FormatSql;

use super::{AndOr, DynExpr, Expr, ExprType, SQLComparison, arguments::ArgumentHolder};
mod expr;
use builder::FilterConditionBuilderInner;
pub use expr::*;

pub struct FilterConditionBuilder<'args, L: ExprType<'args> + 'args, R: ExprType<'args> + 'args>(
    FilterConditionBuilderInner<'args, L, R>,
);
impl<'args, L: ExprType<'args> + 'args, R: ExprType<'args> + 'args>
    From<FilterConditionBuilderInner<'args, L, R>> for FilterConditionBuilder<'args, L, R>
{
    fn from(value: FilterConditionBuilderInner<'args, L, R>) -> Self {
        Self(value)
    }
}

impl<'args, L: ExprType<'args> + 'args, R: ExprType<'args> + 'args>
    FilterConditionBuilder<'args, L, R>
{
    pub fn and<RL: ExprType<'args>, RR: ExprType<'args>>(
        self,
        right: FilterConditionBuilder<'args, RL, RR>,
    ) -> FilterConditionBuilder<'args, Self, FilterConditionBuilder<'args, RL, RR>> {
        FilterConditionBuilderInner::Then {
            left: self,
            and_or: AndOr::And,
            right: right,
        }
        .into()
    }
    pub fn or<RL: ExprType<'args>, RR: ExprType<'args>>(
        self,
        right: FilterConditionBuilder<'args, RL, RR>,
    ) -> FilterConditionBuilder<'args, Self, FilterConditionBuilder<'args, RL, RR>> {
        FilterConditionBuilderInner::Then {
            left: self,
            and_or: AndOr::Or,
            right: right,
        }
        .into()
    }
    /// Groups the current expression in parenthesis
    pub fn grouped(self) -> FilterConditionBuilder<'args, Self, ()> {
        FilterConditionBuilderInner::Grouped(self).into()
    }
    pub fn not(self) -> FilterConditionBuilder<'args, Self, ()> {
        FilterConditionBuilderInner::Not(self).into()
    }
    pub fn dyn_expression(self) -> FilterConditionBuilder<'args, DynExpr<'args>, DynExpr<'args>> {
        self.0.dyn_expression().into()
    }

    pub(crate) fn process_inner(self, args: &mut ArgumentHolder<'args>) -> SQLCondition {
        self.0.process_inner(args)
    }
}

impl<'args, L: ExprType<'args> + 'args, R: ExprType<'args> + 'args> ExprType<'args>
    for FilterConditionBuilder<'args, L, R>
{
    fn process(self: Box<Self>, args: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        self.process_unboxed(args)
    }

    fn process_unboxed(self, args: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        Expr::Condition(Box::new(self.process_inner(args)))
    }
}
#[derive(Debug, PartialEq)]
pub enum SQLCondition {
    /// Generic comparison between two values
    CompareValue {
        left: Expr,
        comparison: SQLComparison,
        right: Expr,
    },
    /// Represents a Postgres `BETWEEN` condition
    /// [Postgres Documentation](https://www.postgresql.org/docs/current/functions-comparison.html)
    Between { value: Expr, start: Expr, end: Expr },
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
    /// Represents a Postgres NOT condition
    ///
    /// NOT {expr}
    Not(Expr),
    /// {EXPR} COLLATE {COLLATE}
    Collate {
        expression: Expr,
        collate: crate::expr::collate::Collate,
    },
    /// ({EXPR})
    Grouped(Expr),
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
            SQLCondition::Between { value, start, end } => format!(
                "{} BETWEEN {} AND {}",
                value.format_sql(),
                start.format_sql(),
                end.format_sql()
            )
            .into(),
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
            SQLCondition::Not(expr) => format!("NOT {}", expr.format_sql()).into(),
            SQLCondition::Collate {
                expression,
                collate,
            } => format!("{} {}", expression.format_sql(), collate.format_sql()).into(),

            SQLCondition::Grouped(expr) => format!("({})", expr.format_sql()).into(),
        }
    }
}
