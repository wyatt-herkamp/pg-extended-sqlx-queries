use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Field, Ident, LitStr, Result,
};

use crate::utils::ident_to_upper_camel;

use crate::utils::keywords;
#[derive(Debug, Default)]
pub struct ColumnAttribute {
    /// Forces the Column to be named as the given string
    ///
    /// Instead of using the field's ident, the column will be named as the given string
    pub column_name: Option<LitStr>,
    /// Forces the Columns Enum Variant to be named as the given ident
    ///
    /// Instead of using the fields ident to be converted to UpperCamelCase
    pub enum_variant: Option<Ident>,
    /// Skips for whatever reason
    pub skip: bool,
}
impl Parse for ColumnAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut column_name: Option<LitStr> = None;
        let mut enum_variant: Option<Ident> = None;
        let mut skip = false;
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(keywords::name) {
                let _: keywords::name = input.parse()?;
                let _: syn::Token![=] = input.parse()?;
                column_name = Some(input.parse()?);
            } else if lookahead.peek(keywords::enum_ident) {
                let _: keywords::enum_ident = input.parse()?;
                let _: syn::Token![=] = input.parse()?;
                enum_variant = Some(input.parse()?);
            } else if lookahead.peek(keywords::skip) {
                let _: keywords::skip = input.parse()?;
                skip = true;
            } else {
                return Err(lookahead.error());
            }
        }
        Ok(Self {
            column_name,
            enum_variant,
            skip,
        })
    }
}

#[derive(Debug)]
pub struct ColumnField {
    pub struct_name: syn::Ident,
    pub ident: syn::Ident,
    pub name: syn::LitStr,
    pub ident_as_upper_camel: syn::Ident,
}

impl ColumnField {
    pub fn new(field: Field, struct_name: Ident) -> Result<Option<Self>> {
        let ident = field
            .ident
            .ok_or_else(|| syn::Error::new_spanned(field.ty, "expected named field"))?;

        let column_attr = field
            .attrs
            .iter()
            .find_map(|attr| {
                if attr.path().is_ident("column") {
                    Some(attr.parse_args::<ColumnAttribute>())
                } else {
                    None
                }
            })
            .transpose()?
            .unwrap_or_default();
        if column_attr.skip {
            return Ok(None);
        }
        let name = if let Some(name) = column_attr.column_name {
            name
        } else {
            LitStr::new(&ident.to_string(), ident.span())
        };

        let ident_as_upper_camel = if let Some(ident) = column_attr.enum_variant {
            ident
        } else {
            ident_to_upper_camel(&ident)
        };
        let result = Self {
            struct_name,
            ident,
            name,
            ident_as_upper_camel,
        };
        Ok(Some(result))
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
