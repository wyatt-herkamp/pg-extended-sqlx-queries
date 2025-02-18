pub mod delete;
pub mod expr;
#[doc(hidden)]
pub mod fake;
pub mod insert;
pub mod pagination;
pub mod select;
pub mod table_layout;
pub mod traits;
pub mod update;

/// Derive macro for generating table types
/// ```
/// use pg_extended_sqlx_queries::prelude::*;
/// #[derive(TableType)]
/// #[table(name = "users")]
/// pub struct User {
///     pub id: i32,
///     pub name: String,
/// }
/// ```
pub use pg_extended_sqlx_queries_macros::TableType;
use prelude::ColumnType;

#[cfg(test)]
pub(crate) mod testing;

pub mod prelude {
    pub use crate::delete::DeleteQueryBuilder;
    pub use crate::expr::{ArgumentHolder, ArgumentIndex, HasArguments};

    pub use crate::expr::*;
    pub use crate::insert::{many::*, InsertQueryBuilder};
    pub use crate::pagination::*;
    pub use crate::select::*;
    pub use crate::table_layout::{AllColumns, ColumnType, DynColumn, TableQuery, TableType};
    pub use crate::traits::*;

    pub use crate::update::UpdateQueryBuilder;
    pub use pg_extended_sqlx_queries_macros::TableType;
}
