use crate::{
    ColumnType, prelude::DynColumn, table_layout::concat_columns_no_table_name, traits::FormatSql,
};

#[derive(Debug)]
pub enum Returning {
    /// Return Wildcard
    ///
    /// `RETURNING *`
    All,
    /// Return specific columns
    ///
    /// `RETURNING column1, column2`
    Columns(Vec<DynColumn>),
}
impl FormatSql for Returning {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        match self {
            Self::All => "RETURNING *".into(),
            Self::Columns(columns) => {
                let columns = concat_columns_no_table_name(columns);
                format!("RETURNING {}", columns).into()
            }
        }
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
