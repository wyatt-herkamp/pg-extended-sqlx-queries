use proc_macro::TokenStream;
use syn::parse::Parse;
pub(crate) mod table_type;
pub(crate) mod utils;
pub(crate) mod value_expr_type;
/// Derive macro for generating table types
/// ```rust,ignore
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
#[proc_macro_derive(ValueExprType)]
pub fn derive_value_expr_type(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    match value_expr_type::derive::expand(input) {
        Ok(result) => result.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn value_expr_type(input: TokenStream) -> TokenStream {
    let types = syn::parse_macro_input!(input with value_expr_type::ValueExprTypes::parse);

    match types.expand() {
        Ok(result) => result.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
