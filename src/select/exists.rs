use std::fmt::Debug;

use crate::prelude::*;

pub struct SelectExists<'args> {
    table: &'static str,
    where_comparisons: Vec<SQLCondition>,
    sql: Option<String>,
    arguments: ArgumentHolder<'args>,
}
impl<'args> WhereableTool<'args> for SelectExists<'args> {
    fn push_where_comparison(&mut self, comparison: SQLCondition) {
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
            arguments: Default::default(),
        }
    }
}
impl<'args> HasArguments<'args> for SelectExists<'args> {
    fn holder(&mut self) -> &mut ArgumentHolder<'args> {
        &mut self.arguments
    }
}
impl<'args> Debug for SelectExists<'args> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectExists")
            .field("table", &self.table)
            .field("where_comparisons", &self.where_comparisons)
            .field("sql", &self.sql)
            .finish()
    }
}
impl<'args> QueryTool<'args> for SelectExists<'args> {}
impl<'args> QueryScalarTool<'args> for SelectExists<'args> {
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
    use crate::prelude::*;
    use crate::testing::{TestTable, TestTableColumn};
    use sqlformat::{FormatOptions, QueryParams};

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
