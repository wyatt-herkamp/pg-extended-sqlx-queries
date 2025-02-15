use sqlx::{Database, Postgres};

use crate::{
    FormatSqlQuery, FormatWhere, HasArguments, QueryScalarTool, QueryTool, SQLCondition,
    WhereableTool,
};

pub struct SelectExists<'args> {
    table: &'static str,
    where_comparisons: Vec<SQLCondition>,
    sql: Option<String>,
    arguments: Option<<Postgres as Database>::Arguments<'args>>,
}
impl<'args> WhereableTool<'args> for SelectExists<'args> {
    fn push_where_comparison(&mut self, comparison: crate::SQLCondition) {
        self.where_comparisons.push(comparison);
    }
}
impl FormatWhere for SelectExists<'_> {
    fn get_conditions(&self) -> &[SQLCondition] {
        &self.where_comparisons
    }
}
impl SelectExists<'_> {
    pub fn new(table: &'static str) -> Self {
        Self {
            table,
            where_comparisons: Vec::new(),
            sql: None,
            arguments: Some(Default::default()),
        }
    }
}
impl HasArguments<'_> for SelectExists<'_> {
    fn take_arguments_or_error(&mut self) -> <Postgres as Database>::Arguments<'_> {
        self.arguments.take().expect("Arguments already taken")
    }
    fn borrow_arguments_or_error(&mut self) -> &mut <Postgres as Database>::Arguments<'_> {
        self.arguments.as_mut().expect("Arguments already taken")
    }
}
impl QueryTool<'_> for SelectExists<'_> {}
impl QueryScalarTool<'_> for SelectExists<'_> {
    type Output = bool;
}
impl FormatSqlQuery for SelectExists<'_> {
    fn format_sql_query(&mut self) -> &str {
        let mut sql = format!("SELECT EXISTS (SELECT 1 FROM {} ", self.table);

        if !self.where_comparisons.is_empty() {
            let where_sql = self.format_where();
            sql.push_str(" WHERE ");
            sql.push_str(&where_sql);
        }

        sql.push(')');

        self.sql = Some(sql);

        self.sql.as_ref().expect("SQL not set")
    }
}
#[cfg(test)]
mod tests {
    use sqlformat::{FormatOptions, QueryParams};

    use crate::{
        testing::{TestTable, TestTableColumn},
        DynEncodeType, FilterExpr, FormatSqlQuery, SelectExists, TableType, WhereableTool,
    };

    #[test]
    pub fn someone_who_is_50() {
        let mut query = SelectExists::new(TestTable::table_name());
        query.filter(TestTableColumn::Age.equals(50.value()));

        let sql = query.format_sql_query();
        assert_eq!(
            sql,
            "SELECT EXISTS (SELECT 1 FROM test_table  WHERE test_table.age = $1)"
        );

        let sql = sqlformat::format(sql, &QueryParams::None, &FormatOptions::default());

        println!("{}", sql);
    }
}
