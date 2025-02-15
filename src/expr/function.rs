use std::borrow::Cow;

use crate::FormatSql;

use super::{Aliasable, All, Expr, ExprType, MultipleExprBuilder, MultipleExprType};
pub struct ExprFunctionBuilder<'args> {
    function_name: Cow<'static, str>,
    params: Vec<Box<dyn ExprType<'args> + 'args>>,
    phantom: std::marker::PhantomData<&'args ()>,
}
impl<'args> Aliasable<'args> for ExprFunctionBuilder<'args> {}
impl<'args> ExprFunctionBuilder<'args> {
    pub fn new(function_name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            function_name: function_name.into(),
            params: Vec::new(),
            phantom: std::marker::PhantomData,
        }
    }
    pub fn add_param<E>(mut self, param: E) -> Self
    where
        E: ExprType<'args> + 'args,
    {
        self.params.push(Box::new(param));
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
    ///
    ///  let mut fake_query = FakeQuery::default();
    ///  let expr = ExprFunctionBuilder::count_all();
    ///  let other_way = ExprFunctionBuilder::count().add_param(All::new());
    ///
    ///  let expr = expr.process_unboxed(&mut fake_query);
    ///  let other_way = other_way.process_unboxed(&mut fake_query);
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
}
impl<'args> ExprType<'args> for ExprFunctionBuilder<'args> {
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
        let function = ExprFunction {
            function_name: self.function_name,
            params: self
                .params
                .into_iter()
                .map(|param| param.process(args))
                .collect(),
        };
        Expr::Function(function)
    }
}
#[derive(Debug, Default)]
pub struct ExprFunction {
    function_name: Cow<'static, str>,
    params: Vec<Expr>,
}
impl ExprFunction {
    pub fn now() -> Self {
        Self {
            function_name: "NOW".into(),
            params: Vec::new(),
        }
    }
}

impl FormatSql for ExprFunction {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        let params = self
            .params
            .iter()
            .map(|param| param.format_sql())
            .collect::<Vec<_>>()
            .join(", ");
        Cow::Owned(format!("{}({params})", self.function_name))
    }
}
impl<'args> MultipleExprType<'args> for ExprFunctionBuilder<'args> {
    fn then<E>(self, function: E) -> super::MultipleExprBuilder<'args>
    where
        E: ExprType<'args> + 'args,
    {
        MultipleExprBuilder::with(self).then(function)
    }
}
pub trait WrapInFunction<'args>: ExprType<'args> + 'args {
    fn wrap_in_function(
        self,
        function_name: impl Into<Cow<'static, str>>,
    ) -> ExprFunctionBuilder<'args>
    where
        Self: Sized,
    {
        ExprFunctionBuilder::new(function_name).add_param(self)
    }

    fn lower(self) -> ExprFunctionBuilder<'args>
    where
        Self: Sized,
    {
        self.wrap_in_function("LOWER")
    }
    fn upper(self) -> ExprFunctionBuilder<'args>
    where
        Self: Sized,
    {
        self.wrap_in_function("UPPER")
    }
    fn count(self) -> ExprFunctionBuilder<'args>
    where
        Self: Sized,
    {
        self.wrap_in_function("COUNT")
    }
    fn sum(self) -> ExprFunctionBuilder<'args>
    where
        Self: Sized,
    {
        self.wrap_in_function("SUM")
    }
    fn avg(self) -> ExprFunctionBuilder<'args>
    where
        Self: Sized,
    {
        self.wrap_in_function("AVG")
    }
    fn array_agg(self) -> ExprFunctionBuilder<'args>
    where
        Self: Sized,
    {
        self.wrap_in_function("ARRAY_AGG")
    }
    fn array(self) -> ExprFunctionBuilder<'args>
    where
        Self: Sized,
    {
        self.wrap_in_function("ARRAY")
    }
    fn any(self) -> ExprFunctionBuilder<'args>
    where
        Self: Sized,
    {
        self.wrap_in_function("ANY")
    }
}
#[cfg(test)]
mod tests {
    use crate::{fake::FakeQuery, Aliasable, ExprType, FormatSql, MultipleExprType};

    use super::ExprFunctionBuilder;

    #[test]
    pub fn many_functions() {
        let expr = ExprFunctionBuilder::count_all()
            .then(ExprFunctionBuilder::over())
            .alias("count_over");

        // Code for faking the query
        let mut parent = FakeQuery::default();
        let expr = expr.process_unboxed(&mut parent);

        assert_eq!(expr.format_sql(), "COUNT(*) OVER() AS count_over");
    }
}
