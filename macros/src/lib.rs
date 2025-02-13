use proc_macro::TokenStream;
pub(crate) mod table_type;
pub(crate) mod utils;
/// Derive macro for generating table types
/// ```rust,no_compile
/// use pg_extended_sqlx_queries_macros::TableType;
/// #[derive(TableType)]
/// #[table(name = "users")]
/// pub struct User {
///     pub id: i32,
///     pub name: String,
/// }
/// ```
#[proc_macro_derive(TableType, attributes(column, table))]
pub fn table_type(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    match table_type::expand(input) {
        Ok(result) => result.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
