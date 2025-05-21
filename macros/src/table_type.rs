use column::{ColumnField, ColumnTypeAttribute};
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

    let column_type_attr: ColumnTypeAttribute = attrs
        .iter()
        .find(|attr| attr.path().is_ident("column"))
        .map(|attr| {
            attr.parse_args::<ColumnTypeAttribute>()
                .map_err(|err| syn::Error::new_spanned(attr, err))
        })
        .transpose()?
        .unwrap_or_default();
    let column_enum_name: syn::Ident = if let Some(enum_ident) = column_type_attr.enum_ident {
        enum_ident
    } else {
        format_ident!("{}Column", ident)
    };
    let fields = match data_struct.fields {
        syn::Fields::Named(fields_named) => fields_named
            .named
            .into_iter()
            .filter_map(|field| ColumnField::new(field, ident.clone()).transpose())
            .collect::<Result<Vec<_>>>()?,
        _ => return Err(syn::Error::new_spanned(ident, "expected named fields")),
    };
    let primary_key = fields
        .iter()
        .find(|field| field.primary_key)
        .map(|field| has_primary_key(field, &ident, &column_enum_name))
        .unwrap_or_default();
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
    let expr_type = expr_type(column_type_attr.impl_expr, &column_enum_name);
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
                fn table_name(&self) -> &'static str {
                    <#ident as TableType>::table_name()
                }
                fn full_name(&self) -> std::borrow::Cow<'static, str> {
                    match self {
                        #(
                            #formatted_column
                        ),*
                    }
                }
            }
            impl From<#column_enum_name> for DynColumn{
                fn from(column: #column_enum_name) -> Self {
                    DynColumn::new(column)
                }
            }
            impl From<#column_enum_name> for Expr{
                fn from(column: #column_enum_name) -> Self {
                    Expr::Column(DynColumn::new(column))
                }
            }
            /// Implement the `FormatSql` trait for the column enum
            ///
            /// Will just call ColumnType::full_name
            impl FormatSql for #column_enum_name{
                #[inline]
                fn format_sql(&self) -> std::borrow::Cow<'_, str> {
                    self.full_name()
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
            #primary_key
        };
        #expr_type

    };

    Ok(result)
}
fn has_primary_key(
    field: &ColumnField,
    table_type: &syn::Ident,
    column_enum_name: &syn::Ident,
) -> TokenStream {
    let column_name = &field.ident_as_upper_camel;
    quote! {
        impl HasPrimaryKey for #table_type {
            fn primary_key() -> Self::Columns {
                #column_enum_name::#column_name
            }
        }
    }
}
fn expr_type(implement: bool, column_enum_name: &syn::Ident) -> TokenStream {
    if !implement {
        return TokenStream::default();
    }
    quote! {
        const _: () = {
            impl<'args> ExprType<'args> for #column_enum_name {
                #[inline]
                fn process(self: Box<Self>, _: &mut ArgumentHolder<'args>) -> Expr
                    where
                        Self: 'args,
                    {
                        (*self).into()
                    }
                    #[inline]
                    fn process_unboxed(self, _: &mut ArgumentHolder<'args>) -> Expr
                        where
                            Self: 'args,
                    {
                        self.into()
                    }
            }
        };
    }
}
