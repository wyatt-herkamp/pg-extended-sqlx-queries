use std::borrow::Cow;

use crate::expr::{Expr, ExprType, HasArguments, SQLCondition};

use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    FullOuter,
    LeftOuter,
    RightOuter,
}
impl FormatSql for JoinType {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        match self {
            JoinType::Inner => "INNER JOIN".into(),
            JoinType::Left => "LEFT JOIN".into(),
            JoinType::Right => "RIGHT JOIN".into(),
            JoinType::Full => "FULL JOIN".into(),
            JoinType::LeftOuter => "LEFT OUTER JOIN".into(),
            JoinType::RightOuter => "RIGHT OUTER JOIN".into(),
            JoinType::FullOuter => "FULL OUTER JOIN".into(),
        }
    }
}
#[derive(Debug)]
pub struct OnCondition {
    left: Expr,
    value: SQLCondition,
    then: Option<(AndOr, Box<OnCondition>)>,
}
impl FormatSql for OnCondition {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        let mut sql = self.left.format_sql().into_owned();
        sql.push_str(" ");
        sql.push_str(&self.value.format_sql());
        if let Some((and_or, then)) = &self.then {
            sql.push(' ');
            sql.push_str(&and_or.format_sql());
            sql.push(' ');
            sql.push_str(&then.format_sql());
        }
        Cow::Owned(sql)
    }
}
#[derive(Debug)]
pub struct Join {
    pub join_type: JoinType,
    pub table: &'static str,
    pub on: SQLCondition,
    pub columns_to_select: Vec<Expr>,
}
impl FormatSql for Join {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        let mut sql = format!("{} {}", self.join_type.format_sql(), self.table);
        sql.push_str(" ON ");
        sql.push_str(&self.on.format_sql());

        Cow::Owned(sql)
    }
}
pub struct JoinBuilder<'query, 'args, A>
where
    A: HasArguments<'args>,
{
    join_type: JoinType,
    table: &'static str,
    args: &'query mut A,
    select: Vec<Expr>,

    phantoms: std::marker::PhantomData<&'args ()>,
}
impl<'query, 'args, A> JoinBuilder<'query, 'args, A>
where
    A: HasArguments<'args>,
{
    pub fn new(args: &'query mut A, table_name: &'static str, join: JoinType) -> Self {
        Self {
            args,
            join_type: join,
            table: table_name,
            phantoms: std::marker::PhantomData,
            select: Vec::new(),
        }
    }
    pub fn select<E>(mut self, expr: E) -> Self
    where
        E: ExprType<'args> + 'args,
    {
        let expr = expr.process_unboxed(&mut self.args.holder());
        self.select.push(expr);
        self
    }
    pub fn select_many<E>(mut self, columns: Vec<E>) -> Self
    where
        E: ExprType<'args> + 'args,
    {
        let expr: Vec<_> = columns
            .into_iter()
            .map(|expr| expr.process_unboxed(&mut self.args.holder()))
            .collect();

        self.select.extend(expr);
        self
    }
    pub fn on<L: ExprType<'args> + 'args, R: ExprType<'args> + 'args>(
        self,
        condition: FilterConditionBuilder<'args, L, R>,
    ) -> Join {
        let on = condition.process_inner(&mut self.args.holder());
        Join {
            join_type: self.join_type,
            table: self.table,
            on,
            columns_to_select: self.select,
        }
    }
}
