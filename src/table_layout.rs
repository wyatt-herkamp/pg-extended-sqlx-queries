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
pub trait HasPrimaryKey: TableType {
    fn primary_key() -> Self::Columns
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

    fn primary_key() -> <Self::Table as TableType>::Columns
    where
        Self::Table: HasPrimaryKey,
    {
        <Self::Table as HasPrimaryKey>::primary_key()
    }
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

pub trait Relation<T: TableType>: TableType {
    fn from_column() -> Self::Columns
    where
        Self: Sized;

    fn to_column() -> T::Columns
    where
        Self: Sized;
}
