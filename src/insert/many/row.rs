use std::{borrow::Cow, fmt::Debug};

use crate::{ColumnType, Expr, ExprType, FormatSql, SqlDefault};

use super::InsertManyBuilder;

pub struct InsertRowBuilder<'query, 'args, C: ColumnType> {
    /// If None then it will use DEFAULT
    columns_to_insert: Vec<(C, Expr)>,
    query: &'query mut InsertManyBuilder<'args, C>,
}
impl<'query, 'args, C: ColumnType + PartialEq + Clone> InsertRowBuilder<'query, 'args, C> {
    pub(super) fn new(query: &'query mut InsertManyBuilder<'args, C>) -> Self {
        Self {
            columns_to_insert: Vec::with_capacity(query.columns_to_insert.len()),
            query,
        }
    }
    /// Insert a value into the query
    pub fn insert<E>(&mut self, column: C, value: E) -> &mut Self
    where
        E: ExprType<'args> + 'args,
    {
        let expr = value.process_unboxed(self.query);
        self.columns_to_insert.push((column, expr));
        self
    }

    /// Will check if option is Some and insert the value if it is
    ///
    /// This will allow for the database to just use its default value if the option is None
    ///
    /// If you want to insert a NULL value use `insert` with `None`
    pub fn insert_option<E>(&mut self, column: C, value: Option<E>) -> &mut Self
    where
        E: ExprType<'args> + 'args,
    {
        if let Some(value) = value {
            let expr = value.process_unboxed(self.query);

            self.columns_to_insert.push((column, expr));
        } else {
            self.columns_to_insert
                .push((column, SqlDefault::default().into()));
        }
        self
    }
    pub(super) fn finish(mut self) -> InsertRow<C> {
        let mut values = Vec::with_capacity(self.query.columns_to_insert.len());
        for column in self.query.columns_to_insert.iter() {
            let index_within_row = self.columns_to_insert.iter().position(|(c, _)| c == column);
            // Any values that do not have a value will be set to DEFAULT
            let value = index_within_row
                .map(|index| self.columns_to_insert.remove(index).1)
                .unwrap_or_else(|| SqlDefault::default().into());
            values.push((column.clone(), value));
        }
        InsertRow(values)
    }
}

pub struct InsertRow<C>(Vec<(C, Expr)>);
impl<C> Debug for InsertRow<C>
where
    C: ColumnType + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("InsertRow").field(&self.0).finish()
    }
}
impl<C: ColumnType> FormatSql for InsertRow<C> {
    fn format_sql(&self) -> Cow<'_, str> {
        let mut string_builder = "(".to_owned();
        let mut iter = self.0.iter().peekable();
        while let Some((_, value)) = iter.next() {
            string_builder.push_str(&value.format_sql());
            if iter.peek().is_some() {
                string_builder.push_str(", ");
            }
        }
        string_builder.push(')');
        Cow::Owned(string_builder)
    }
}

pub struct InsertRowOrderedBuilder<'query, 'args, C: ColumnType> {
    /// If None then it will use DEFAULT
    columns_to_insert: Vec<Expr>,
    query: &'query mut InsertManyBuilder<'args, C>,
}
impl<'query, 'args, C: ColumnType + PartialEq + Clone> InsertRowOrderedBuilder<'query, 'args, C> {
    pub(super) fn new(query: &'query mut InsertManyBuilder<'args, C>) -> Self {
        Self {
            columns_to_insert: Vec::with_capacity(query.columns_to_insert.len()),
            query,
        }
    }
    /// Insert a value into the query
    pub fn insert<E>(&mut self, value: E) -> &mut Self
    where
        E: ExprType<'args> + 'args,
    {
        let expr = value.process_unboxed(self.query);
        self.columns_to_insert.push(expr);
        self
    }

    /// Will check if option is Some and insert the value if it is
    ///
    /// This will allow for the database to just use its default value if the option is None
    ///
    /// If you want to insert a NULL value use `insert` with `None`
    pub fn insert_option<E>(&mut self, value: Option<E>) -> &mut Self
    where
        E: ExprType<'args> + 'args,
    {
        if let Some(value) = value {
            self.columns_to_insert
                .push(value.process_unboxed(self.query));
        } else {
            self.columns_to_insert.push(SqlDefault::default().into());
        }
        self
    }
    pub(super) fn finish(self) -> InsertRow<C> {
        let mut column_values = Vec::with_capacity(self.query.columns_to_insert.len());
        let mut values = self.columns_to_insert.into_iter();
        for column in self.query.columns_to_insert.iter() {
            let value = values.next().unwrap_or(SqlDefault::default().into());
            // Any values that do not have a value will be set to DEFAULT
            column_values.push((column.clone(), value));
        }
        InsertRow(column_values)
    }
}
