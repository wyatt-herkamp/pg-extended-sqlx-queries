use std::borrow::Cow;
use std::future::Future;

use sqlx::postgres::{PgArguments, PgRow};
use sqlx::query::{Query, QueryAs, QueryScalar};
use sqlx::{Arguments, Decode, FromRow, Type};
use sqlx::{Database, Postgres};
use tracing::trace;

use crate::{DynEncode, FilterConditionBuilder, SQLCondition};

/// A sql tool that has [Arguments](sqlx::Arguments) that can be used to build a query.
///
/// Arguments being the values that will be used to fill in the placeholders in the query.
///
/// # Note
/// The tools are highly inspired by the QueryBuilder in sqlx.
///
/// This means it stores arguments in an [Option] and can be removed from the type and may cause panics. if removed and then the item is used again
pub trait HasArguments<'args> {
    /// Takes the arguments list or panics if it is not available.
    fn take_arguments_or_error(&mut self) -> PgArguments;
    /// Borrows the arguments list or panics if it is not available.
    fn borrow_arguments_or_error(&mut self) -> &mut PgArguments;
    /// Pushes an argument to the arguments list.
    fn push_argument<T>(&mut self, value: T) -> usize
    where
        T: 'args + sqlx::Encode<'args, Postgres> + sqlx::Type<Postgres>,
        Self: Sized,
    {
        let arguments = self.borrow_arguments_or_error();
        arguments.add(value).expect("Failed to add argument");
        arguments.len()
    }

    fn push_dyn_argument(&mut self, value: DynEncode<'args>) -> usize {
        let arguments = self.borrow_arguments_or_error();
        arguments
            .add(value)
            .expect("Failed to add dynamic argument");
        arguments.len()
    }
}

pub trait FormatSql {
    fn format_sql(&self) -> Cow<'_, str>;
}

pub trait FormatSqlQuery {
    fn format_sql_query(&mut self) -> &str;

    #[cfg(feature = "format")]
    fn formatted_sql_query(&mut self) -> String {
        use sqlformat::{FormatOptions, QueryParams};

        let sql = self.format_sql_query();
        let sql = sqlformat::format(sql, &QueryParams::None, &FormatOptions::default());
        sql
    }
}
/// A base Query TOol type that can be used to build queries.
///
/// ## Note
///
/// These query builders are not meant to be fill every case in sql.
///
/// They are great for the simple cases but the more you need. The more you should consider using sql directly.
pub trait QueryTool<'args>: HasArguments<'args> + FormatSqlQuery {
    /// Builds a query that can be executed.
    ///
    /// See [sqlx::query_with] for more information.
    fn query(&mut self) -> Query<'_, Postgres, <Postgres as Database>::Arguments<'args>> {
        let args = self.take_arguments_or_error();
        let sql = self.format_sql_query();
        trace!(?sql, "Generated SQL");

        sqlx::query_with(sql, args)
    }
    /// Builds a query that can be executed and returns the results as a type.
    ///
    /// See [sqlx::query_as_with] for more information.
    fn query_as<T>(&mut self) -> QueryAs<'_, Postgres, T, <Postgres as Database>::Arguments<'args>>
    where
        T: for<'r> FromRow<'r, PgRow>,
    {
        let args = self.take_arguments_or_error();

        let sql = self.format_sql_query();
        trace!(?sql, "Generated SQL");
        sqlx::query_as_with(sql, args)
    }
    /// Builds a query that can be executed and returns the results as a scalar.
    ///
    /// See [sqlx::query_scalar_with] for more information.
    fn query_scalar<O>(
        &mut self,
    ) -> QueryScalar<'_, Postgres, O, <Postgres as Database>::Arguments<'args>>
    where
        (O,): for<'r> FromRow<'r, PgRow>,
    {
        let args = self.take_arguments_or_error();

        let sql = self.format_sql_query();
        trace!(?sql, "Generated SQL");
        sqlx::query_scalar_with(sql, args)
    }
}
/// Tools such as [SelectExists](super::SelectExists) and [SelectCount](super::SelectCount)
/// that can be used to build queries that return a single value.
pub trait QueryScalarTool<'args>: QueryTool<'args> + Send {
    type Output: for<'r> Decode<'r, Postgres> + Type<Postgres> + Send + Unpin;
    /// Executes the query and returns the number of rows affected.
    ///
    /// See [sqlx::query] for more information.
    fn execute<'c, E>(
        &mut self,
        conn: E,
    ) -> impl Future<Output = Result<Self::Output, sqlx::Error>> + Send
    where
        E: sqlx::Executor<'c, Database = Postgres> + Send,
    {
        async move {
            let query = self.query_scalar();
            let result = query.fetch_one(conn).await?;
            Ok(result)
        }
    }
}

pub trait WhereableTool<'args>: HasArguments<'args> + Sized {
    fn filter(&mut self, filter: FilterConditionBuilder<'args>) -> &mut Self {
        let condition = filter.process_inner(self);

        self.push_where_comparison(condition);

        self
    }

    /// Required to push the where comparison to the query
    ///
    /// The internal structure will be a Vec<WhereComparison>
    ///
    /// Each are concatenated with an AND
    fn push_where_comparison(&mut self, comparison: SQLCondition);
}
pub trait FormatWhere {
    /// Gets the conditions to be formatted
    fn get_conditions(&self) -> &[SQLCondition];
    /// Joins all conditions with an AND
    fn format_where(&self) -> String {
        self.get_conditions()
            .iter()
            .map(|condition| condition.format_sql())
            .collect::<Vec<_>>()
            .join(" AND ")
    }
}
pub trait ExpressionWhereable<'args>: Sized {
    fn filter(mut self, filter: FilterConditionBuilder<'args>) -> Self {
        self.push_where_comparison(filter);

        self
    }

    /// Required to push the where comparison to the query
    ///
    /// The internal structure will be a Vec<WhereComparison>
    ///
    /// Each are concatenated with an AND
    fn push_where_comparison(&mut self, comparison: FilterConditionBuilder<'args>);
}
