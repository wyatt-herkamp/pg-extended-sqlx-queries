use sqlx::{Database, Postgres};

use crate::{
    FormatSqlQuery, FormatWhere, HasArguments, QueryScalarTool, QueryTool, SQLCondition,
    WhereableTool,
};
/// Counts the number of rows in a table based on the given where comparisons.
pub struct SelectCount<'args> {
    table: &'static str,
    where_comparisons: Vec<SQLCondition>,
    sql: Option<String>,
    arguments: Option<<Postgres as Database>::Arguments<'args>>,
}
impl<'args> WhereableTool<'args> for SelectCount<'args> {
    fn push_where_comparison(&mut self, comparison: SQLCondition) {
        self.where_comparisons.push(comparison);
    }
}
impl FormatWhere for SelectCount<'_> {
    fn get_conditions(&self) -> &[SQLCondition] {
        &self.where_comparisons
    }
}
impl SelectCount<'_> {
    pub fn new(table: &'static str) -> Self {
        Self {
            table,
            where_comparisons: Vec::new(),
            sql: None,
            arguments: Some(Default::default()),
        }
    }
}
impl HasArguments<'_> for SelectCount<'_> {
    fn take_arguments_or_error(&mut self) -> <Postgres as Database>::Arguments<'_> {
        self.arguments.take().expect("Arguments already taken")
    }
    fn borrow_arguments_or_error(&mut self) -> &mut <Postgres as Database>::Arguments<'_> {
        self.arguments.as_mut().expect("Arguments already taken")
    }
}

impl QueryTool<'_> for SelectCount<'_> {}
impl FormatSqlQuery for SelectCount<'_> {
    fn format_sql_query(&mut self) -> &str {
        let mut sql = format!("SELECT COUNT(1) FROM {}", self.table);

        if !self.where_comparisons.is_empty() {
            let where_sql = self.format_where();
            sql.push_str(" WHERE ");
            sql.push_str(&where_sql);
        }

        self.sql = Some(sql);

        self.sql.as_ref().expect("SQL not set")
    }
}
impl QueryScalarTool<'_> for SelectCount<'_> {
    type Output = i64;
}

#[cfg(test)]
mod tests {
    use sqlformat::{FormatOptions, QueryParams};

    use crate::{
        testing::{TestTable, TestTableColumn},
        DynEncodeType, FilterExpr, FormatSqlQuery, SelectCount, TableType, WhereableTool,
    };

    #[test]
    pub fn count_people_who_are_18() {
        let mut query = SelectCount::new(TestTable::table_name());
        query.filter(TestTableColumn::Age.equals(18.value()));

        let sql = query.format_sql_query();
        assert_eq!(
            sql,
            "SELECT COUNT(1) FROM test_table WHERE test_table.age = $1"
        );

        let sql = sqlformat::format(sql, &QueryParams::None, &FormatOptions::default());

        println!("{}", sql);
    }
}
