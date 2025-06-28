use crate::traits::FormatSql;
use std::borrow::Cow;

use super::{Expr, ExprType, arguments::ArgumentHolder};
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

    ArrayContains,
    ArrayContainedBy,
    ArrayOverlap,
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
            Self::ArrayContains => Cow::Borrowed("@>"),
            Self::ArrayContainedBy => Cow::Borrowed("<@"),
            Self::ArrayOverlap => Cow::Borrowed("&&"),
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
/// SQL Wildcards
///
/// If the Option is None, it means `*`
///
/// Otherwise it will be `{value}.*`
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Wildcard(pub Option<Cow<'static, str>>);
impl From<&'static str> for Wildcard {
    fn from(s: &'static str) -> Self {
        Wildcard(Some(Cow::Borrowed(s)))
    }
}
impl From<String> for Wildcard {
    fn from(s: String) -> Self {
        Wildcard(Some(Cow::Owned(s)))
    }
}
impl<'args> ExprType<'args> for Wildcard {
    #[inline]
    fn process(self: Box<Self>, _: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        Expr::Wildcard(*self)
    }
    #[inline]
    fn process_unboxed(self, _: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        Expr::Wildcard(self)
    }
}
impl Wildcard {
    pub fn new<'args>() -> Wildcard {
        Wildcard(None)
    }
}
impl FormatSql for Wildcard {
    fn format_sql(&self) -> Cow<'static, str> {
        match &self.0 {
            Some(s) => Cow::Owned(format!("{s}.*")),
            None => Cow::Borrowed("*"),
        }
    }
}

/// Just a way to represent some weird cases in SQL
#[derive(Debug)]
pub struct OtherSql(pub Box<dyn FormatSql + Send + Sync>);
impl PartialEq for OtherSql {
    fn eq(&self, other: &Self) -> bool {
        let this = self.0.format_sql();
        let other = other.0.format_sql();
        this == other
    }
}
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
