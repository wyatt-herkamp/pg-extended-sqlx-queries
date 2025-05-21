use std::borrow::Cow;

use crate::traits::FormatSql;

use super::{DynExpr, Expr, ExprType};

pub struct CastBuilder<'args> {
    pub(crate) expr: DynExpr<'args>,
    pub(crate) cast_type: Cow<'static, str>,
}
impl<'args> ExprType<'args> for CastBuilder<'args> {
    fn process(self: Box<Self>, args: &mut super::ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        self.process_unboxed(args)
    }

    fn process_unboxed(self, args: &mut super::ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        let inner = self.expr.process_unboxed(args);
        Expr::Cast(Cast {
            expr: Box::new(inner),
            as_type: self.cast_type,
        })
    }
}
#[derive(Debug, PartialEq)]
pub struct Cast {
    expr: Box<Expr>,
    as_type: Cow<'static, str>,
}
impl FormatSql for Cast {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        Cow::Owned(format!("{}::{}", self.expr.format_sql(), self.as_type))
    }
}
macro_rules! cast_fn {
    (
        $(
            $fn_name:ident => $cast_type:literal
        ),*
    ) => {
        $(
            fn $fn_name(self) -> CastBuilder<'args>
            where
                Self: Sized + 'args,
            {
                self.cast($cast_type)
            }
        )*
    };
}
pub trait CastableExprType<'args>: ExprType<'args> {
    fn cast(self, cast_type: impl Into<Cow<'static, str>>) -> CastBuilder<'args>
    where
        Self: Sized + 'args,
    {
        CastBuilder {
            expr: DynExpr::new(self),
            cast_type: cast_type.into(),
        }
    }

    cast_fn! {
        as_date => "DATE",
        as_text => "TEXT"
    }
}
