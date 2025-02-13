mod expr;
mod insert;
mod pagination;
mod select;
mod table_layout;
mod traits;
mod update;
pub use expr::*;
pub use insert::*;
pub use pagination::*;
pub use select::*;
pub use table_layout::*;
pub use traits::*;
pub use update::*;
#[cfg(test)]
pub(crate) mod testing;

pub mod prelude {
    pub use crate::*;
    pub use pg_extended_sqlx_queries_macros::TableType;
}

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
            .map(|column| column.formatted_column())
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
