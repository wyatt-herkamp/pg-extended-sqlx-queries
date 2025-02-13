use proc_macro::TokenStream;
pub(crate) mod table_type;
pub(crate) mod utils;
#[proc_macro_derive(TableType, attributes(column, table))]
pub fn columns(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    match table_type::expand(input) {
        Ok(result) => result.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
