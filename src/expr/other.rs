use std::borrow::Cow;

use crate::FormatSql;

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

    GreaterThan,

    LessThan,

    GreaterThanOrEqualTo,

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
