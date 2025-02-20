use std::borrow::Cow;

use crate::prelude::*;

pub trait WrapInFunction<'args>: ExprType<'args> + 'args {
    fn wrap_in_function(
        self,
        function_name: impl Into<Cow<'static, str>>,
    ) -> SqlFunctionBuilder<'args>
    where
        Self: Sized,
    {
        SqlFunctionBuilder::new(function_name).add_param(self)
    }

    fn lower(self) -> SqlFunctionBuilder<'args>
    where
        Self: Sized,
    {
        self.wrap_in_function("LOWER")
    }
    fn upper(self) -> SqlFunctionBuilder<'args>
    where
        Self: Sized,
    {
        self.wrap_in_function("UPPER")
    }
    fn count(self) -> SqlFunctionBuilder<'args>
    where
        Self: Sized,
    {
        self.wrap_in_function("COUNT")
    }
    fn sum(self) -> SqlFunctionBuilder<'args>
    where
        Self: Sized,
    {
        self.wrap_in_function("SUM")
    }
    fn avg(self) -> SqlFunctionBuilder<'args>
    where
        Self: Sized,
    {
        self.wrap_in_function("AVG")
    }
    fn array_agg(self) -> SqlFunctionBuilder<'args>
    where
        Self: Sized,
    {
        self.wrap_in_function("ARRAY_AGG")
    }
    fn array(self) -> SqlFunctionBuilder<'args>
    where
        Self: Sized,
    {
        self.wrap_in_function("ARRAY")
    }
    fn any(self) -> SqlFunctionBuilder<'args>
    where
        Self: Sized,
    {
        self.wrap_in_function("ANY")
    }

    fn extract(self, field: ExtractType) -> SqlFunctionBuilder<'args>
    where
        Self: Sized,
    {
        let params = MultipleExprBuilder::new()
            .push(field)
            .push(Keywords::From)
            .push(self);
        SqlFunctionBuilder::new("EXTRACT").add_param(params)
    }
}
