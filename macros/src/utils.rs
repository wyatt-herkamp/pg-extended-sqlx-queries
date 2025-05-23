use heck::ToUpperCamelCase;
use syn::spanned::Spanned;

pub fn ident_to_upper_camel(ident: &syn::Ident) -> syn::Ident {
    let ident = ident.to_string().to_upper_camel_case();
    syn::Ident::new(&ident, ident.span())
}
pub mod keywords {
    syn::custom_keyword!(name);
    syn::custom_keyword!(skip);
    syn::custom_keyword!(column_variant_name);
    syn::custom_keyword!(column_enum_name);
    syn::custom_keyword!(impl_expr);
    syn::custom_keyword!(primary_key);
}
