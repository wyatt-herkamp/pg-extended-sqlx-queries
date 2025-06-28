use std::{borrow::Cow, fmt::Debug};

use tracing::{debug, instrument};

use crate::prelude::*;

pub struct UpdateQueryBuilder<'args> {
    table: Cow<'args, str>,
    columns_to_update: Vec<(DynColumn, Expr)>,
    where_comparisons: Vec<SQLCondition>,
    sql: Option<String>,
    returning: Option<Returning>,
    arguments: ArgumentHolder<'args>,
}

impl<'args> HasArguments<'args> for UpdateQueryBuilder<'args> {
    fn holder(&mut self) -> &mut ArgumentHolder<'args> {
        &mut self.arguments
    }
}

impl<'args> WhereableTool<'args> for UpdateQueryBuilder<'args> {
    fn push_where_comparison(&mut self, comparison: SQLCondition) {
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
            returning = SpaceBefore::from(self.returning.as_ref())
        );
        debug!(?sql, "UpdateQueryBuilder::format_sql_query");
        self.sql = Some(sql);

        self.sql.as_ref().expect("SQL not set")
    }
}

impl<'args> QueryTool<'args> for UpdateQueryBuilder<'args> {}
impl<'args> Debug for UpdateQueryBuilder<'args> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpdateQueryBuilder")
            .field("table", &self.table)
            .field("columns_to_update", &self.columns_to_update)
            .field("where_comparisons", &self.where_comparisons)
            .field("sql", &self.sql)
            .field("returning", &self.returning)
            .finish()
    }
}
impl<'args> UpdateQueryBuilder<'args> {
    pub fn new(table: &'args str) -> Self {
        Self {
            table: Cow::Borrowed(table),
            columns_to_update: Vec::new(),
            where_comparisons: Vec::new(),
            sql: None,
            returning: None,
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

    /// Sets the column to NULL
    ///
    /// Shortcut for `set(column, SqlNull)`
    pub fn set_null<C>(&mut self, column: C) -> &mut Self
    where
        C: ColumnType + 'static,
    {
        self.columns_to_update
            .push((column.dyn_column(), Keywords::Null.into()));
        self
    }
}
impl SupportsReturning for UpdateQueryBuilder<'_> {
    fn returning(&mut self, returning: Returning) -> &mut Self {
        self.returning = Some(returning);
        self
    }
}
#[cfg(test)]
mod tests {
    use sqlformat::{FormatOptions, QueryParams};

    use crate::{
        prelude::*,
        testing::{AnotherTable, AnotherTableColumn, TestTable, TestTableColumn},
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
            .set(TestTableColumn::UpdatedAt, SqlFunctionBuilder::now());
        let sql = query.format_sql_query();
        assert_eq!(
            sql,
            "UPDATE test_table SET age = $2, email = $3, phone = (SELECT another_table.phone FROM another_table WHERE another_table.id = $4), updated_at = NOW() WHERE test_table.id = $1;"
        );

        let sql = sqlformat::format(sql, &QueryParams::None, &FormatOptions::default());

        println!("{sql}");
    }

    #[test]
    pub fn test_builder_with_return() {
        let mut query = UpdateQueryBuilder::new(TestTable::table_name());
        query.filter(TestTableColumn::Id.equals(1));
        query
            .set(TestTableColumn::Age, 19)
            .set(TestTableColumn::Email, "test_ref_value@kingtux.dev")
            .set(TestTableColumn::UpdatedAt, SqlFunctionBuilder::now())
            .return_all();
        let sql = query.format_sql_query();
        assert_eq!(
            sql,
            "UPDATE test_table SET age = $2, email = $3, updated_at = NOW() WHERE test_table.id = $1 RETURNING *;"
        );
        let sql = sqlformat::format(sql, &QueryParams::None, &FormatOptions::default());

        println!("{sql}");
    }

    #[test]
    pub fn test_builder_set_null() {
        let mut query = UpdateQueryBuilder::new(TestTable::table_name());
        query.filter(TestTableColumn::Id.equals(1));
        query
            .set(TestTableColumn::Age, SqlNull)
            .set(TestTableColumn::Email, "test_ref_value@kingtux.dev")
            .set(TestTableColumn::UpdatedAt, SqlFunctionBuilder::now())
            .return_all();
        let sql = query.format_sql_query();
        assert_eq!(
            sql,
            "UPDATE test_table SET age = NULL, email = $2, updated_at = NOW() WHERE test_table.id = $1 RETURNING *;"
        );
        let sql = sqlformat::format(sql, &QueryParams::None, &FormatOptions::default());

        println!("{sql}");
    }
}
