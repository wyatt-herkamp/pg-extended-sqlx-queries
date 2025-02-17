use std::borrow::Cow;

use crate::arguments::{ArgumentHolder, HasArguments};
use tracing::{debug, instrument};

use crate::{DynColumn, FormatWhere, FormatWhereItem, Returning, SQLCondition, SupportsReturning};

use super::{ColumnType, Expr, ExprType, FormatSql, FormatSqlQuery, QueryTool, WhereableTool};

pub struct UpdateQueryBuilder<'args> {
    table: Cow<'args, str>,
    columns_to_update: Vec<(DynColumn, Expr)>,
    where_comparisons: Vec<SQLCondition>,
    sql: Option<String>,
    returning: Returning,
    arguments: ArgumentHolder<'args>,
}

impl<'args> HasArguments<'args> for UpdateQueryBuilder<'args> {
    fn holder(&mut self) -> &mut ArgumentHolder<'args> {
        &mut self.arguments
    }
}

impl<'args> WhereableTool<'args> for UpdateQueryBuilder<'args> {
    fn push_where_comparison(&mut self, comparison: crate::SQLCondition) {
        self.where_comparisons.push(comparison);
    }
}
impl<'args> FormatWhere for UpdateQueryBuilder<'args> {
    fn get_conditions(&self) -> &[SQLCondition] {
        &self.where_comparisons
    }
}
impl<'args> FormatSqlQuery for UpdateQueryBuilder<'args> {
    #[instrument(skip(self), fields(table = %self.table, statement.type = "UPDATE"))]
    fn format_sql_query(&mut self) -> &str {
        let columns_to_update = self
            .columns_to_update
            .iter()
            .map(|(column, value)| format!("{} = {}", column.column_name(), value.format_sql()))
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!(
            "UPDATE {table} SET {columns_to_update}{filter}{returning};",
            table = self.table,
            filter = FormatWhereItem(self),
            returning = self.returning
        );
        debug!(?sql, "UpdateQueryBuilder::format_sql_query");
        self.sql = Some(sql);

        self.sql.as_ref().expect("SQL not set")
    }
}

impl<'args> QueryTool<'args> for UpdateQueryBuilder<'args> {}

impl<'args> UpdateQueryBuilder<'args> {
    pub fn new(table: &'args str) -> Self {
        Self {
            table: Cow::Borrowed(table),
            columns_to_update: Vec::new(),
            where_comparisons: Vec::new(),
            sql: None,
            returning: Returning::None,
            arguments: Default::default(),
        }
    }
    pub fn set<C, V>(&mut self, column: C, value: V) -> &mut Self
    where
        C: ColumnType + 'static,
        V: ExprType<'args> + 'args,
    {
        let value = value.process_unboxed(&mut self.arguments);
        self.columns_to_update.push((column.dyn_column(), value));
        self
    }
}
impl SupportsReturning for UpdateQueryBuilder<'_> {
    fn returning(&mut self, returning: Returning) -> &mut Self {
        self.returning = returning;
        self
    }
}
#[cfg(test)]
mod tests {
    use sqlformat::{FormatOptions, QueryParams};

    use crate::{
        testing::{AnotherTable, AnotherTableColumn, TestTable, TestTableColumn},
        DynEncodeType, ExprFunctionBuilder, ExpressionWhereable, FilterExpr, FormatSqlQuery,
        SelectExprBuilder, SupportsReturning, TableType, UpdateQueryBuilder, WhereableTool,
    };

    #[test]
    pub fn test_builder() {
        let mut query = UpdateQueryBuilder::new(TestTable::table_name());
        query.filter(TestTableColumn::Id.equals(1.value()));
        query
            .set(TestTableColumn::Age, 19.value())
            .set(TestTableColumn::Email, "test_ref_value@kingtux.dev")
            .set(
                TestTableColumn::Phone,
                SelectExprBuilder::new(AnotherTable::table_name())
                    .column(AnotherTableColumn::Phone)
                    .filter(AnotherTableColumn::Id.equals(1.value())),
            )
            .set(TestTableColumn::UpdatedAt, ExprFunctionBuilder::now());
        let sql = query.format_sql_query();
        assert_eq!(sql, "UPDATE test_table SET age = $2, email = $3, phone = (SELECT another_table.phone FROM another_table WHERE another_table.id = $4), updated_at = NOW() WHERE test_table.id = $1;");

        let sql = sqlformat::format(sql, &QueryParams::None, &FormatOptions::default());

        println!("{}", sql);
    }

    #[test]
    pub fn test_builder_with_return() {
        let mut query = UpdateQueryBuilder::new(TestTable::table_name());
        query.filter(TestTableColumn::Id.equals(1));
        query
            .set(TestTableColumn::Age, 19)
            .set(TestTableColumn::Email, "test_ref_value@kingtux.dev")
            .set(TestTableColumn::UpdatedAt, ExprFunctionBuilder::now())
            .return_all();
        let sql = query.format_sql_query();
        assert_eq!(sql, "UPDATE test_table SET age = $2, email = $3, updated_at = NOW() WHERE test_table.id = $1 RETURNING *;");
        let sql = sqlformat::format(sql, &QueryParams::None, &FormatOptions::default());

        println!("{}", sql);
    }
}
