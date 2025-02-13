mod columns;
pub use columns::*;
/// The primary definition of a table.
pub trait TableType {
    /// The Column type for this table
    type Columns: ColumnType + 'static;
    /// The name of the table
    fn table_name() -> &'static str
    where
        Self: Sized;
}
/// Defines a specific query for a table
pub trait TableQuery {
    /// The Table type for this query
    type Table: TableType;
    /// Get all the columns for this specific query
    fn columns() -> Vec<<Self::Table as TableType>::Columns>
    where
        Self: Sized;
}

impl<T> TableQuery for T
where
    T: TableType,
    T::Columns: AllColumns,
{
    type Table = T;

    fn columns() -> Vec<<Self::Table as TableType>::Columns> {
        <Self::Table as TableType>::Columns::all()
    }
}
