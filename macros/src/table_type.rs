use column::ColumnField;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
pub mod column;
pub mod table;
use syn::{Data, DeriveInput, Result};
use table::TableAttr;

pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput {
        ident, data, attrs, ..
    } = input;
    let column_enum_name = format_ident!("{}Column", ident);
    let Data::Struct(data_struct) = data else {
        return Err(syn::Error::new_spanned(ident, "expected struct"));
    };
    let table_attr = attrs
        .iter()
        .find(|attr| attr.path().is_ident("table"))
        .map(|attr| {
            attr.parse_args::<table::TableAttr>()
                .map_err(|err| syn::Error::new_spanned(attr, err))
        })
        .ok_or_else(|| syn::Error::new_spanned(&ident, "table attribute is required"))??;
    let fields = match data_struct.fields {
        syn::Fields::Named(fields_named) => fields_named
            .named
            .into_iter()
            .filter_map(|field| ColumnField::new(field, ident.clone()).transpose())
            .collect::<Result<Vec<_>>>()?,
        _ => return Err(syn::Error::new_spanned(ident, "expected named fields")),
    };
    let enum_variants: Vec<_> = fields
        .iter()
        .map(|field| field.enum_variant_def())
        .collect();
    let display_match_arms: Vec<_> = fields
        .iter()
        .map(|field| field.display_match_arm())
        .collect();
    let column_type_match_arms: Vec<_> = fields
        .iter()
        .map(|field| field.column_type_name())
        .collect();
    let formatted_column: Vec<_> = fields
        .iter()
        .map(|field| field.formatted_column(&table_attr.name))
        .collect();
    let column_type_all: Vec<_> = fields.iter().map(|field| field.column_type_all()).collect();
    let TableAttr { name: table_name } = table_attr;
    let result = quote! {
        impl TableType for #ident {
            type Columns = #column_enum_name;
            fn table_name() -> &'static str {
                #table_name
            }
        }
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum #column_enum_name {
            #(
                #enum_variants
            ),*
        }
        const _: () = {
            impl std::fmt::Display for #column_enum_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(
                            #display_match_arms
                        ),*
                    }
                }
            }

            impl ColumnType for #column_enum_name {
                fn column_name(&self) -> &'static str {
                    match self {
                        #(
                            #column_type_match_arms
                        ),*
                    }
                }
                fn formatted_column(&self) -> std::borrow::Cow<'static, str> {
                    match self {
                        #(
                            #formatted_column
                        ),*
                    }
                }
            }
            impl FormatSql for #column_enum_name{
                fn format_sql(&self) -> std::borrow::Cow<'_, str> {
                    self.formatted_column()
                }
            }
            impl AllColumns for #column_enum_name {
                fn all() -> std::vec::Vec<Self>
                    where
                        Self: Sized {
                    std::vec![
                        #(
                            #column_type_all
                        ),*
                    ]
                }
            }
        };

    };

    Ok(result)
}
