use std::{borrow::Cow, fmt::Debug};

use crate::arguments::ArgumentHolder;
use crate::{Aliasable, Expr, ExprType, WrapInFunction};

#[derive(Debug)]
pub struct DynColumn(Box<dyn ColumnType + Send + Sync>);
impl DynColumn {
    pub fn new<C>(column: C) -> Self
    where
        C: ColumnType + Send + Sync + 'static,
    {
        Self(Box::new(column))
    }
}
impl ColumnType for DynColumn {
    fn column_name(&self) -> &'static str {
        self.0.column_name()
    }
    fn dyn_column(self) -> DynColumn
    where
        Self: Sized + Send + Sync + 'static,
    {
        self
    }

    fn format_column_with_prefix(&self, prefix: Option<&str>) -> Cow<'static, str> {
        self.0.format_column_with_prefix(prefix)
    }
    fn formatted_column(&self) -> Cow<'static, str> {
        self.0.formatted_column()
    }
}

pub trait ColumnType: Debug + Send + Sync {
    fn column_name(&self) -> &'static str;
    /// Should return the `{table_name}.{column_name}` format
    fn formatted_column(&self) -> Cow<'static, str> {
        Cow::Borrowed(self.column_name())
    }
    fn format_column_with_prefix(&self, prefix: Option<&str>) -> Cow<'static, str> {
        if let Some(prefix) = prefix {
            Cow::Owned(format!("{}.{}", prefix, self.column_name()))
        } else {
            Cow::Borrowed(self.column_name())
        }
    }

    fn dyn_column(self) -> DynColumn
    where
        Self: Sized + Send + Sync + 'static,
    {
        DynColumn::new(self)
    }
}

pub trait AllColumns {
    fn all() -> Vec<Self>
    where
        Self: Sized;

    fn all_dyn() -> Vec<DynColumn>
    where
        Self: Sized + ColumnType + 'static,
    {
        Self::all().into_iter().map(|c| c.dyn_column()).collect()
    }
}

impl<'args, C> ExprType<'args> for C
where
    C: ColumnType + 'static,
{
    fn process(self: Box<Self>, _: &mut ArgumentHolder<'args>) -> crate::Expr
    where
        Self: 'args,
    {
        Expr::Column((*self).dyn_column())
    }

    fn process_unboxed(self, _: &mut ArgumentHolder<'args>) -> crate::Expr
    where
        Self: 'args,
    {
        Expr::Column((self).dyn_column())
    }
}
impl<'args, C> Aliasable<'args> for C where C: ColumnType + 'static {}
impl<'args, C> WrapInFunction<'args> for C where C: ColumnType + 'static {}
