//! ExprType for values
use super::{Expr, ExprType, WrapInFunction};
use pg_extended_sqlx_queries_macros::value_expr_type;
use sqlx::{
    Encode, Postgres, Type,
    postgres::{PgHasArrayType, PgTypeInfo},
};
pub mod arguments;
pub use arguments::*;
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
pub trait OptionalDynEncodeType<'args> {
    /// Useful for INSERT queries where they have the insert_optional method
    fn optional_value(self) -> Option<DynEncode<'args>>;
}
impl<'args, T> OptionalDynEncodeType<'args> for Option<T>
where
    T: 'args + Encode<'args, Postgres> + Type<Postgres>,
{
    fn optional_value(self) -> Option<DynEncode<'args>> {
        self.map(|value| DynEncode::new(value))
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
    fn process(self: Box<Self>, args: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        let index = args.push_argument(*self);
        Expr::ArgumentIndex(index)
    }

    fn process_unboxed(self, args: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        let index = args.push_argument(self);
        Expr::ArgumentIndex(index)
    }
}
impl<'args, T> ExprType<'args> for Option<T>
where
    T: 'args + ExprType<'args> + Encode<'args, Postgres> + Type<Postgres>,
{
    fn process(self: Box<Self>, args: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        args.push_argument(*self).into()
    }

    fn process_unboxed(self, args: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        args.push_argument(self).into()
    }
}
impl<'args, T> WrapInFunction<'args> for Option<T> where
    T: 'args + Encode<'args, Postgres> + Type<Postgres> + WrapInFunction<'args>
{
}
impl<'args, T> ExprType<'args> for Vec<T>
where
    T: 'args + ExprType<'args> + Encode<'args, Postgres> + Type<Postgres> + PgHasArrayType,
{
    fn process(self: Box<Self>, args: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        args.push_argument(*self).into()
    }

    fn process_unboxed(self, args: &mut ArgumentHolder<'args>) -> Expr
    where
        Self: 'args,
    {
        args.push_argument(self).into()
    }
}
impl<'args, T> WrapInFunction<'args> for Vec<T> where
    T: 'args + Encode<'args, Postgres> + Type<Postgres> + WrapInFunction<'args> + PgHasArrayType
{
}

// Standard Library types
value_expr_type!(
    bool,
    i8,
    i16,
    i32,
    i64,
    f32,
    f64,
    String,
    Vec<u8>,
    &'args str,
    &'args [u8]
);
#[cfg(feature = "chrono")]
value_expr_type!(chrono::NaiveDateTime, chrono::NaiveTime, chrono::NaiveDate,);
#[cfg(feature = "chrono")]
value_expr_type!(chrono::DateTime<Tz>: where Tz: chrono::TimeZone  + 'args);

#[cfg(feature = "uuid")]
value_expr_type!(uuid::Uuid);

#[cfg(feature = "json")]
value_expr_type!(sqlx::types::Json<T>: where T: serde::Serialize + 'args);

#[cfg(feature = "ipnetwork")]
value_expr_type!(sqlx::types::ipnetwork::IpNetwork, core::net::IpAddr);

#[cfg(feature = "mac_address")]
value_expr_type!(mac_address::MacAddress);
