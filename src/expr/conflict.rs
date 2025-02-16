use std::borrow::Cow;

use crate::{ColumnType, DynColumn, FormatSql};

#[derive(Debug)]
pub enum SetColumm {
    /// Does SET `column = excluded.column`
    SetExcluded(DynColumn),
}
impl<C> From<C> for SetColumm
where
    C: ColumnType + 'static,
{
    fn from(column: C) -> Self {
        Self::SetExcluded(column.dyn_column())
    }
}
impl FormatSql for SetColumm {
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
#[derive(Debug)]
pub enum OnConflictAction {
    DoNothing,
    DoUpdate(Vec<SetColumm>),
}
impl OnConflictAction {
    pub fn update<C>(columns: Vec<C>) -> Self
    where
        C: ColumnType + 'static,
    {
        Self::DoUpdate(columns.into_iter().map(SetColumm::from).collect())
    }
}

impl FormatSql for OnConflictAction {
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
#[derive(Debug)]
pub struct OnConflict {
    pub columns: Vec<DynColumn>,
    pub action: OnConflictAction,
}

impl FormatSql for OnConflict {
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
impl FormatSql for Option<OnConflict> {
    fn format_sql(&self) -> Cow<'_, str> {
        match self {
            Some(on_conflict) => on_conflict.format_sql(),
            None => Cow::Borrowed(""),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        testing::TestTableColumn, ColumnType, FormatSql, OnConflict, OnConflictAction, SetColumm,
    };

    #[test]
    fn format_do_nothing() {
        let on_conflict = OnConflict {
            columns: vec![TestTableColumn::FirstName.dyn_column()],
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
            columns: vec![TestTableColumn::Email.dyn_column()],
            action: OnConflictAction::DoUpdate(vec![
                SetColumm::SetExcluded(TestTableColumn::FirstName.dyn_column()),
                SetColumm::SetExcluded(TestTableColumn::Email.dyn_column()),
            ]),
        };

        assert_eq!(
            on_conflict.format_sql(),
            "ON CONFLICT (email) DO UPDATE SET first_name = EXCLUDED.first_name, email = EXCLUDED.email"
        );
    }
}
