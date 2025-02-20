use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, DeriveInput, Result};

use super::ValueExprType;
pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput { ident, .. } = input;
    // Convert ident to a syn::Type
    let type_to_implement = parse_quote! {
        #ident
    };
    let value_expr_type = ValueExprType {
        type_to_implement,
        where_clause: None,
    };
    let result = quote! {
        const _: () = {
            #value_expr_type
        };
    };

    Ok(result)
}
