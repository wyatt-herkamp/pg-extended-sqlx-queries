use std::fmt::Debug;

use crate::{
    expr::{
        ArgumentHolder, ConflictQuery, Expr, ExprType, HasArguments, OnConflict, Returning,
        SupportsReturning,
    },
    prelude::{ColumnType, DynColumn},
    table_layout::concat_columns_no_table_name,
    traits::{FormatSql, FormatSqlQuery, QueryTool, SpaceBefore},
};
use tracing::{debug, instrument};
pub mod many;
pub struct InsertQueryBuilder<'args> {
    columns: Vec<DynColumn>,
    insert: Vec<Expr>,
    sql: Option<String>,
    returning: Option<Returning>,
    table: &'static str,
    on_conflict: Option<OnConflict>,
    arguments: ArgumentHolder<'args>,
}

impl Debug for InsertQueryBuilder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleInsertQueryBuilder")
            .field("columns_to_insert", &self.insert)
            .field("sql", &self.sql)
            .field("returning", &self.returning)
            .field("table", &self.table)
            .finish()
    }
}
impl<'args> ConflictQuery<'args> for InsertQueryBuilder<'args> {
    fn set_on_conflict(&mut self, on_conflict: OnConflict) -> &mut Self {
        self.on_conflict = Some(on_conflict);
        self
    }
}
impl<'args> InsertQueryBuilder<'args> {
    pub fn new(table: &'static str) -> Self {
        Self {
            table,
            arguments: Default::default(),
            columns: Vec::new(),
            insert: Vec::new(),
            sql: None,
            on_conflict: None,
            returning: Default::default(),
        }
    }

    /// Insert a value into the query
    pub fn insert<C, E>(&mut self, column: C, value: E) -> &mut Self
    where
        C: ColumnType + 'static,
        E: ExprType<'args> + 'args,
    {
        self.sql = None;
        self.columns.push(column.dyn_column());
        let expr = value.process_unboxed(&mut self.arguments);
        self.insert.push(expr);
        self
    }
    /// Will check if option is Some and insert the value if it is
    ///
    /// This will allow for the database to just use its default value if the option is None
    ///
    /// If you want to insert a NULL value use `insert` with `None`
    pub fn insert_option<C, E>(&mut self, column: C, value: Option<E>) -> &mut Self
    where
        C: ColumnType + 'static,
        E: ExprType<'args> + 'args,
    {
        if let Some(value) = value {
            self.insert(column, value)
        } else {
            self
        }
    }
}
impl<'args> HasArguments<'args> for InsertQueryBuilder<'args> {
    fn holder(&mut self) -> &mut ArgumentHolder<'args> {
        &mut self.arguments
    }
}
impl<'args> QueryTool<'args> for InsertQueryBuilder<'args> {}
impl SupportsReturning for InsertQueryBuilder<'_> {
    fn returning(&mut self, returning: Returning) -> &mut Self {
        self.returning = Some(returning);
        self
    }
}
impl FormatSqlQuery for InsertQueryBuilder<'_> {
    #[instrument(skip(self), fields(table = %self.table, statement.type = "INSERT"))]
    fn format_sql_query(&mut self) -> &str {
        let columns = concat_columns_no_table_name(&self.columns);
        let values = self
            .insert
            .iter()
            .map(|expr| expr.format_sql())
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "INSERT INTO {table} ({columns}) VALUES ({values}){on_conflict}{returning};",
            table = self.table,
            on_conflict = self.on_conflict.format_sql(),
            returning = SpaceBefore::from(self.returning.as_ref()),
        );
        debug!(?sql, "InsertQueryBuilder::gen_sql");
        self.sql = Some(sql);
        self.sql.as_ref().expect("BUG: SQL not generated")
    }
}
pub fn generate_placeholder_string(len: usize) -> String {
    (0..len)
        .map(|index| format!("${}", index + 1))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use sqlformat::QueryParams;

    use crate::{
        prelude::*,
        testing::{AnotherTable, AnotherTableColumn, TestTable, TestTableColumn},
    };

    #[test]
    pub fn test_no_return() {
        let mut builder = super::InsertQueryBuilder::new(TestTable::table_name());
        builder
            .insert(TestTableColumn::LastName, "Doe".value())
            .insert(TestTableColumn::FirstName, "John".value())
            .insert_option(
                TestTableColumn::Phone,
                Some("123-456-7890".to_string().value()),
            )
            .insert_option(TestTableColumn::Email, Option::<DynEncode>::None);

        let sql = builder.format_sql_query();
        assert_eq!(
            sql,
            "INSERT INTO test_table (last_name, first_name, phone) VALUES ($1, $2, $3);"
        );
        println!("{sql}");
    }
    #[test]
    pub fn insert_with_expr() {
        let mut builder = super::InsertQueryBuilder::new(TestTable::table_name());
        builder
            .insert(TestTableColumn::LastName, "Doe".value())
            .insert(TestTableColumn::FirstName, "John".value())
            .insert(TestTableColumn::CreatedAt, SqlFunctionBuilder::now());

        let sql = builder.format_sql_query();
        assert_eq!(
            sql,
            "INSERT INTO test_table (last_name, first_name, created_at) VALUES ($1, $2, NOW());"
        );
        println!("{sql}");
    }
    #[test]
    pub fn insert_sub_query() {
        let mut builder = super::InsertQueryBuilder::new(TestTable::table_name());
        builder
            .insert(TestTableColumn::LastName, "Doe".value())
            .insert(TestTableColumn::FirstName, "John".value())
            .insert(
                TestTableColumn::Phone,
                SelectExprBuilder::new(AnotherTable::table_name())
                    .column(AnotherTableColumn::Phone)
                    .filter(AnotherTableColumn::Id.equals(1.value())),
            );

        let sql = builder.format_sql_query();
        assert_eq!(
            sql,
            "INSERT INTO test_table (last_name, first_name, phone) VALUES ($1, $2, (SELECT another_table.phone FROM another_table WHERE another_table.id = $3));"
        );
        let formatted = sqlformat::format(sql, &QueryParams::default(), &Default::default());
        println!("{formatted}");
    }

    #[test]
    pub fn on_conflict_do_nothing() {
        let mut builder = super::InsertQueryBuilder::new(TestTable::table_name());
        builder
            .insert(TestTableColumn::LastName, "Doe".value())
            .insert(TestTableColumn::FirstName, "John".value())
            .on_conflict_do_nothing(ConflictTarget::columns(vec![TestTableColumn::LastName]));

        let sql = builder.format_sql_query();
        assert_eq!(
            sql,
            "INSERT INTO test_table (last_name, first_name) VALUES ($1, $2) ON CONFLICT (last_name) DO NOTHING;"
        );
        println!("{sql}");
    }

    #[test]
    pub fn on_conflict_update() {
        let mut builder = super::InsertQueryBuilder::new(TestTable::table_name());
        builder
            .insert(TestTableColumn::LastName, "Doe".value())
            .insert(TestTableColumn::FirstName, "John".value())
            .on_conflict_set_excluded(
                ConflictTarget::columns(vec![TestTableColumn::LastName]),
                vec![TestTableColumn::FirstName, TestTableColumn::Phone],
            );

        let sql = builder.format_sql_query();
        assert_eq!(
            sql,
            "INSERT INTO test_table (last_name, first_name) VALUES ($1, $2) ON CONFLICT (last_name) DO UPDATE SET first_name = EXCLUDED.first_name, phone = EXCLUDED.phone;"
        );
        println!("{sql}");
    }
}
