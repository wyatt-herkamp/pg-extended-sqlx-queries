use crate::traits::FormatSql;

use super::{ArgumentHolder, Expr, ExprType, ExtractType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keywords {
    From,
    Select,
    Where,

    ExtractTypes(ExtractType),
}
impl From<ExtractType> for Keywords {
    fn from(extract: ExtractType) -> Self {
        Keywords::ExtractTypes(extract)
    }
}
impl FormatSql for Keywords {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        match self {
            Keywords::From => std::borrow::Cow::Borrowed("FROM"),
            Keywords::Select => std::borrow::Cow::Borrowed("SELECT"),
            Keywords::Where => std::borrow::Cow::Borrowed("WHERE"),
            Keywords::ExtractTypes(extract) => extract.format_sql(),
        }
    }
}

impl<'args> ExprType<'args> for Keywords {
    fn process(self: Box<Self>, _: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        Expr::Keywords(*self)
    }

    fn process_unboxed(self, _: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        Expr::Keywords(self)
    }
}
