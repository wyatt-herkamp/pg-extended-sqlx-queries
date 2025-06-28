use std::fmt::Debug;

use crate::{
    expr::{ArgumentHolder, ConflictQuery, HasArguments, OnConflict, Returning, SupportsReturning},
    prelude::ColumnType,
    table_layout::concat_columns_no_table_name,
    traits::{FormatSql, FormatSqlQuery, QueryTool, SpaceBefore},
};

mod row;
pub use row::*;
use tracing::{debug, instrument};

pub struct InsertManyBuilder<'args, C: ColumnType> {
    columns_to_insert: Vec<C>,
    sql: Option<String>,
    returning: Option<Returning>,
    rows: Vec<InsertRow<C>>,
    table: &'static str,
    arguments: ArgumentHolder<'args>,
    on_conflict: Option<OnConflict>,
}
impl<'args, C: ColumnType> ConflictQuery<'args> for InsertManyBuilder<'args, C> {
    fn set_on_conflict(&mut self, on_conflict: OnConflict) -> &mut Self {
        self.on_conflict = Some(on_conflict);
        self
    }
}
impl<'args, C> HasArguments<'args> for InsertManyBuilder<'args, C>
where
    C: ColumnType,
{
    fn holder(&mut self) -> &mut ArgumentHolder<'args> {
        &mut self.arguments
    }
}

impl<C> Debug for InsertManyBuilder<'_, C>
where
    C: ColumnType + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InsertManyBuilder")
            .field("columns_to_insert", &self.columns_to_insert)
            .field("sql", &self.sql)
            .field("returning", &self.returning)
            .field("table", &self.table)
            .finish()
    }
}
impl<'args, C: ColumnType> InsertManyBuilder<'args, C> {
    pub fn new(table: &'static str, columns: impl Into<Vec<C>>) -> Self {
        Self {
            table,
            arguments: Default::default(),
            columns_to_insert: columns.into(),
            sql: None,
            rows: Vec::new(),
            returning: Default::default(),
            on_conflict: None,
        }
    }

    /// Insert a value into the query
    pub fn insert_row<F>(&mut self, insert_row: F) -> &mut Self
    where
        F: FnOnce(&mut InsertRowBuilder<'_, 'args, C>),
        C: ColumnType + PartialEq + Clone,
    {
        self.sql = None;
        let mut builder = InsertRowBuilder::new(self);
        insert_row(&mut builder);

        let row = builder.finish();

        self.rows.push(row);

        self
    }
    /// Instead of having to specify the columns that you want to insert it is done based on order
    pub fn insert_row_ordered<F>(&mut self, insert_row: F) -> &mut Self
    where
        F: FnOnce(&mut InsertRowOrderedBuilder<'_, 'args, C>),
        C: ColumnType + PartialEq + Clone,
    {
        self.sql = None;
        let mut builder = InsertRowOrderedBuilder::new(self);
        insert_row(&mut builder);

        let row = builder.finish();

        self.rows.push(row);

        self
    }
}
impl<C: ColumnType> SupportsReturning for InsertManyBuilder<'_, C> {
    fn returning(&mut self, returning: Returning) -> &mut Self {
        self.returning = Some(returning);
        self
    }
}
impl<'args, C> QueryTool<'args> for InsertManyBuilder<'args, C> where C: ColumnType {}
impl<C: ColumnType> FormatSqlQuery for InsertManyBuilder<'_, C> {
    #[instrument(skip(self), fields(table = %self.table, statement.type = "INSERT"))]
    fn format_sql_query(&mut self) -> &str {
        let columns = concat_columns_no_table_name(&self.columns_to_insert);
        let placeholders = self
            .rows
            .iter()
            .map(|row| row.format_sql())
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "INSERT INTO {table} ({columns}) VALUES {placeholders}{on_conflict}{returning};",
            table = self.table,
            on_conflict = self.on_conflict.format_sql(),
            returning = SpaceBefore::from(self.returning.as_ref()),
        );
        debug!(?sql, "InsertManyBuilder::gen_sql");

        self.sql = Some(sql);
        self.sql.as_ref().expect("BUG: SQL not generated")
    }
}

#[cfg(test)]
mod tests {
    use sqlformat::QueryParams;

    use crate::{
        prelude::*,
        testing::{TestTable, TestTableColumn},
    };

    #[test]
    fn test_insert_many() {
        let mut query =
            super::InsertManyBuilder::new(TestTable::table_name(), TestTable::columns());
        query
            .insert_row(|row| {
                row.insert(TestTableColumn::FirstName, "value1".value());
                row.insert(TestTableColumn::LastName, "value2".value());
            })
            .insert_row(|row| {
                row.insert(TestTableColumn::FirstName, "value1".value());
                row.insert(TestTableColumn::LastName, "value2".value());
            })
            .return_all();
        assert_eq!(
            query.format_sql_query(),
            "INSERT INTO test_table (id, first_name, last_name, age, email, phone, another_table_id, updated_at, created_at) VALUES (DEFAULT, $1, $2, DEFAULT, DEFAULT, DEFAULT, DEFAULT, DEFAULT, DEFAULT), (DEFAULT, $3, $4, DEFAULT, DEFAULT, DEFAULT, DEFAULT, DEFAULT, DEFAULT) RETURNING *;"
        );

        let formatted = sqlformat::format(
            query.format_sql_query(),
            &QueryParams::default(),
            &Default::default(),
        );
        println!("{formatted}");
    }
}
