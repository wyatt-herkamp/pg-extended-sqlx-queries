use std::fmt::Display;

use crate::{prelude::DynColumn, table_layout::concat_columns_no_table_name, ColumnType};

#[derive(Debug)]
pub enum Returning {
    None,
    All,
    Columns(Vec<DynColumn>),
}
impl Display for Returning {
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
impl Default for Returning {
    fn default() -> Self {
        Self::None
    }
}

pub trait SupportsReturning {
    fn returning(&mut self, returning: Returning) -> &mut Self;
    fn return_all(&mut self) -> &mut Self {
        self.returning(Returning::All)
    }
    fn return_columns<C>(&mut self, columns: Vec<C>) -> &mut Self
    where
        C: ColumnType + 'static,
    {
        self.returning(Returning::Columns(
            columns.into_iter().map(|c| c.dyn_column()).collect(),
        ))
    }
}
