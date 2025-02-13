use std::fmt::Display;

use crate::{concat_columns_no_table_name, ColumnType, DynColumn};

#[derive(Debug, Clone)]
pub enum Returning<C: ColumnType> {
    None,
    All,
    Columns(Vec<C>),
}
impl<C> Returning<C>
where
    C: ColumnType + 'static,
{
    /// Converts the current type to a `Returning<DynColumn>`.
    pub fn dyn_column(self) -> Returning<DynColumn> {
        match self {
            Self::None => Returning::None,
            Self::All => Returning::All,
            Self::Columns(columns) => {
                Returning::Columns(columns.into_iter().map(|c| c.dyn_column()).collect())
            }
        }
    }
}
impl<C: ColumnType> Display for Returning<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, ""),
            Self::All => write!(f, " RETURNING *"),
            Self::Columns(columns) => {
                let columns = concat_columns_no_table_name(columns);
                write!(f, " RETURNING {}", columns)
            }
        }
    }
}
impl<C: ColumnType> Default for Returning<C> {
    fn default() -> Self {
        Self::None
    }
}
