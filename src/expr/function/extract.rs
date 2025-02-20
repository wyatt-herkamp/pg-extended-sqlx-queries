use crate::{
    expr::{ArgumentHolder, Expr, ExprType, OtherSql},
    traits::FormatSql,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractType {
    Month,
    Day,
    Year,
    Century,
    Hour,
    Minute,
    Second,
    DayOfYear,
}
impl<'args> ExprType<'args> for ExtractType {
    fn process(self: Box<Self>, _: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        Expr::Other(OtherSql::new(*self))
    }

    fn process_unboxed(self, _: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        Expr::Other(OtherSql::new(self))
    }
}

impl FormatSql for ExtractType {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        match self {
            ExtractType::Month => std::borrow::Cow::Borrowed("MONTH"),
            ExtractType::Day => std::borrow::Cow::Borrowed("DAY"),
            ExtractType::Year => std::borrow::Cow::Borrowed("YEAR"),
            ExtractType::Century => std::borrow::Cow::Borrowed("CENTURY"),
            ExtractType::Hour => std::borrow::Cow::Borrowed("HOUR"),
            ExtractType::Minute => std::borrow::Cow::Borrowed("MINUTE"),
            ExtractType::Second => std::borrow::Cow::Borrowed("SECOND"),
            ExtractType::DayOfYear => std::borrow::Cow::Borrowed("DOY"),
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    mod tests {
        use crate::fake::FakeQuery;
        use crate::prelude::*;
        use crate::testing::TestTableColumn;

        #[test]
        pub fn test_extract() {
            let expr = TestTableColumn::CreatedAt.extract(ExtractType::Day);

            // Code for faking the query
            let mut parent = FakeQuery::default();
            let expr = expr.process_unboxed(&mut parent.arguments);

            assert_eq!(expr.format_sql(), "EXTRACT(DAY FROM test_table.created_at)");
        }
    }
}
