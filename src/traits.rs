use std::borrow::Cow;
use std::fmt::{Debug, Display};
use std::future::Future;

use sqlx::postgres::PgRow;
use sqlx::query::{Query, QueryAs, QueryScalar};
use sqlx::{Database, Postgres};
use sqlx::{Decode, FromRow, Type};
use tracing::trace;

use crate::expr::{ExprType, FilterConditionBuilder, HasArguments, SQLCondition};

pub trait FormatSql: Debug {
    fn format_sql(&self) -> Cow<'_, str>;
}
impl FormatSql for () {
    #[inline]
    fn format_sql(&self) -> Cow<'_, str> {
        Cow::default()
    }
}
pub struct SpaceBefore<'sql, Sql>(pub Option<&'sql Sql>)
where
    Sql: FormatSql;
impl<'sql, Sql: FormatSql> Display for SpaceBefore<'sql, Sql> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(sql) = self.0 {
            write!(f, " {}", sql.format_sql())
        } else {
            Ok(())
        }
    }
}
impl<'sql, Sql: FormatSql> From<Option<&'sql Sql>> for SpaceBefore<'sql, Sql> {
    fn from(sql: Option<&'sql Sql>) -> Self {
        Self(sql)
    }
}
impl<'sql, Sql: FormatSql> From<&'sql Sql> for SpaceBefore<'sql, Sql> {
    fn from(sql: &'sql Sql) -> Self {
        Self(Some(sql))
    }
}

pub trait FormatSqlQuery {
    fn format_sql_query(&mut self) -> &str;

    #[cfg(feature = "format")]
    fn formatted_sql_query(&mut self) -> String {
        use sqlformat::{FormatOptions, QueryParams};

        let sql = self.format_sql_query();

        sqlformat::format(sql, &QueryParams::None, &FormatOptions::default())
    }
}
/// A base Query TOol type that can be used to build queries.
///
/// ## Note
///
/// These query builders are not meant to be fill every case in sql.
///
/// They are great for the simple cases but the more you need. The more you should consider using sql directly.
pub trait QueryTool<'args>: HasArguments<'args> + FormatSqlQuery + Debug {
    /// Builds a query that can be executed.
    ///
    /// See [sqlx::query_with] for more information.
    fn query(&mut self) -> Query<'_, Postgres, <Postgres as Database>::Arguments<'args>> {
        let args = self.holder().take_arguments_or_error();
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
        let args = self.holder().take_arguments_or_error();

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
        let args = self.holder().take_arguments_or_error();

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
    fn filter<L: ExprType<'args> + 'args, R: ExprType<'args> + 'args>(
        &mut self,
        filter: FilterConditionBuilder<'args, L, R>,
    ) -> &mut Self {
        let condition = filter.process_inner(self.holder());

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
pub(crate) struct FormatWhereItem<'query, F>(pub &'query F)
where
    F: FormatWhere;
impl<'query, F> Display for FormatWhereItem<'query, F>
where
    F: FormatWhere,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " WHERE {}", self.0.format_where())
    }
}
pub trait ExpressionWhereable<'args>: Sized {
    fn filter<L: ExprType<'args> + 'args, R: ExprType<'args> + 'args>(
        mut self,
        filter: FilterConditionBuilder<'args, L, R>,
    ) -> Self {
        self.push_where_comparison(filter);

        self
    }

    /// Required to push the where comparison to the query
    ///
    /// The internal structure will be a Vec<WhereComparison>
    ///
    /// Each are concatenated with an AND
    fn push_where_comparison<L: ExprType<'args> + 'args, R: ExprType<'args> + 'args>(
        &mut self,
        comparison: FilterConditionBuilder<'args, L, R>,
    );
}
