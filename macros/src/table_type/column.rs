use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, Ident, LitStr, Result};

use crate::utils::ident_to_upper_camel;

#[derive(Debug)]
pub struct ColumnField {
    pub struct_name: syn::Ident,
    pub ident: syn::Ident,
    pub name: syn::LitStr,
    pub ident_as_upper_camel: syn::Ident,
}

impl ColumnField {
    pub fn new(field: Field, struct_name: Ident) -> Result<Self> {
        let ident = field
            .ident
            .ok_or_else(|| syn::Error::new_spanned(field.ty, "expected named field"))?;
        let name = LitStr::new(ident.to_string().as_str(), ident.span());

        let ident_as_upper_camel = ident_to_upper_camel(&ident);
        Ok(Self {
            struct_name,
            ident,
            name,
            ident_as_upper_camel,
        })
    }
    pub fn enum_variant_def(&self) -> TokenStream {
        let doc_str = format!("Corresponds to  [{}::{}].", self.struct_name, self.ident);
        let doc_lit = LitStr::new(doc_str.as_str(), self.ident.span());
        let ident = &self.ident_as_upper_camel;
        quote! {
            #[doc = #doc_lit]
            #ident
        }
    }
    pub fn formatted_column(&self, table_name: &LitStr) -> TokenStream {
        let ident = &self.ident_as_upper_camel;
        let name = &self.name;
        quote! {
            Self::#ident => std::borrow::Cow::Borrowed(concat!(#table_name,".", #name))
        }
    }
    pub fn display_match_arm(&self) -> TokenStream {
        let ident = &self.ident_as_upper_camel;
        let name = &self.name;
        quote! {
            Self::#ident => f.write_str(#name)
        }
    }
    pub fn column_type_name(&self) -> TokenStream {
        let ident = &self.ident_as_upper_camel;
        let name = &self.name;
        quote! {
            Self::#ident => #name
        }
    }
    pub fn column_type_all(&self) -> TokenStream {
        let ident = &self.ident_as_upper_camel;
        quote! {
            Self::#ident
        }
    }
}
