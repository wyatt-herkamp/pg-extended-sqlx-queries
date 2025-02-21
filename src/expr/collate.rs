use std::{borrow::Cow, fmt::Display};

use crate::traits::FormatSql;

#[derive(Debug, Clone, PartialEq)]
pub enum Collate {
    /// Collation: tr-TR-x-icu
    TrTrxIcu,
    Other(Cow<'static, str>),
}
impl FormatSql for Collate {
    fn format_sql(&self) -> Cow<'_, str> {
        Cow::Owned(format!(r#"COLLATE "{}""#, self))
    }
}
impl From<&'static str> for Collate {
    fn from(s: &'static str) -> Self {
        Self::Other(Cow::Borrowed(s))
    }
}
impl Display for Collate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TrTrxIcu => write!(f, "tr-TR-x-icu"),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}
