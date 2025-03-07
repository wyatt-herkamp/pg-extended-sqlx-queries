use std::borrow::Cow;

use crate::prelude::*;
macro_rules! wrap_in_function {
    (
        $(
            $name:ident => $value:literal
        ),*
    ) => {
        $(
            fn $name(self) -> SqlFunctionBuilder<'args>
            where
                Self: Sized,
            {
                self.wrap_in_function($value)
            }
        )*
    };
}
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
    wrap_in_function!(
        lower => "LOWER",
        upper => "UPPER",
        count => "COUNT",
        sum => "SUM",
        avg => "AVG",
        array_agg => "ARRAY_AGG",
        array => "ARRAY",
        any => "ANY",
        abs => "ABS",
        ceil => "CEIL",
        floor => "FLOOR",
        ln => "LN",
        log10 => "LOG10",
        degrees => "DEGREES",
        radians => "RADIANS"
    );

    /// Calls Postgres EXTRACT function on the given expression.
    ///
    /// Reference: https://www.postgresql.org/docs/current/functions-datetime.html#FUNCTIONS-DATETIME-EXTRACT
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
