use crate::traits::FormatSql;
use std::borrow::Cow;

use super::{arguments::ArgumentHolder, Expr, ExprType};
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
    /// ILike (Case Insensitive Like)
    ILike,
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
            Self::ILike => Cow::Borrowed("ILIKE"),
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

#[derive(Debug, Default)]
pub struct SqlNull;
impl<'args> ExprType<'args> for SqlNull {
    #[inline]
    fn process(self: Box<Self>, _: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        Expr::Null(*self)
    }
    #[inline]
    fn process_unboxed(self, _: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        Expr::Null(self)
    }
}

impl FormatSql for SqlNull {
    fn format_sql(&self) -> Cow<'_, str> {
        Cow::Borrowed("NULL")
    }
}
/// Just a way to represent some weird cases in SQL
#[derive(Debug)]
pub struct OtherSql(pub Box<dyn FormatSql + Send + Sync>);
impl OtherSql {
    pub fn new<T>(value: T) -> OtherSql
    where
        T: FormatSql + Send + Sync + 'static,
    {
        OtherSql(Box::new(value))
    }
}

impl FormatSql for OtherSql {
    fn format_sql(&self) -> Cow<'_, str> {
        self.0.format_sql()
    }
}

impl<'args> ExprType<'args> for OtherSql {
    fn process(self: Box<Self>, _: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        Expr::Other(*self)
    }

    fn process_unboxed(self, _: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        Expr::Other(self)
    }
}
