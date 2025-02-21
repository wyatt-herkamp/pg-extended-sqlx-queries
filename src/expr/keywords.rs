use std::borrow::Cow;

use crate::traits::FormatSql;

use super::{ArgumentHolder, Expr, ExprType, ExtractType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keywords {
    From,
    Select,
    Where,
    Default,
    Null,
    Limit,
    Offset,
    /// Extract types
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
            Keywords::Default => std::borrow::Cow::Borrowed("DEFAULT"),
            Keywords::Null => std::borrow::Cow::Borrowed("NULL"),
            Keywords::Limit => std::borrow::Cow::Borrowed("LIMIT"),
            Keywords::Offset => std::borrow::Cow::Borrowed("OFFSET"),
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

macro_rules! keyword_type {
    (
        $ty:ty => $variant:ident
    ) => {
        impl From<$ty> for Keywords {
            #[inline]
            fn from(_: $ty) -> Self {
                Keywords::$variant
            }
        }
        impl From<$ty> for Expr {
            #[inline]
            fn from(_: $ty) -> Self {
                Keywords::$variant.into()
            }
        }
        impl FormatSql for $ty {
            #[inline]
            fn format_sql(&self) -> Cow<'_, str> {
                Keywords::$variant.format_sql()
            }
        }
        impl<'args> ExprType<'args> for $ty {
            #[inline]
            fn process(self: Box<Self>, _: &mut ArgumentHolder<'args>) -> Expr
            where
                Self: 'args,
            {
                Expr::Keywords(Keywords::$variant)
            }
            #[inline]
            fn process_unboxed(self, _: &mut ArgumentHolder<'args>) -> Expr
            where
                Self: 'args,
            {
                Expr::Keywords(Keywords::$variant)
            }
        }
    };
}
#[derive(Debug, Default)]
pub struct SqlDefault;
keyword_type!(SqlDefault => Default);

#[derive(Debug, Default)]
pub struct SqlNull;
keyword_type!(SqlNull => Null);
