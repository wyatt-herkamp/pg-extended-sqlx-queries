use crate::expr::ExprType;

use super::{
    MathExprBuilder, OneOperandOperations, TwoOperandOperations, builder::MathExprBuilderInner,
};

macro_rules! operations {
    (two_operand:
        $(
            $name:ident: $value:ident
        ),*
    ) => {
        $(
        fn $name<E>(self, value: E) -> MathExprBuilder<'args, Self, E>
        where
            Self: Sized + 'args,
            E: ExprType<'args> + 'args,
        {
            self.two_operands(TwoOperandOperations::$value, value)
        }
        )*
    };
    (one_operand:
        $(
            $name:ident: $value:ident
        ),*
    ) => {
        $(
        fn $name(self) -> MathExprBuilder<'args, Self, ()>
        where
            Self: Sized + 'args,
        {
            self.one_operand(OneOperandOperations::$value)
        }
        )*
    };
}
pub trait MathExpr<'args>: ExprType<'args> {
    operations!(two_operand:
        add: Addition,
        subtract: Subtraction,
        multiply: Multiplication,
        divide: Division,
        modulo: Modulus,
        pow: Power,
        bit_and: BitwiseAnd,
        bit_or: BitwiseOr,
        bit_xor: BitwiseXor,
        bit_shift_left: BitwiseShiftLeft,
        bit_shift_right: BitwiseShiftRight
    );
    operations!(one_operand:
        sqrt: SquareRoot,
        cube_root: CubeRoot,
        neg: Negate,
        unary_plus: UnaryPlus,
        bit_not: BitWiseNot
    );
    /// Compares the current Self with a provided Binary Comparison to the right
    fn two_operands<L>(
        self,
        operation: TwoOperandOperations,
        value: L,
    ) -> MathExprBuilder<'args, Self, L>
    where
        Self: Sized + 'args,
        L: ExprType<'args> + 'args,
    {
        MathExprBuilderInner::TwoOperands {
            left: self,
            operation,
            right: value,
        }
        .into()
    }
    fn one_operand(self, operation: OneOperandOperations) -> MathExprBuilder<'args, Self, ()>
    where
        Self: Sized + 'args,
    {
        MathExprBuilderInner::OneOperand {
            expression: self,
            operation,
        }
        .into()
    }
}

impl<'args, E> MathExpr<'args> for E where E: ExprType<'args> {}
