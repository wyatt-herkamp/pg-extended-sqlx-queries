use std::borrow::Cow;
mod alias;
mod filter;
mod select_expr;
mod value;
pub use alias::*;
pub use filter::*;
mod function;
pub use function::*;
pub use value::*;
mod keywords;
mod math;
pub use math::*;
mod multi;
pub use keywords::*;
mod returning;
pub use returning::*;
mod conflict;
mod other;
use crate::{table_layout::*, traits::FormatSql};
pub use conflict::*;
pub use multi::*;
pub use other::*;
mod cast;
mod collate;
pub use cast::*;
pub use collate::*;
pub use select_expr::*;
pub struct DynExpr<'args>(Box<dyn ExprType<'args> + 'args>);
impl<'args> DynExpr<'args> {
    pub fn new<E>(expr: E) -> Self
    where
        E: ExprType<'args> + 'args,
    {
        Self(Box::new(expr))
    }
}
impl<'args> ExprType<'args> for DynExpr<'args> {
    fn process(self: Box<Self>, args: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        self.0.process(args)
    }

    fn process_unboxed(self, args: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        self.0.process(args)
    }
}

pub trait ExprType<'args> {
    fn process(self: Box<Self>, args: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args;
    fn process_unboxed(self, args: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args;
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    ArgumentIndex(ArgumentIndex),
    Function(SqlFunction),
    Column(DynColumn),
    Condition(Box<SQLCondition>),
    Select(SelectExpr),
    Alias(ExprAlias),
    Wildcard(Wildcard),
    Math(Box<SQLMathExpr>),
    Multiple(MultipleExpr),
    Empty,
    Other(OtherSql),
    Keywords(Keywords),
    Cast(Cast),
}
impl From<DynColumn> for Expr {
    fn from(column: DynColumn) -> Self {
        Expr::Column(column)
    }
}
macro_rules! from_expr {
    (
        $(
            $type:ty => $variant:ident
        ),*
    ) => {
        $(
            impl From<$type> for Expr {
                fn from(value: $type) -> Self {
                    Expr::$variant(value)
                }
            }
        )*
    };
}
macro_rules! from_box_expr {
    (
    $(
        $type:ty => $variant:ident
    ),*
) => {
        $(
            impl From<Box<$type>> for Expr {
                fn from(value: Box<$type>) -> Self {
                    Expr::$variant(value)
                }
            }
            impl From<$type> for Expr {
                fn from(value: $type) -> Self {
                    Expr::$variant(Box::new(value))
                }
            }
        )*
    };
}
from_expr! {
    ArgumentIndex => ArgumentIndex,
    SelectExpr => Select,
    ExprAlias => Alias,
    Wildcard => Wildcard,
    MultipleExpr => Multiple,
    OtherSql => Other,
    Keywords => Keywords

}

from_box_expr!(
   SQLCondition => Condition,
    SQLMathExpr => Math
);

impl FormatSql for Expr {
    fn format_sql(&self) -> Cow<'_, str> {
        match self {
            Expr::ArgumentIndex(index) => index.format_sql(),
            Expr::Function(function) => function.format_sql(),
            Expr::Column(column) => column.full_name(),
            Expr::Condition(condition) => condition.format_sql(),
            Expr::Select(select) => select.format_sql(),
            Expr::Alias(alias) => alias.format_sql(),
            Expr::Wildcard(all) => all.format_sql(),
            Expr::Multiple(multiple) => multiple.format_sql(),
            Expr::Empty => Cow::default(),
            Expr::Math(math) => math.format_sql(),
            Expr::Other(other) => other.format_sql(),
            Expr::Keywords(keywords) => keywords.format_sql(),
            Expr::Cast(cast) => cast.format_sql(),
        }
    }
}

impl<'args> ExprType<'args> for () {
    fn process(self: Box<Self>, _: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        Expr::Empty
    }

    fn process_unboxed(self, _: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        Expr::Empty
    }
}
