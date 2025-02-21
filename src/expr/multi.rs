use super::{Aliasable, DynExpr, Expr, ExprType, arguments::ArgumentHolder};
use crate::traits::FormatSql;
use std::borrow::Cow;
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
    pub fn push<E>(mut self, function: E) -> Self
    where
        E: ExprType<'args> + 'args,
    {
        self.functions.push(DynExpr::new(function));
        self
    }
    pub(crate) fn process_inner(self, args: &mut ArgumentHolder<'args>) -> MultipleExpr {
        self.process_inner_with_seperator(args, " ")
    }
    pub(crate) fn process_inner_with_seperator(
        self,
        args: &mut ArgumentHolder<'args>,
        seperator: impl Into<Cow<'static, str>>,
    ) -> MultipleExpr {
        let exprs = MultipleExpr::with_separator(
            self.functions
                .into_iter()
                .map(|function| function.process_unboxed(args))
                .collect(),
            seperator,
        );
        exprs
    }
}
impl<'args> MultipleExprType<'args> for MultipleExprBuilder<'args> {
    fn then<E>(mut self, function: E) -> MultipleExprBuilder<'args>
    where
        E: ExprType<'args> + 'args,
    {
        self.functions.push(DynExpr::new(function));
        self
    }
}
#[derive(Debug, PartialEq)]
pub struct MultipleExpr {
    functions: Vec<Expr>,
    seperator: Cow<'static, str>,
}
impl Default for MultipleExpr {
    fn default() -> Self {
        Self {
            functions: Vec::new(),
            seperator: Cow::Borrowed(" "),
        }
    }
}
impl MultipleExpr {
    pub fn new(functions: Vec<Expr>) -> Self {
        Self {
            functions,
            seperator: Cow::Borrowed(" "),
        }
    }
    pub fn with_separator(functions: Vec<Expr>, seperator: impl Into<Cow<'static, str>>) -> Self {
        Self {
            functions,
            seperator: seperator.into(),
        }
    }
    pub fn set_separator(&mut self, seperator: impl Into<Cow<'static, str>>) {
        self.seperator = seperator.into();
    }
}
impl FormatSql for MultipleExpr {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        let functions = self
            .functions
            .iter()
            .map(|function| function.format_sql())
            .collect::<Vec<_>>()
            .join(&self.seperator);
        Cow::Owned(functions)
    }
}

impl<'args> ExprType<'args> for MultipleExprBuilder<'args> {
    fn process(self: Box<Self>, args: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        self.process_unboxed(args)
    }

    fn process_unboxed(self, args: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        let exprs = self.process_inner(args);
        Expr::Multiple(exprs)
    }
}

pub fn concat_with_comma(expr: &[Expr]) -> String {
    expr.iter()
        .map(|e| e.format_sql())
        .collect::<Vec<_>>()
        .join(", ")
}
