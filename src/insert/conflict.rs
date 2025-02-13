use std::borrow::Cow;

use crate::{ColumnType, DynColumn, FormatSql};

#[derive(Debug, Clone)]
pub enum SetColumm<C> {
    /// Does SET `column = excluded.column`
    SetExcluded(C),
}
impl<C: ColumnType> From<C> for SetColumm<C> {
    fn from(column: C) -> Self {
        Self::SetExcluded(column)
    }
}
impl<C: ColumnType> FormatSql for SetColumm<C> {
    fn format_sql(&self) -> Cow<'_, str> {
        match self {
            Self::SetExcluded(column) => format!(
                "{column_name} = EXCLUDED.{column_name}",
                column_name = column.column_name()
            )
            .into(),
        }
    }
}
#[derive(Debug, Clone)]
pub enum OnConflictAction<C: ColumnType> {
    DoNothing,
    DoUpdate(Vec<SetColumm<C>>),
}
impl<C> OnConflictAction<C>
where
    C: ColumnType + 'static,
{
    pub(crate) fn dyn_column(self) -> OnConflictAction<DynColumn> {
        match self {
            Self::DoNothing => OnConflictAction::DoNothing,
            Self::DoUpdate(columns) => {
                let columns = columns
                    .into_iter()
                    .map(|column| match column {
                        SetColumm::SetExcluded(column) => {
                            SetColumm::SetExcluded(column.dyn_column())
                        }
                    })
                    .collect();

                OnConflictAction::DoUpdate(columns)
            }
        }
    }
}
impl<C: ColumnType> OnConflictAction<C> {
    pub fn update(columns: Vec<C>) -> Self {
        Self::DoUpdate(columns.into_iter().map(SetColumm::SetExcluded).collect())
    }
}

impl<C: ColumnType> FormatSql for OnConflictAction<C> {
    fn format_sql(&self) -> Cow<'_, str> {
        match self {
            Self::DoNothing => "DO NOTHING".into(),
            Self::DoUpdate(columns) => {
                let columns = columns
                    .iter()
                    .map(|column| column.format_sql())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("DO UPDATE SET {}", columns).into()
            }
        }
    }
}
#[derive(Debug, Clone)]
pub struct OnConflict<C: ColumnType> {
    pub columns: Vec<C>,
    pub action: OnConflictAction<C>,
}
impl<C> OnConflict<C>
where
    C: ColumnType + 'static,
{
    pub(crate) fn dyn_column(self) -> OnConflict<DynColumn> {
        let columns = self
            .columns
            .into_iter()
            .map(|column| column.dyn_column())
            .collect();
        let action = self.action.dyn_column();
        OnConflict { columns, action }
    }
}
impl<C: ColumnType> FormatSql for OnConflict<C> {
    fn format_sql(&self) -> Cow<'_, str> {
        let columns = self
            .columns
            .iter()
            .map(|column| column.column_name())
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "ON CONFLICT ({columns}) {action}",
            columns = columns,
            action = self.action.format_sql()
        )
        .into()
    }
}
impl<C> FormatSql for Option<OnConflict<C>>
where
    C: ColumnType,
{
    fn format_sql(&self) -> Cow<'_, str> {
        match self {
            Some(on_conflict) => on_conflict.format_sql(),
            None => Cow::Borrowed(""),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{testing::TestTableColumn, FormatSql, OnConflict, OnConflictAction, SetColumm};

    #[test]
    fn format_do_nothing() {
        let on_conflict = OnConflict {
            columns: vec![TestTableColumn::FirstName],
            action: OnConflictAction::DoNothing,
        };

        assert_eq!(
            on_conflict.format_sql(),
            "ON CONFLICT (first_name) DO NOTHING"
        );
    }

    #[test]
    fn format_set_columns() {
        let on_conflict = OnConflict {
            columns: vec![TestTableColumn::Email],
            action: OnConflictAction::DoUpdate(vec![
                SetColumm::SetExcluded(TestTableColumn::FirstName),
                SetColumm::SetExcluded(TestTableColumn::Email),
            ]),
        };

        assert_eq!(
            on_conflict.format_sql(),
            "ON CONFLICT (email) DO UPDATE SET first_name = EXCLUDED.first_name, email = EXCLUDED.email"
        );
    }
}
