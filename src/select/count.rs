use std::fmt::Debug;

use crate::prelude::*;

/// Counts the number of rows in a table based on the given where comparisons.
pub struct SelectCount<'args> {
    table: &'static str,
    where_comparisons: Vec<SQLCondition>,
    sql: Option<String>,
    arguments: ArgumentHolder<'args>,
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
            arguments: Default::default(),
        }
    }
}
impl<'args> HasArguments<'args> for SelectCount<'args> {
    fn holder(&mut self) -> &mut ArgumentHolder<'args> {
        &mut self.arguments
    }
}
impl<'args> Debug for SelectCount<'args> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectCount")
            .field("table", &self.table)
            .field("where_comparisons", &self.where_comparisons)
            .field("sql", &self.sql)
            .finish()
    }
}

impl<'args> QueryTool<'args> for SelectCount<'args> {}
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
impl<'args> QueryScalarTool<'args> for SelectCount<'args> {
    type Output = i64;
}

#[cfg(test)]
mod tests {
    use sqlformat::{FormatOptions, QueryParams};

    use crate::prelude::*;
    use crate::testing::{TestTable, TestTableColumn};

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

        println!("{sql}");
    }
}
