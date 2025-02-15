use sqlx::{Database, Postgres};
use tracing::{debug, instrument};

use crate::{FormatSqlQuery, FormatWhere, HasArguments, SQLCondition, WhereableTool};

pub struct DeleteQueryBuilder<'args> {
    table: &'static str,
    where_comparisons: Vec<SQLCondition>,
    sql: Option<String>,
    arguments: Option<<Postgres as Database>::Arguments<'args>>,
}
impl<'args> DeleteQueryBuilder<'args> {
    pub fn new(table: &'static str) -> Self {
        Self {
            table,
            where_comparisons: vec![],
            sql: None,
            arguments: Some(Default::default()),
        }
    }
}
impl<'args> HasArguments<'args> for DeleteQueryBuilder<'args> {
    fn take_arguments_or_error(&mut self) -> <Postgres as Database>::Arguments<'args> {
        self.arguments.take().expect("Arguments already taken")
    }

    fn borrow_arguments_or_error(&mut self) -> &mut <Postgres as Database>::Arguments<'args> {
        self.arguments.as_mut().expect("Arguments already taken")
    }
}

impl<'args> WhereableTool<'args> for DeleteQueryBuilder<'args> {
    fn push_where_comparison(&mut self, comparison: crate::SQLCondition) {
        self.where_comparisons.push(comparison);
    }
}
impl<'args> FormatWhere for DeleteQueryBuilder<'args> {
    fn get_conditions(&self) -> &[SQLCondition] {
        &self.where_comparisons
    }
}
impl<'args> FormatSqlQuery for DeleteQueryBuilder<'args> {
    #[instrument(skip(self), fields(table = %self.table, statement.type = "DELETE"))]
    fn format_sql_query(&mut self) -> &str {
        let mut sql = format!("DELETE FROM {}", self.table);

        if !self.where_comparisons.is_empty() {
            let where_sql = self.format_where();
            sql.push_str(" WHERE ");
            sql.push_str(&where_sql);
        }
        debug!(?sql, "UpdateQueryBuilder::format_sql_query");
        self.sql = Some(sql);

        self.sql.as_ref().expect("SQL not set")
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        testing::{TestTable, TestTableColumn},
        DynEncodeType, FilterExpr, TableType, WhereableTool,
    };

    #[test]
    fn test_delete_query_builder() {
        let mut delete_query_builder = DeleteQueryBuilder::new(TestTable::table_name());
        delete_query_builder.filter(TestTableColumn::Id.equals(1.value()));
        let sql = delete_query_builder.format_sql_query();
        assert_eq!(sql, "DELETE FROM test_table WHERE test_table.id = $1");
    }
}
