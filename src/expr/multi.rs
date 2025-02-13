use std::borrow::Cow;

use crate::FormatSql;

use super::{Aliasable, DynExpr, Expr, ExprType};
pub trait MultipleExprType<'args>: ExprType<'args> {
    fn then<E>(self, function: E) -> MultipleExprBuilder<'args>
    where
        E: ExprType<'args> + 'args;
}
pub struct MultipleExprBuilder<'args> {
    functions: Vec<DynExpr<'args>>,
    phantom: std::marker::PhantomData<&'args ()>,
}
impl<'args> Aliasable<'args> for MultipleExprBuilder<'args> {}
impl<'args> MultipleExprBuilder<'args> {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            phantom: std::marker::PhantomData,
        }
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            functions: Vec::with_capacity(capacity),
            phantom: std::marker::PhantomData,
        }
    }
    pub fn with<E>(function: E) -> Self
    where
        E: ExprType<'args> + 'args,
    {
        Self::with_capacity(2).then(function)
    }
}
impl<'args> MultipleExprType<'args> for MultipleExprBuilder<'args> {
    fn then<E>(mut self, function: E) -> MultipleExprBuilder<'args>
    where
        E: ExprType<'args> + 'args,
    {
        self.functions.push(Box::new(function));
        self
    }
}
#[derive(Debug, Default)]
pub struct MultipleExpr {
    functions: Vec<Expr>,
}
impl MultipleExpr {
    pub fn new(functions: Vec<Expr>) -> Self {
        Self { functions }
    }
}
impl FormatSql for MultipleExpr {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        let functions = self
            .functions
            .iter()
            .map(|function| function.format_sql())
            .collect::<Vec<_>>()
            .join(" ");
        Cow::Owned(functions)
    }
}

impl<'args> ExprType<'args> for MultipleExprBuilder<'args> {
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
        let functions = MultipleExpr {
            functions: self
                .functions
                .into_iter()
                .map(|function| function.process(args))
                .collect(),
        };
        Expr::Multiple(functions)
    }
}

pub fn concat_with_comma(expr: &[Expr]) -> String {
    expr.iter()
        .map(|e| e.format_sql())
        .collect::<Vec<_>>()
        .join(", ")
}
