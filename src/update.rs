use sqlx::{Database, Postgres};
use tracing::{debug, instrument};

use crate::{FormatWhere, SQLCondition};

use super::{
    ColumnType, Expr, ExprType, FormatSql, FormatSqlQuery, HasArguments, QueryTool, WhereableTool,
};

pub struct UpdateQueryBuilder<'table, 'args, C: ColumnType> {
    table: &'table str,
    columns_to_update: Vec<(C, Expr)>,
    where_comparisons: Vec<SQLCondition>,
    sql: Option<String>,
    arguments: Option<<Postgres as Database>::Arguments<'args>>,
}

impl<'args, C: ColumnType> HasArguments<'args> for UpdateQueryBuilder<'_, 'args, C> {
    fn take_arguments_or_error(&mut self) -> <Postgres as Database>::Arguments<'args> {
        self.arguments.take().expect("Arguments already taken")
    }

    fn borrow_arguments_or_error(&mut self) -> &mut <Postgres as Database>::Arguments<'args> {
        self.arguments.as_mut().expect("Arguments already taken")
    }
}

impl<'args, C: ColumnType> WhereableTool<'args> for UpdateQueryBuilder<'_, 'args, C> {
    fn push_where_comparison(&mut self, comparison: crate::SQLCondition) {
        self.where_comparisons.push(comparison);
    }
}
impl<'args, C: ColumnType> FormatWhere for UpdateQueryBuilder<'_, 'args, C> {
    fn get_conditions(&self) -> &[SQLCondition] {
        &self.where_comparisons
    }
}
impl<'args, C: ColumnType> FormatSqlQuery for UpdateQueryBuilder<'_, 'args, C> {
    #[instrument(skip(self), fields(table = %self.table, statement.type = "UPDATE"))]
    fn format_sql_query(&mut self) -> &str {
        let mut sql = format!("UPDATE {} SET ", self.table);

        let columns_to_update = self
            .columns_to_update
            .iter()
            .map(|(column, value)| format!("{} = {}", column.column_name(), value.format_sql()))
            .collect::<Vec<_>>()
            .join(", ");

        sql.push_str(&columns_to_update);

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

impl<'args, C: ColumnType> QueryTool<'args> for UpdateQueryBuilder<'_, 'args, C> {}

impl<'table, 'args, C> UpdateQueryBuilder<'table, 'args, C>
where
    C: ColumnType,
{
    pub fn new(table: &'table str) -> Self {
        Self {
            table,
            columns_to_update: Vec::new(),
            where_comparisons: Vec::new(),
            sql: None,
            arguments: Some(Default::default()),
        }
    }
    pub fn set<V>(&mut self, column: C, value: V) -> &mut Self
    where
        V: ExprType<'args> + 'args,
    {
        let value = value.process_unboxed(self);
        self.columns_to_update.push((column, value));
        self
    }
}

#[cfg(test)]
mod tests {
    use sqlformat::{FormatOptions, QueryParams};

    use crate::{
        testing::{AnotherTable, AnotherTableColumn, TestTable, TestTableColumn},
        DynEncodeType, ExprFunctionBuilder, ExprType, ExpressionWhereable, FormatSqlQuery,
        SelectExprBuilder, TableType, UpdateQueryBuilder, WhereableTool,
    };

    #[test]
    pub fn test_builder() {
        let mut query = UpdateQueryBuilder::new(TestTable::table_name());
        query.filter(TestTableColumn::Id.equals(1.value()));
        query
            .set(TestTableColumn::Age, 19.value())
            .set(TestTableColumn::Email, "test_ref_value@kingtux.dev".value())
            .set(
                TestTableColumn::Phone,
                SelectExprBuilder::new(AnotherTable::table_name())
                    .column(AnotherTableColumn::Phone)
                    .filter(AnotherTableColumn::Id.equals(1.value())),
            )
            .set(TestTableColumn::UpdatedAt, ExprFunctionBuilder::now());
        let sql = query.format_sql_query();

        let sql = sqlformat::format(sql, &QueryParams::None, &FormatOptions::default());

        println!("{}", sql);
    }
}
