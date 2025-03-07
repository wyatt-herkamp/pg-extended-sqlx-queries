use std::{borrow::Cow, fmt::Debug};

use crate::{
    expr::{ArgumentHolder, Expr, ExprType, HasArguments, SQLCondition, SQLOrder},
    pagination::PaginationSupportingTool,
    prelude::{ColumnType, DynColumn},
    traits::*,
};

mod count;
mod exists;
mod join;
pub use count::*;
pub use exists::*;
pub use join::*;
use tracing::{debug, instrument};

pub struct SelectQueryBuilder<'args> {
    table: Cow<'args, str>,
    select: Vec<Expr>,
    where_comparisons: Vec<SQLCondition>,
    sql: Option<String>,
    joins: Vec<join::Join>,
    arguments: ArgumentHolder<'args>,
    limit: Option<i32>,
    offset: Option<i32>,
    distinct: bool,
    order_by: Option<(DynColumn, SQLOrder)>,
}

impl PaginationSupportingTool for SelectQueryBuilder<'_> {
    fn limit(&mut self, limit: i32) -> &mut Self {
        self.limit = Some(limit);
        self
    }

    fn offset(&mut self, offset: i32) -> &mut Self {
        self.offset = Some(offset);
        self
    }
}
impl<'args> SelectQueryBuilder<'args> {
    pub fn new(table: &'args str) -> Self {
        Self {
            table: Cow::Borrowed(table),
            select: Vec::new(),
            where_comparisons: vec![],
            sql: None,
            joins: Vec::new(),
            arguments: Default::default(),
            limit: None,
            offset: None,
            order_by: None,
            distinct: false,
        }
    }
    pub fn distinct(&mut self) -> &mut Self {
        self.distinct = true;
        self
    }
    pub fn with_columns<C>(table: &'args str, columns: impl Into<Vec<C>>) -> Self
    where
        C: ColumnType + 'static,
    {
        let columns = columns
            .into()
            .into_iter()
            .map(|column| column.dyn_column())
            .collect();
        let mut this = Self::new(table);
        this.select_many(columns);
        this
    }

    pub fn order_by<C>(&mut self, column: C, order: SQLOrder) -> &mut Self
    where
        C: ColumnType + 'static,
    {
        self.order_by = Some((column.dyn_column(), order));
        self
    }

    pub fn join<F>(&mut self, table: &'static str, join_type: JoinType, join: F) -> &mut Self
    where
        F: FnOnce(join::JoinBuilder<'_, 'args, Self>) -> join::Join,
    {
        let builder = join::JoinBuilder::new(self, table, join_type);
        let join = join(builder);

        self.joins.push(join);

        self
    }

    pub fn select<E>(&mut self, expr: E) -> &mut Self
    where
        E: ExprType<'args> + 'args,
    {
        let expr = expr.process_unboxed(&mut self.arguments);
        self.select.push(expr);
        self
    }
    pub fn select_many<E>(&mut self, exprs: Vec<E>) -> &mut Self
    where
        E: ExprType<'args> + 'args,
    {
        let exprs: Vec<_> = exprs
            .into_iter()
            .map(|expr| expr.process_unboxed(&mut self.arguments))
            .collect();
        self.select.extend(exprs);
        self
    }
    /// SELECT * FROM table
    pub fn select_all(&mut self) -> &mut Self {
        self.select.push(Expr::Wildcard(Default::default()));
        self
    }
}
impl<'args> FormatSqlQuery for SelectQueryBuilder<'args> {
    #[instrument(skip(self), fields(table = %self.table, statement.type = "SELECT"))]
    fn format_sql_query(&mut self) -> &str {
        let mut columns: Vec<_> = self
            .select
            .iter_mut()
            .map(|item| item.format_sql())
            .collect();
        for join in &self.joins {
            columns.extend(join.columns_to_select.iter().map(|expr| expr.format_sql()));
        }

        let concat_columns = columns.join(", ");
        let distinct = if self.distinct { " DISTINCT " } else { " " };
        let mut sql = format!(
            "SELECT{distinct}{columns} FROM {table}",
            columns = concat_columns,
            table = self.table
        );

        for join in &self.joins {
            sql.push(' ');
            sql.push_str(&join.format_sql());
        }
        if !self.where_comparisons.is_empty() {
            let where_sql = self.format_where();
            sql.push_str(" WHERE ");
            sql.push_str(&where_sql);
        }

        if let Some((column, order)) = &self.order_by {
            sql.push_str(" ORDER BY ");
            sql.push_str(column.column_name());
            sql.push(' ');
            sql.push_str(&order.format_sql());
        }

        if let Some(limit) = self.limit {
            sql.push_str(" LIMIT ");
            sql.push_str(&limit.to_string());
        }
        if let Some(offset) = self.offset {
            sql.push_str(" OFFSET ");
            sql.push_str(&offset.to_string());
        }
        debug!(?sql, "SelectQueryBuilder::format_sql_query");
        self.sql = Some(sql);

        self.sql.as_ref().expect("SQL not set")
    }
}
impl<'args> QueryTool<'args> for SelectQueryBuilder<'args> {}
impl<'args> Debug for SelectQueryBuilder<'args> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectQueryBuilder")
            .field("table", &self.table)
            .field("select", &self.select)
            .field("where_comparisons", &self.where_comparisons)
            .field("sql", &self.sql)
            .field("joins", &self.joins)
            .field("limit", &self.limit)
            .field("offset", &self.offset)
            .field("order_by", &self.order_by)
            .finish()
    }
}
impl<'args> HasArguments<'args> for SelectQueryBuilder<'args> {
    fn holder(&mut self) -> &mut ArgumentHolder<'args> {
        &mut self.arguments
    }
}
impl FormatWhere for SelectQueryBuilder<'_> {
    fn get_conditions(&self) -> &[SQLCondition] {
        &self.where_comparisons
    }
}
impl<'args> WhereableTool<'args> for SelectQueryBuilder<'args> {
    fn push_where_comparison(&mut self, comparison: SQLCondition) {
        self.where_comparisons.push(comparison);
    }
}
#[cfg(test)]
mod tests {

    use crate::{
        expr::Collate,
        pagination::PaginationSupportingTool,
        prelude::*,
        testing::{AnotherTable, AnotherTableColumn, TestTable, TestTableColumn, print_query},
    };
    #[test]
    fn basic_select() {
        let mut select = SelectQueryBuilder::new(TestTable::table_name());
        select
            .select(TestTableColumn::Age)
            .select(TestTableColumn::Email)
            .select(TestTableColumn::FirstName)
            .select(TestTableColumn::LastName)
            .select(TestTableColumn::Id.alias("user_id"))
            .filter(TestTableColumn::Age.equals(50.value()));

        let sql = select.format_sql_query();

        assert_eq!(
            sql,
            "SELECT test_table.age, test_table.email, test_table.first_name, test_table.last_name, test_table.id AS user_id FROM test_table WHERE test_table.age = $1"
        );

        print_query(sql, "Basic Select");
    }

    #[test]
    fn basic_select_with_expr() {
        let mut select = SelectQueryBuilder::new(TestTable::table_name());
        select
            .select(TestTableColumn::Id.alias("user_id"))
            .select(TestTableColumn::Email)
            .select(TestTableColumn::FirstName)
            .limit(10)
            .offset(5)
            .select(
                SqlFunctionBuilder::count_all()
                    .then(SqlFunctionBuilder::over())
                    .alias("total_count"),
            )
            .filter(TestTableColumn::Age.equals(50.value()));

        let sql = select.format_sql_query();

        assert_eq!(
            sql,
            "SELECT test_table.id AS user_id, test_table.email, test_table.first_name, COUNT(*) OVER() AS total_count FROM test_table WHERE test_table.age = $1 LIMIT 10 OFFSET 5"
        );

        print_query(sql, "Basic Select with Expr");
    }

    #[test]
    fn select_join() {
        let mut select = SelectQueryBuilder::new(TestTable::table_name());
        select
            .select(TestTableColumn::Id.alias("user_id"))
            .join(AnotherTable::table_name(), JoinType::Inner, |join| {
                join.select(AnotherTableColumn::Age)
                    .on(TestTableColumn::Id.equals(AnotherTableColumn::Id))
            })
            .filter(TestTableColumn::Age.equals(50.value()));

        let sql = select.format_sql_query();
        assert_eq!(
            sql,
            "SELECT test_table.id AS user_id, another_table.age FROM test_table INNER JOIN another_table ON test_table.id = another_table.id WHERE test_table.age = $1"
        );
        print_query(sql, "Select Join");
    }
    #[test]
    fn select_any() {
        let mut select = SelectQueryBuilder::new(TestTable::table_name());
        select.select(TestTableColumn::Id.alias("user_id"));

        select.filter(TestTableColumn::Phone.equals(vec!["555-555-5555", "555-555-7777"].any()));

        let sql = select.format_sql_query();
        assert_eq!(
            sql,
            "SELECT test_table.id AS user_id FROM test_table WHERE test_table.phone = ANY($1)"
        );

        print_query(sql, "Select Any");
    }
    #[test]
    fn select_between() {
        let mut select = SelectQueryBuilder::new(TestTable::table_name());
        select.select(TestTableColumn::Id.alias("user_id"));

        select.filter(TestTableColumn::Age.between(50, 100));

        let sql = select.format_sql_query();
        assert_eq!(
            sql,
            "SELECT test_table.id AS user_id FROM test_table WHERE test_table.age BETWEEN $1 AND $2"
        );

        print_query(sql, "Select Between");
    }
    #[test]
    fn select_collate() {
        let mut select = SelectQueryBuilder::new(TestTable::table_name());
        select.select_all().filter(
            TestTableColumn::FirstName
                .equals("test".value())
                .collate(Collate::TrTrxIcu)
                .or(TestTableColumn::LastName
                    .equals("test".value())
                    .collate(Collate::TrTrxIcu)),
        );

        let sql = select.format_sql_query();
        assert_eq!(
            sql,
            "SELECT * FROM test_table WHERE test_table.first_name = $1 COLLATE \"tr-TR-x-icu\" OR test_table.last_name = $2 COLLATE \"tr-TR-x-icu\""
        );
        print_query(sql, "Select Collate");
    }

    #[test]
    fn select_with_grouped() {
        let mut select = SelectQueryBuilder::new(TestTable::table_name());
        select.select_all().filter(
            TestTableColumn::FirstName.equals("test".value()).and(
                TestTableColumn::LastName
                    .equals("test".value())
                    .and(TestTableColumn::Phone.like("555%"))
                    .grouped(),
            ),
        );

        let sql = select.format_sql_query();
        assert_eq!(
            sql,
            "SELECT * FROM test_table WHERE test_table.first_name = $1 AND (test_table.last_name = $2 AND test_table.phone LIKE $3)"
        );

        print_query(sql, "Select with Grouped");
    }
    #[test]
    fn select_distinct() {
        let mut select = SelectQueryBuilder::new(TestTable::table_name());
        select.select_all().distinct();
        let sql = select.format_sql_query();
        assert_eq!(sql, "SELECT DISTINCT * FROM test_table");

        print_query(sql, "select distinct");
    }
}
