use std::borrow::Cow;
mod join;
use crate::prelude::ColumnType;
use crate::select::{Join, JoinType};
use crate::traits::{ExpressionWhereable, FormatSql, FormatWhere};
use crate::{pagination::PaginationOwnedSupportingTool, prelude::DynColumn};
pub use join::*;

use super::{Aliasable, DynExpr, Expr, ExprType, FilterConditionBuilder, WrapInFunction};
use super::{SQLCondition, SQLOrder, arguments::ArgumentHolder, concat_with_comma};
#[derive(Debug, PartialEq)]
pub struct SelectExpr {
    table: &'static str,
    select: Vec<Expr>,
    where_comparisons: Vec<SQLCondition>,
    limit: Option<i64>,
    order_by: Option<(DynColumn, SQLOrder)>,
    joins: Vec<Join>,
}
impl FormatWhere for SelectExpr {
    fn get_conditions(&self) -> &[SQLCondition] {
        &self.where_comparisons
    }
}
impl FormatSql for SelectExpr {
    fn format_sql(&self) -> Cow<'_, str> {
        let mut columns: Vec<_> = self.select.iter().map(|item| item.format_sql()).collect();
        for join in &self.joins {
            columns.extend(join.columns_to_select.iter().map(|expr| expr.format_sql()));
        }

        let concat_columns = columns.join(", ");
        // Wrap the {columns} in parentheses because they are only allowed to return 1 column

        let mut sql = if self.select.len() == 1 {
            format!(
                "(SELECT {columns} FROM {table}",
                columns = concat_columns,
                table = self.table
            )
        } else {
            format!(
                "(SELECT ({columns}) FROM {table}",
                columns = concat_columns,
                table = self.table
            )
        };
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
            sql.push_str(&column.full_name());
            sql.push(' ');
            sql.push_str(&order.format_sql());
        }
        if let Some(limit) = self.limit {
            sql.push_str(" LIMIT ");
            sql.push_str(&limit.to_string());
        }

        sql.push(')');

        Cow::Owned(sql)
    }
}
pub struct SelectExprBuilder<'args> {
    table: &'static str,
    select: Vec<DynExpr<'args>>,
    where_comparisons: Vec<FilterConditionBuilder<'args, DynExpr<'args>, DynExpr<'args>>>,
    limit: Option<i64>,
    offset: Option<i64>,
    order_by: Option<(DynColumn, SQLOrder)>,
    joins: Vec<JoinExprWithOn<'args>>,
}
impl<'args> ExpressionWhereable<'args> for SelectExprBuilder<'args> {
    fn push_where_comparison<L: ExprType<'args> + 'args, R: ExprType<'args> + 'args>(
        &mut self,
        comparison: FilterConditionBuilder<'args, L, R>,
    ) {
        self.where_comparisons.push(comparison.dyn_expression());
    }
}
impl PaginationOwnedSupportingTool for SelectExprBuilder<'_> {
    fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit as i64);
        self
    }

    fn offset(mut self, offset: i32) -> Self {
        self.offset = Some(offset as i64);
        self
    }
}

impl<'args> SelectExprBuilder<'args> {
    pub fn new(table: &'static str) -> Self {
        Self {
            table,
            select: Vec::new(),
            where_comparisons: Vec::new(),
            limit: None,
            offset: None,
            order_by: None,
            joins: Vec::new(),
        }
    }
    pub fn column<C>(mut self, column: C) -> Self
    where
        C: ColumnType + 'static,
    {
        self.select.push(DynExpr::new(column.dyn_column()));
        self
    }
    pub fn select_expr<E>(mut self, expr: E) -> Self
    where
        E: ExprType<'args> + 'args,
    {
        self.select.push(DynExpr::new(expr));
        self
    }
    pub fn order_by<C>(mut self, column: C, order: SQLOrder) -> Self
    where
        C: ColumnType + 'static,
    {
        self.order_by = Some((column.dyn_column(), order));
        self
    }

    pub fn join<F>(mut self, table: &'static str, join_type: JoinType, join: F) -> Self
    where
        F: FnOnce(join::JoinExprBuilder<'args>) -> join::JoinExprWithOn<'args>,
    {
        let builder = join::JoinExprBuilder::new(table, join_type);
        let join = join(builder);

        self.joins.push(join);

        self
    }
}
impl<'args> ExprType<'args> for SelectExprBuilder<'args> {
    fn process(self: Box<Self>, args: &mut ArgumentHolder<'args>) -> super::Expr
    where
        Self: 'args,
    {
        self.process_unboxed(args)
    }

    fn process_unboxed(self, args: &mut ArgumentHolder<'args>) -> super::Expr
    where
        Self: 'args,
    {
        let where_comparisons = self
            .where_comparisons
            .into_iter()
            .map(|filter| filter.process_inner(args))
            .collect();
        let select = self
            .select
            .into_iter()
            .map(|expr| expr.process_unboxed(args))
            .collect();
        let joins = self
            .joins
            .into_iter()
            .map(|join| join.process_args(args))
            .collect();
        let select = SelectExpr {
            table: self.table,
            select,
            where_comparisons,
            limit: self.limit,
            order_by: self.order_by,
            joins,
        };

        Expr::Select(select)
    }
}
impl<'args> Aliasable<'args> for SelectExprBuilder<'args> {}
impl<'args> WrapInFunction<'args> for SelectExprBuilder<'args> {}
#[cfg(test)]
mod tests {
    use crate::{
        fake::FakeQuery,
        pagination::PaginationOwnedSupportingTool,
        prelude::*,
        testing::{AnotherTable, AnotherTableColumn, TestTable, TestTableColumn},
    };

    use super::SelectExprBuilder;

    #[test]
    pub fn basic_select() {
        let sub_select = SelectExprBuilder::new(TestTable::table_name())
            .column(TestTableColumn::Age)
            .select_expr(SqlFunctionBuilder::now())
            .limit(20)
            .array()
            .alias("test_alias");

        // Code for faking the query
        let mut parent = FakeQuery::default();
        let expr = sub_select.process_unboxed(&mut parent.arguments);

        assert_eq!(
            expr.format_sql().into_owned(),
            "ARRAY((SELECT (test_table.age, NOW()) FROM test_table LIMIT 20)) AS test_alias"
        );
    }

    #[test]
    pub fn basic_select_with_join() {
        let sub_select = SelectExprBuilder::new(TestTable::table_name())
            .column(TestTableColumn::Age)
            .select_expr(SqlFunctionBuilder::now())
            .join(AnotherTable::table_name(), JoinType::Inner, |join| {
                join.select(AnotherTableColumn::Age)
                    .on(TestTableColumn::Id.equals(AnotherTableColumn::Id))
            })
            .limit(20)
            .array()
            .alias("test_alias");

        // Code for faking the query
        let mut parent = FakeQuery::default();
        let expr = sub_select.process_unboxed(&mut parent.arguments);

        assert_eq!(
            expr.format_sql().into_owned(),
            "ARRAY((SELECT (test_table.age, NOW(), another_table.age) FROM test_table INNER JOIN another_table ON test_table.id = another_table.id LIMIT 20)) AS test_alias"
        );
    }
}
