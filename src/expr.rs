use std::borrow::Cow;
mod alias;
mod filter;
mod select_expr;
mod value;
pub use alias::*;
pub use filter::*;
mod function;
pub use function::*;
pub use value::*;
mod multi;
mod returning;
pub use returning::*;
mod other;
pub use multi::*;
pub use other::*;
pub use select_expr::*;
pub type DynExpr<'args> = Box<dyn ExprType<'args> + 'args>;
use super::{ColumnType, DynColumn, FormatSql, HasArguments};
pub trait ExprType<'args> {
    fn process(self: Box<Self>, args: &mut dyn HasArguments<'args>) -> Expr
    where
        Self: 'args;
    fn process_unboxed(self, args: &mut dyn HasArguments<'args>) -> Expr
    where
        Self: 'args;

    fn equals<E>(self, value: E) -> FilterConditionBuilder<'args>
    where
        Self: Sized + 'args,
        E: ExprType<'args> + 'args,
    {
        FilterConditionBuilder::CompareValue {
            left: Box::new(self),
            comparison: SQLComparison::Equals,
            right: Box::new(value),
        }
    }
    fn like<E>(self, value: E) -> FilterConditionBuilder<'args>
    where
        Self: Sized + 'args,
        E: ExprType<'args> + 'args,
    {
        FilterConditionBuilder::CompareValue {
            left: Box::new(self),
            comparison: SQLComparison::Like,
            right: Box::new(value),
        }
    }
    fn is_not_null(self) -> FilterConditionBuilder<'args>
    where
        Self: Sized + 'args,
    {
        FilterConditionBuilder::NotNull(Box::new(self))
    }
    fn is_null(self) -> FilterConditionBuilder<'args>
    where
        Self: Sized + 'args,
    {
        FilterConditionBuilder::Null(Box::new(self))
    }
}

#[derive(Debug)]
pub enum Expr {
    ArgumentIndex(ArgumentIndex),
    Function(ExprFunction),
    Column(DynColumn),
    Condition(Box<SQLCondition>),
    Select(SelectExpr),
    Alias(ExprAlias),
    All(All),
    Default(SqlDefault),
    Multiple(MultipleExpr),
}
impl<C> From<C> for Expr
where
    C: ColumnType + 'static,
{
    fn from(column: C) -> Self {
        Expr::Column(column.dyn_column())
    }
}

impl FormatSql for Expr {
    fn format_sql(&self) -> Cow<'_, str> {
        match self {
            Expr::ArgumentIndex(index) => index.format_sql(),
            Expr::Function(function) => function.format_sql(),
            Expr::Column(column) => column.formatted_column(),
            Expr::Condition(condition) => condition.format_sql(),
            Expr::Select(select) => select.format_sql(),
            Expr::Alias(alias) => alias.format_sql(),
            Expr::All(all) => all.format_sql(),
            Expr::Multiple(multiple) => multiple.format_sql(),
            Expr::Default(sql_default) => sql_default.format_sql(),
        }
    }
}

#[derive(Debug, Default)]
pub struct All;
impl<'args> ExprType<'args> for All {
    fn process(self: Box<Self>, _: &mut dyn HasArguments<'args>) -> Expr
    where
        Self: 'args,
    {
        Expr::All(*self)
    }

    fn process_unboxed(self, _: &mut dyn HasArguments<'args>) -> Expr
    where
        Self: 'args,
    {
        Expr::All(self)
    }
}
impl All {
    pub fn new<'args>() -> All {
        All
    }
}
impl FormatSql for All {
    fn format_sql(&self) -> Cow<'_, str> {
        Cow::Borrowed("*")
    }
}
impl From<All> for Expr {
    fn from(all: All) -> Self {
        Expr::All(all)
    }
}
#[derive(Debug, Default)]

pub struct SqlDefault;
impl From<SqlDefault> for Expr {
    fn from(default: SqlDefault) -> Self {
        Expr::Default(default)
    }
}

impl<'args> ExprType<'args> for SqlDefault {
    fn process(self: Box<Self>, _: &mut dyn HasArguments<'args>) -> Expr
    where
        Self: 'args,
    {
        Expr::Default(*self)
    }

    fn process_unboxed(self, _: &mut dyn HasArguments<'args>) -> Expr
    where
        Self: 'args,
    {
        Expr::Default(self)
    }
}

impl FormatSql for SqlDefault {
    fn format_sql(&self) -> Cow<'_, str> {
        Cow::Borrowed("DEFAULT")
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArgumentIndex(usize);

impl FormatSql for ArgumentIndex {
    fn format_sql(&self) -> Cow<'_, str> {
        Cow::Owned(format!("${}", self.0))
    }
}
