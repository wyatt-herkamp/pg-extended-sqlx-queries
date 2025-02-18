use crate::traits::FormatSql;
use std::borrow::Cow;

use super::{arguments::ArgumentHolder, Expr, ExprType};

pub struct ExprAsType<'args, E: ExprType<'args>> {
    expr: E,
    alias: Cow<'static, str>,
    phantom: std::marker::PhantomData<&'args E>,
}
impl<'args, E> ExprAsType<'args, E>
where
    E: ExprType<'args>,
{
    pub fn new(expr: E, alias: impl Into<Cow<'static, str>>) -> Self {
        Self {
            expr,
            alias: alias.into(),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'args, E> ExprType<'args> for ExprAsType<'args, E>
where
    E: ExprType<'args>,
{
    fn process(self: Box<Self>, args: &mut ArgumentHolder<'args>) -> super::Expr {
        self.process_unboxed(args)
    }

    fn process_unboxed(self, args: &mut ArgumentHolder<'args>) -> super::Expr {
        let inner = self.expr.process_unboxed(args);
        Expr::Alias(ExprAlias {
            expr: Box::new(inner),
            alias: self.alias,
        })
    }
}
#[derive(Debug)]
pub struct ExprAlias {
    expr: Box<Expr>,
    alias: Cow<'static, str>,
}
impl FormatSql for ExprAlias {
    fn format_sql(&self) -> Cow<'_, str> {
        Cow::Owned(format!("{} AS {}", self.expr.format_sql(), self.alias))
    }
}

pub trait Aliasable<'args>: ExprType<'args> {
    fn alias(self, alias: impl Into<Cow<'static, str>>) -> ExprAsType<'args, Self>
    where
        Self: Sized + 'args,
    {
        ExprAsType::new(self, alias)
    }
}
