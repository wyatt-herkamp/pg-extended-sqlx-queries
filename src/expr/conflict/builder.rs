use crate::prelude::*;

use super::{ConflictAction, SetColumm};

pub enum SetColummBuilder<'args> {
    /// Does SET `column = excluded.column`
    SetExcluded(DynColumn),
    SetExpr {
        column: DynColumn,
        expr: DynExpr<'args>,
    },
}
impl<C: ColumnType + 'static> From<C> for SetColummBuilder<'_> {
    fn from(column: C) -> Self {
        Self::SetExcluded(column.dyn_column())
    }
}
pub enum ConflictActionBuilder<'args> {
    DoNothing,
    DoUpdate(Vec<SetColummBuilder<'args>>),
}
impl<'args> ConflictActionBuilder<'args> {
    pub fn do_nothing() -> Self {
        Self::DoNothing
    }
    pub fn do_update() -> Self {
        Self::DoUpdate(Vec::new())
    }
    pub fn update_to_excluded<C>(columns: Vec<C>) -> Self
    where
        C: ColumnType + 'static,
    {
        Self::DoUpdate(columns.into_iter().map(SetColummBuilder::from).collect())
    }
    pub fn set_many_to_excluded<C>(self, new_columns: Vec<C>) -> Self
    where
        C: ColumnType + 'static,
    {
        match self {
            Self::DoUpdate(mut columns) => {
                let new_columns: Vec<_> = new_columns
                    .into_iter()
                    .map(SetColummBuilder::from)
                    .collect();
                columns.extend(new_columns);
                Self::DoUpdate(columns)
            }
            _ => Self::DoUpdate(
                new_columns
                    .into_iter()
                    .map(SetColummBuilder::from)
                    .collect(),
            ),
        }
    }
    pub fn set_column_to_excluded<C>(self, column: C) -> Self
    where
        C: ColumnType + 'static,
    {
        match self {
            Self::DoUpdate(mut columns) => {
                columns.push(SetColummBuilder::SetExcluded(column.dyn_column()));
                Self::DoUpdate(columns)
            }
            _ => Self::DoUpdate(vec![SetColummBuilder::SetExcluded(column.dyn_column())]),
        }
    }

    pub fn set_column(self, column: DynColumn, expr: DynExpr<'args>) -> Self {
        match self {
            Self::DoUpdate(mut columns) => {
                columns.push(SetColummBuilder::SetExpr { column, expr });
                Self::DoUpdate(columns)
            }
            _ => Self::DoUpdate(vec![SetColummBuilder::SetExpr { column, expr }]),
        }
    }
    pub fn process(self, args: &mut ArgumentHolder<'args>) -> ConflictAction {
        match self {
            Self::DoNothing => ConflictAction::DoNothing,
            Self::DoUpdate(columns) => {
                let columns = columns
                    .into_iter()
                    .map(|column| match column {
                        SetColummBuilder::SetExcluded(column) => SetColumm::SetExcluded(column),
                        SetColummBuilder::SetExpr { column, expr } => SetColumm::SetExpr {
                            column,
                            expr: expr.process_unboxed(args),
                        },
                    })
                    .collect();
                ConflictAction::DoUpdate(columns)
            }
        }
    }
}
