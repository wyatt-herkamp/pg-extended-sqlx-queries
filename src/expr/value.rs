use sqlx::{postgres::PgTypeInfo, Encode, Postgres, Type};

use super::{ExprType, WrapInFunction};

pub trait DynEncodeType<'args> {
    fn value(self) -> DynEncode<'args>;
}
impl<'args, T> DynEncodeType<'args> for T
where
    T: 'args + Encode<'args, Postgres> + Type<Postgres>,
{
    fn value(self) -> DynEncode<'args> {
        DynEncode::new(self)
    }
}
impl<'args> WrapInFunction<'args> for DynEncode<'args> {}
pub struct DynEncode<'args> {
    value: Box<dyn sqlx::Encode<'args, Postgres> + 'args>,
    type_info: PgTypeInfo,
}

impl<'args> DynEncode<'args> {
    pub fn new<T>(value: T) -> Self
    where
        T: 'args + sqlx::Encode<'args, Postgres> + sqlx::Type<Postgres>,
    {
        let type_info = value.produces().unwrap_or_else(T::type_info);
        Self {
            value: Box::new(value),
            type_info: type_info,
        }
    }
}
impl<'args> Encode<'args, Postgres> for DynEncode<'args> {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'args>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        self.value.encode_by_ref(buf)
    }
    fn produces(&self) -> Option<<Postgres as sqlx::Database>::TypeInfo> {
        Some(self.type_info.clone())
    }
}
impl<'args> Type<Postgres> for DynEncode<'args> {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        panic!("DynEncode does not have a static type info")
    }
}
impl<'args> ExprType<'args> for DynEncode<'args> {
    fn process(self: Box<Self>, args: &mut dyn crate::HasArguments<'args>) -> super::Expr
    where
        Self: 'args,
    {
        let index = args.push_dyn_argument(*self);
        super::Expr::ArgumentIndex(super::ArgumentIndex(index))
    }

    fn process_unboxed(self, args: &mut dyn crate::HasArguments<'args>) -> super::Expr
    where
        Self: 'args,
    {
        let index = args.push_dyn_argument(self);
        super::Expr::ArgumentIndex(super::ArgumentIndex(index))
    }
}
