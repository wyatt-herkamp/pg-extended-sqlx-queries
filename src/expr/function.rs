use std::borrow::Cow;
mod expr;
pub use expr::*;

use crate::traits::FormatSql;

use super::{
    Aliasable, All, ArgumentHolder, Expr, ExprType, MultipleExpr, MultipleExprBuilder,
    MultipleExprType,
};

pub struct SqlFunctionBuilder<'args> {
    function_name: Cow<'static, str>,
    params: MultipleExprBuilder<'args>,
}
impl<'args> Aliasable<'args> for SqlFunctionBuilder<'args> {}
impl<'args> SqlFunctionBuilder<'args> {
    pub fn new(function_name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            function_name: function_name.into(),
            params: MultipleExprBuilder::new(),
        }
    }
    pub fn add_param<E>(mut self, param: E) -> Self
    where
        E: ExprType<'args> + 'args,
    {
        self.params = self.params.push(param);
        self
    }
    pub fn now() -> Self {
        Self::new("NOW")
    }
    pub fn count() -> Self {
        Self::new("COUNT")
    }
    /// Alias for `COUNT(*)`
    ///
    /// ```rust
    ///  use pg_extended_sqlx_queries::prelude::*;
    ///  use pg_extended_sqlx_queries::fake::FakeQuery;
    ///  use crate::pg_extended_sqlx_queries::expr::arguments::HasArguments;
    ///  let mut fake_query = FakeQuery::default();
    ///  let expr = SqlFunctionBuilder::count_all();
    ///  let other_way = SqlFunctionBuilder::count().add_param(All::new());
    ///
    ///  let expr = expr.process_unboxed(&mut fake_query.holder());
    ///  let other_way = other_way.process_unboxed(&mut fake_query.holder());
    ///
    ///  let expr = expr.format_sql();
    ///  let other_way = other_way.format_sql();
    ///  assert_eq!(expr, other_way);
    ///  ```
    pub fn count_all() -> Self {
        Self::new("COUNT").add_param(All::new())
    }
    pub fn over() -> Self {
        Self::new("OVER")
    }
    pub fn array() -> Self {
        Self::new("ARRAY")
    }
    pub fn array_agg() -> Self {
        Self::new("ARRAY_AGG")
    }
    pub fn sum() -> Self {
        Self::new("SUM")
    }
    pub fn avg() -> Self {
        Self::new("AVG")
    }
    pub fn lower() -> Self {
        Self::new("LOWER")
    }
    pub fn upper() -> Self {
        Self::new("UPPER")
    }
}
impl<'args> ExprType<'args> for SqlFunctionBuilder<'args> {
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
        let params = self.params.process_inner_with_seperator(args, ", ");

        let function = SqlFunction {
            function_name: self.function_name,
            params: params,
        };
        Expr::Function(function)
    }
}
impl<'args> MultipleExprType<'args> for SqlFunctionBuilder<'args> {
    fn then<E>(self, function: E) -> super::MultipleExprBuilder<'args>
    where
        E: ExprType<'args> + 'args,
    {
        MultipleExprBuilder::with(self).then(function)
    }
}
#[derive(Debug, Default)]
pub struct SqlFunction<Params: FormatSql = MultipleExpr> {
    function_name: Cow<'static, str>,
    params: Params,
}

impl<Params: FormatSql> FormatSql for SqlFunction<Params> {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        let params = self.params.format_sql();
        Cow::Owned(format!("{}({params})", self.function_name))
    }
}

#[cfg(test)]
mod tests {
    use crate::fake::FakeQuery;
    use crate::prelude::*;

    use super::SqlFunctionBuilder;

    #[test]
    pub fn many_functions() {
        let expr = SqlFunctionBuilder::count_all()
            .then(SqlFunctionBuilder::over())
            .alias("count_over");

        // Code for faking the query
        let mut parent = FakeQuery::default();
        let expr = expr.process_unboxed(&mut parent.arguments);

        assert_eq!(expr.format_sql(), "COUNT(*) OVER() AS count_over");
    }
}
