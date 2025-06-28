use std::borrow::Cow;

use crate::prelude::*;
mod builder;
mod query;
pub use builder::*;
pub use query::*;
pub enum SetColummBuilder<'args> {
    /// Does SET `column = excluded.column`
    SetExcluded(DynColumn),
    SetExpr {
        column: DynColumn,
        expr: DynExpr<'args>,
    },
}
#[derive(Debug)]
pub enum SetColumm {
    /// Does SET `column = excluded.column`
    SetExcluded(DynColumn),
    SetExpr {
        column: DynColumn,
        expr: Expr,
    },
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
            Self::SetExpr { column, expr } => format!(
                "{column_name} = {expr}",
                column_name = column.column_name(),
                expr = expr.format_sql()
            )
            .into(),
        }
    }
}
#[derive(Debug)]
pub enum ConflictAction {
    DoNothing,
    DoUpdate(Vec<SetColumm>),
}

impl FormatSql for ConflictAction {
    fn format_sql(&self) -> Cow<'_, str> {
        match self {
            Self::DoNothing => "DO NOTHING".into(),
            Self::DoUpdate(columns) => {
                let columns = columns
                    .iter()
                    .map(|column| column.format_sql())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("DO UPDATE SET {columns}").into()
            }
        }
    }
}

#[derive(Debug)]
pub enum ConflictTarget {
    Columns(Vec<DynColumn>),
    Constraint(&'static str),
}
impl ConflictTarget {
    pub fn columns<C>(columns: Vec<C>) -> Self
    where
        C: ColumnType + 'static,
    {
        Self::Columns(
            columns
                .into_iter()
                .map(|column| column.dyn_column())
                .collect(),
        )
    }
    pub fn constraint(constraint: &'static str) -> Self {
        Self::Constraint(constraint)
    }
}
impl FormatSql for ConflictTarget {
    fn format_sql(&self) -> Cow<'_, str> {
        match self {
            Self::Columns(columns) => {
                let columns = columns
                    .iter()
                    .map(|column| column.column_name())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({columns})").into()
            }
            Self::Constraint(constraint) => format!("ON CONSTRAINT {constraint}").into(),
        }
    }
}
#[derive(Debug)]
pub struct OnConflict {
    pub conflict_target: ConflictTarget,
    pub action: ConflictAction,
}

impl FormatSql for OnConflict {
    fn format_sql(&self) -> Cow<'_, str> {
        let conflict_target = self.conflict_target.format_sql();
        format!(
            " ON CONFLICT {conflict_target} {action}",
            conflict_target = conflict_target,
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

    use crate::prelude::*;
    use crate::testing::TestTableColumn;

    #[test]
    fn format_do_nothing() {
        let on_conflict = OnConflict {
            conflict_target: ConflictTarget::columns(vec![TestTableColumn::FirstName]),
            action: ConflictAction::DoNothing,
        };

        assert_eq!(
            on_conflict.format_sql(),
            " ON CONFLICT (first_name) DO NOTHING"
        );
    }

    #[test]
    fn format_set_columns() {
        let on_conflict = OnConflict {
            conflict_target: ConflictTarget::columns(vec![TestTableColumn::Email]),
            action: ConflictAction::DoUpdate(
                vec![TestTableColumn::FirstName, TestTableColumn::Email]
                    .into_iter()
                    .map(SetColumm::from)
                    .collect(),
            ),
        };

        assert_eq!(
            on_conflict.format_sql(),
            " ON CONFLICT (email) DO UPDATE SET first_name = EXCLUDED.first_name, email = EXCLUDED.email"
        );
    }
}
