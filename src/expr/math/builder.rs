use std::marker::PhantomData;

use crate::prelude::*;

use super::{OneOperandOperations, SQLMathExpr, TwoOperandOperations};

pub(crate) enum MathExprBuilderInner<'args, L: ExprType<'args> + 'args, R: ExprType<'args> + 'args>
{
    TwoOperands {
        left: L,
        operation: TwoOperandOperations,
        right: R,
    },
    OneOperand {
        expression: L,
        operation: OneOperandOperations,
    },
    /// Exists just to make the compiler happy
    #[allow(dead_code)]
    Hidden(PhantomData<&'args ()>),
}
impl<'args, L: ExprType<'args> + 'args, R: ExprType<'args> + 'args>
    MathExprBuilderInner<'args, L, R>
{
    pub(crate) fn dyn_expression(
        self,
    ) -> MathExprBuilderInner<'args, DynExpr<'args>, DynExpr<'args>> {
        match self {
            Self::TwoOperands {
                left,
                operation,
                right,
            } => MathExprBuilderInner::TwoOperands {
                left: DynExpr::new(left),
                operation,
                right: DynExpr::new(right),
            },
            Self::OneOperand {
                expression,
                operation,
            } => MathExprBuilderInner::OneOperand {
                expression: DynExpr::new(expression),
                operation,
            },
            Self::Hidden(_) => unreachable!(),
        }
    }

    pub(crate) fn process_inner(self, args: &mut ArgumentHolder<'args>) -> SQLMathExpr {
        match self {
            MathExprBuilderInner::TwoOperands {
                left,
                operation,
                right,
            } => SQLMathExpr::TwoOperands {
                left: left.process_unboxed(args),
                operation,
                right: right.process_unboxed(args),
            },
            MathExprBuilderInner::OneOperand {
                expression,
                operation,
            } => SQLMathExpr::OneOperand {
                value: expression.process_unboxed(args),
                operation,
            },
            MathExprBuilderInner::Hidden(_) => unreachable!(),
        }
    }
}
