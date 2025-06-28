
use crate::expr::ExprType;

use crate::prelude::*;

pub struct JoinExprWithOn<'args> {
    join_type: JoinType,
    table: &'static str,
    select: Vec<DynExpr<'args>>,
    on: FilterConditionBuilder<'args, DynExpr<'args>, DynExpr<'args>>,
}
impl<'args> JoinExprWithOn<'args> {
    pub fn process_args(self, args: &mut ArgumentHolder<'args>) -> Join {
        let on = self.on.process_inner(args);
        let select = self
            .select
            .into_iter()
            .map(|expr| expr.process_unboxed(args))
            .collect::<Vec<_>>();
        Join {
            join_type: self.join_type,
            table: self.table,
            on,
            columns_to_select: select,
        }
    }
}
pub struct JoinExprBuilder<'args> {
    join_type: JoinType,
    table: &'static str,
    select: Vec<DynExpr<'args>>,
}
impl<'args> JoinExprBuilder<'args> {
    pub fn new(table: &'static str, join_type: JoinType) -> Self {
        Self {
            join_type,
            table,
            select: Vec::new(),
        }
    }
    pub fn select<E>(mut self, expr: E) -> Self
    where
        E: ExprType<'args> + 'args,
    {
        self.select.push(DynExpr::new(expr));
        self
    }
    pub fn select_many<E>(mut self, columns: Vec<E>) -> Self
    where
        E: ExprType<'args> + 'args,
    {
        for column in columns {
            self.select.push(DynExpr::new(column));
        }
        self
    }
    pub fn on<L: ExprType<'args> + 'args, R: ExprType<'args> + 'args>(
        self,
        condition: FilterConditionBuilder<'args, L, R>,
    ) -> JoinExprWithOn<'args> {
        JoinExprWithOn {
            join_type: self.join_type,
            table: self.table,
            select: self.select,
            on: condition.dyn_expression(),
        }
    }
}
