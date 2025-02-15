use crate::{AndOr, FormatSql, SQLComparison};

use super::{Expr, ExprType};
mod expr;
pub use expr::*;
pub enum FilterConditionBuilder<'args> {
    CompareValue {
        left: Box<dyn ExprType<'args> + 'args>,
        comparison: SQLComparison,
        right: Box<dyn ExprType<'args> + 'args>,
    },
    Between {
        start: Box<dyn ExprType<'args> + 'args>,
        end: Box<dyn ExprType<'args> + 'args>,
    },
    NotNull(Box<dyn ExprType<'args> + 'args>),
    Null(Box<dyn ExprType<'args> + 'args>),

    Then {
        left: Box<FilterConditionBuilder<'args>>,
        and_or: AndOr,
        right: Box<FilterConditionBuilder<'args>>,
    },
}
impl<'args> FilterConditionBuilder<'args> {
    pub fn and(self, right: FilterConditionBuilder<'args>) -> FilterConditionBuilder<'args> {
        FilterConditionBuilder::Then {
            left: Box::new(self),
            and_or: AndOr::And,
            right: Box::new(right),
        }
    }
    pub fn or(self, right: FilterConditionBuilder<'args>) -> FilterConditionBuilder<'args> {
        FilterConditionBuilder::Then {
            left: Box::new(self),
            and_or: AndOr::Or,
            right: Box::new(right),
        }
    }
    pub(crate) fn process_inner(self, args: &mut dyn crate::HasArguments<'args>) -> SQLCondition {
        match self {
            FilterConditionBuilder::CompareValue {
                left,
                comparison,
                right,
            } => SQLCondition::CompareValue {
                left: left.process(args),
                comparison,
                right: right.process(args),
            },
            FilterConditionBuilder::Between { start, end } => SQLCondition::Between {
                start: start.process(args),
                end: end.process(args),
            },
            FilterConditionBuilder::NotNull(expr) => SQLCondition::NotNull(expr.process(args)),
            FilterConditionBuilder::Null(expr) => SQLCondition::Null(expr.process(args)),
            FilterConditionBuilder::Then {
                left,
                and_or,
                right,
            } => SQLCondition::Then {
                left: left.process(args),
                and_or,
                right: right.process(args),
            },
        }
    }
}
impl<'args> ExprType<'args> for FilterConditionBuilder<'args> {
    fn process(self: Box<Self>, args: &mut dyn crate::HasArguments<'args>) -> Expr
    where
        Self: 'args,
    {
        self.process_unboxed(args)
    }

    fn process_unboxed(self, args: &mut dyn crate::HasArguments<'args>) -> Expr
    where
        Self: 'args,
    {
        let condition = self.process_inner(args);
        Expr::Condition(Box::new(condition))
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
                format!("{} AND {}", start.format_sql(), end.format_sql()).into()
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
