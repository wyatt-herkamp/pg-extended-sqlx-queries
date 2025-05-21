use std::{borrow::Cow, fmt::Debug};

use crate::expr::{Aliasable, ArgumentHolder, CastableExprType, Expr, ExprType, WrapInFunction};

#[derive(Debug)]
pub struct DynColumn(Box<dyn ColumnType + Send + Sync>);
impl DynColumn {
    pub fn new<C>(column: C) -> Self
    where
        C: ColumnType + Send + Sync + 'static,
    {
        Self(Box::new(column))
    }
}
impl ColumnType for DynColumn {
    fn column_name(&self) -> &'static str {
        self.0.column_name()
    }
    fn table_name(&self) -> &'static str {
        self.0.table_name()
    }
    fn dyn_column(self) -> DynColumn
    where
        Self: Sized + Send + Sync + 'static,
    {
        self
    }

    fn format_column_with_prefix(&self, prefix: Option<&str>) -> Cow<'static, str> {
        self.0.format_column_with_prefix(prefix)
    }
    fn full_name(&self) -> Cow<'static, str> {
        self.0.full_name()
    }
}
impl<C> PartialEq<C> for DynColumn
where
    C: ColumnType,
{
    fn eq(&self, other: &C) -> bool {
        self.column_name() == other.column_name() && self.table_name() == other.table_name()
    }
}
impl<'args> ExprType<'args> for DynColumn {
    fn process(self: Box<Self>, _args: &mut ArgumentHolder<'args>) -> Expr {
        Expr::Column(*self)
    }

    fn process_unboxed(self, _args: &mut ArgumentHolder<'args>) -> Expr {
        Expr::Column(self)
    }
}

pub trait ColumnType: Debug + Send + Sync {
    /// Returns the column name
    fn column_name(&self) -> &'static str;
    /// Returns the table name of the column
    fn table_name(&self) -> &'static str;
    /// Should return the `{table_name}.{column_name}` format
    fn full_name(&self) -> Cow<'static, str> {
        Cow::Owned(format!("{}.{}", self.table_name(), self.column_name()))
    }
    /// Should return the `{prefix}.{column_name}` format
    fn format_column_with_prefix(&self, prefix: Option<&str>) -> Cow<'static, str> {
        if let Some(prefix) = prefix {
            Cow::Owned(format!("{}.{}", prefix, self.column_name()))
        } else {
            Cow::Borrowed(self.column_name())
        }
    }

    fn dyn_column(self) -> DynColumn
    where
        Self: Sized + Send + Sync + 'static,
    {
        DynColumn::new(self)
    }
}

pub trait AllColumns {
    fn all() -> Vec<Self>
    where
        Self: Sized;

    fn all_dyn() -> Vec<DynColumn>
    where
        Self: Sized + ColumnType + 'static,
    {
        Self::all().into_iter().map(|c| c.dyn_column()).collect()
    }
}
impl<'args, C> Aliasable<'args> for C where C: ColumnType + ExprType<'args> + 'static {}
impl<'args, C> WrapInFunction<'args> for C where C: ColumnType + ExprType<'args> + 'static {}
impl<'args, C> CastableExprType<'args> for C where C: ColumnType + ExprType<'args> + 'static {}

pub fn concat_columns<'column, I, C>(columns: I, prefix: Option<&str>) -> String
where
    I: IntoIterator<Item = &'column C>,
    C: ColumnType + 'column,
{
    if prefix.is_some() {
        columns
            .into_iter()
            .map(|column| column.format_column_with_prefix(prefix))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        columns
            .into_iter()
            .map(|column| column.full_name())
            .collect::<Vec<_>>()
            .join(", ")
    }
}
/// Why? Because returning columns won't allow table name
pub fn concat_columns_no_table_name<'column, I, C>(columns: I) -> String
where
    I: IntoIterator<Item = &'column C>,
    C: ColumnType + 'column,
{
    columns
        .into_iter()
        .map(|column| column.column_name())
        .collect::<Vec<_>>()
        .join(", ")
}
