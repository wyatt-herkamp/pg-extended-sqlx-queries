use std::borrow::Cow;

use sqlx::{postgres::PgArguments, Arguments, Database, Postgres};

/// A sql tool that has [Arguments](sqlx::Arguments) that can be used to build a query.
///
/// Arguments being the values that will be used to fill in the placeholders in the query.
///
/// # Note
/// The tools are highly inspired by the QueryBuilder in sqlx.
///
/// This means it stores arguments in an [Option] and can be removed from the type and may cause panics. if removed and then the item is used again
pub trait HasArguments<'args> {
    /// Returns a mutable reference to the argument holder.
    fn holder(&mut self) -> &mut ArgumentHolder<'args>;
}

use crate::FormatSql;
pub struct ArgumentHolder<'args> {
    arguments: Option<<Postgres as Database>::Arguments<'args>>,
}
impl Default for ArgumentHolder<'_> {
    fn default() -> Self {
        Self {
            arguments: Some(Default::default()),
        }
    }
}
impl<'args> ArgumentHolder<'args> {
    /// Takes the arguments list or panics if it is not available.
    pub fn take_arguments_or_error(&mut self) -> PgArguments {
        self.arguments.take().expect("Arguments already taken")
    }
    /// Borrows the arguments list or panics if it is not available.
    pub fn borrow_arguments_or_error(&mut self) -> &mut PgArguments {
        self.arguments.as_mut().expect("Arguments already taken")
    }
    /// Pushes an argument to the arguments list.
    pub fn push_argument<T>(&mut self, value: T) -> ArgumentIndex
    where
        T: 'args + sqlx::Encode<'args, Postgres> + sqlx::Type<Postgres>,
    {
        let arguments = self.borrow_arguments_or_error();
        arguments.add(value).expect("Failed to add argument");
        ArgumentIndex(arguments.len())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArgumentIndex(usize);

impl FormatSql for ArgumentIndex {
    fn format_sql(&self) -> Cow<'_, str> {
        Cow::Owned(format!("${}", self.0))
    }
}
