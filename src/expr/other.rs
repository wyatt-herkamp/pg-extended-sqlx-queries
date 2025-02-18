use std::borrow::Cow;

use super::{arguments::ArgumentHolder, many::FormatSql, Expr, ExprType};
/// SQL Basic Comparisons Types
///
/// This is used in the `WHERE` clause of a query
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SQLComparison {
    /// Equals
    ///
    /// `=`
    Equals,
    /// [LIKE](https://www.postgresql.org/docs/current/functions-matching.html#FUNCTIONS-LIKE)
    ///
    /// `LIKE`
    Like,
    /// Not Equals
    ///
    /// `!=`
    NotEquals,
    /// Greater Than
    GreaterThan,
    /// Less Than
    LessThan,
    /// Greater Than or Equals
    GreaterThanOrEqualTo,
    /// Less Than or Equals
    LessThanOrEqualTo,
}
impl FormatSql for SQLComparison {
    fn format_sql(&self) -> Cow<'static, str> {
        match self {
            Self::Equals => Cow::Borrowed("="),
            Self::NotEquals => Cow::Borrowed("!="),
            Self::Like => Cow::Borrowed("LIKE"),
            Self::GreaterThan => Cow::Borrowed(">"),
            Self::LessThan => Cow::Borrowed("<"),
            Self::GreaterThanOrEqualTo => Cow::Borrowed(">="),
            Self::LessThanOrEqualTo => Cow::Borrowed("<="),
        }
    }
}

/// SQL Ordering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SQLOrder {
    Ascending,
    Descending,
}
impl FormatSql for SQLOrder {
    fn format_sql(&self) -> Cow<'static, str> {
        match self {
            Self::Ascending => Cow::Borrowed("ASC"),
            Self::Descending => Cow::Borrowed("DESC"),
        }
    }
}
/// SQL And Or
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AndOr {
    And,
    Or,
}
impl FormatSql for AndOr {
    fn format_sql(&self) -> Cow<'static, str> {
        match self {
            Self::And => Cow::Borrowed("AND"),
            Self::Or => Cow::Borrowed("OR"),
        }
    }
}

#[derive(Debug, Default)]
pub struct All;
impl<'args> ExprType<'args> for All {
    #[inline]
    fn process(self: Box<Self>, _: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        Expr::All(*self)
    }
    #[inline]
    fn process_unboxed(self, _: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        Expr::All(self)
    }
}
impl All {
    pub fn new<'args>() -> All {
        All
    }
}
impl FormatSql for All {
    fn format_sql(&self) -> Cow<'_, str> {
        Cow::Borrowed("*")
    }
}

#[derive(Debug, Default)]
pub struct SqlDefault;
impl<'args> ExprType<'args> for SqlDefault {
    #[inline]
    fn process(self: Box<Self>, _: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        Expr::Default(*self)
    }
    #[inline]
    fn process_unboxed(self, _: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        Expr::Default(self)
    }
}

impl FormatSql for SqlDefault {
    fn format_sql(&self) -> Cow<'_, str> {
        Cow::Borrowed("DEFAULT")
    }
}
