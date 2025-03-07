use builder::MathExprBuilderInner;

use crate::traits::FormatSql;

use super::{DynExpr, Expr, ExprType};
mod builder;
mod expr;
pub use expr::*;
pub struct MathExprBuilder<'args, L: ExprType<'args> + 'args, R: ExprType<'args> + 'args>(
    MathExprBuilderInner<'args, L, R>,
);
impl<'args, L: ExprType<'args> + 'args, R: ExprType<'args> + 'args>
    From<MathExprBuilderInner<'args, L, R>> for MathExprBuilder<'args, L, R>
{
    fn from(value: MathExprBuilderInner<'args, L, R>) -> Self {
        Self(value)
    }
}
impl<'args, L: ExprType<'args> + 'args, R: ExprType<'args> + 'args> MathExprBuilder<'args, L, R> {
    pub fn dyn_expression(self) -> MathExprBuilder<'args, DynExpr<'args>, DynExpr<'args>> {
        self.0.dyn_expression().into()
    }


}
impl<'args, L: ExprType<'args> + 'args, R: ExprType<'args> + 'args> ExprType<'args>
    for MathExprBuilder<'args, L, R>
{
    fn process(self: Box<Self>, args: &mut super::ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        self.0.process_inner(args).into()
    }

    fn process_unboxed(self, args: &mut super::ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        self.0.process_inner(args).into()
    }
}
#[derive(Debug, PartialEq)]
pub enum SQLMathExpr {
    TwoOperands {
        left: Expr,
        operation: TwoOperandOperations,
        right: Expr,
    },
    OneOperand {
        operation: OneOperandOperations,
        value: Expr,
    },
}
impl FormatSql for SQLMathExpr {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        match self {
            SQLMathExpr::TwoOperands {
                left,
                operation,
                right,
            } => format!(
                "({} {} {})",
                left.format_sql(),
                operation.format_sql(),
                right.format_sql()
            )
            .into(),
            SQLMathExpr::OneOperand { operation, value } => {
                format!("({} {})", operation.format_sql(), value.format_sql()).into()
            }
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TwoOperandOperations {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Modulus,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseShiftLeft,
    BitwiseShiftRight,
    Power,
}
impl FormatSql for TwoOperandOperations {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        match self {
            TwoOperandOperations::Addition => "+".into(),
            TwoOperandOperations::Subtraction => "-".into(),
            TwoOperandOperations::Multiplication => "*".into(),
            TwoOperandOperations::Division => "/".into(),
            TwoOperandOperations::Modulus => "%".into(),
            TwoOperandOperations::BitwiseAnd => "&".into(),
            TwoOperandOperations::BitwiseOr => "|".into(),
            TwoOperandOperations::BitwiseXor => "#".into(),
            TwoOperandOperations::BitwiseShiftLeft => "<<".into(),
            TwoOperandOperations::BitwiseShiftRight => ">>".into(),
            TwoOperandOperations::Power => "^".into(),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OneOperandOperations {
    SquareRoot,
    CubeRoot,
    Negate,
    UnaryPlus,
    BitWiseNot,
}
impl FormatSql for OneOperandOperations {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        match self {
            OneOperandOperations::SquareRoot => "|/".into(),
            OneOperandOperations::CubeRoot => "||/".into(),
            OneOperandOperations::Negate => "-".into(),
            OneOperandOperations::UnaryPlus => "+".into(),
            OneOperandOperations::BitWiseNot => "~".into(),
        }
    }
}
