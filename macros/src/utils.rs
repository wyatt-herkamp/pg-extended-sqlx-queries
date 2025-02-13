use heck::ToUpperCamelCase;
use quote::format_ident;

pub fn ident_to_upper_camel(ident: &syn::Ident) -> syn::Ident {
    let ident = ident.to_string().to_upper_camel_case();
    format_ident!("{}", ident)
}
