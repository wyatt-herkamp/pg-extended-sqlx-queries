use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Token,
    parse::{Parse, ParseStream, Result},
};
pub mod derive;
#[derive(Debug)]
pub struct ValueExprType {
    pub type_to_implement: syn::Type,
    pub where_clause: Option<syn::WhereClause>,
}
impl ValueExprType {
    pub fn expand(&self) -> Result<proc_macro2::TokenStream> {
        let result = quote! {
                #self
        };
        Ok(result)
    }
}
impl ToTokens for ValueExprType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let impl_part = if let Some(where_clause) = &self.where_clause {
            let mut generic_names = Vec::new();
            for predicate in &where_clause.predicates {
                if let syn::WherePredicate::Type(t) = predicate {
                    generic_names.push(t.bounded_ty.clone());
                }
            }
            quote! {
                <'args, #(#generic_names),*>
            }
        } else {
            quote! {
                <'args>
            }
        };
        let where_clause_tokens = self
            .where_clause
            .as_ref()
            .map(|where_clause| {
                let where_clause = where_clause.clone();
                quote! { #where_clause }
            })
            .unwrap_or_default();
        let type_to_implement = &self.type_to_implement;
        let base_type = quote! { #type_to_implement };
        let ref_type = quote! { &'args #type_to_implement };

        let impl_base = expanded(&impl_part, &base_type, &where_clause_tokens);
        let impl_ref = expanded(&impl_part, &ref_type, &where_clause_tokens);
        tokens.extend(impl_base);

        tokens.extend(impl_ref);
    }
}

fn expanded(
    impl_part: &TokenStream,
    type_to_implement: &TokenStream,
    where_clause_tokens: &TokenStream,
) -> TokenStream {
    let result = quote! {
            impl #impl_part ExprType<'args> for #type_to_implement #where_clause_tokens{
                fn process(self: Box<Self>, args: &mut ArgumentHolder<'args>) -> Expr
                where
                    Self: 'args,
                {
                    Expr::ArgumentIndex(args.push_argument(*self))
                }

                fn process_unboxed(self, args: &mut ArgumentHolder<'args>) -> Expr
                where
                    Self: 'args,
                {
                    Expr::ArgumentIndex(args.push_argument(self))
                }
            }
            impl #impl_part WrapInFunction<'args> for #type_to_implement #where_clause_tokens {}

    };
    result
}
impl Parse for ValueExprType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let type_to_implement: syn::Type = input.parse()?;
        let where_clause = if input.peek(Token![:]) {
            let _: Token![:] = input.parse()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self {
            type_to_implement,
            where_clause,
        })
    }
}
/// value_expr_type!{
///    i32,
///    i64,
///    f32,
/// }
#[derive(Debug)]
pub struct ValueExprTypes {
    pub values: Vec<ValueExprType>,
}
impl ValueExprTypes {
    pub fn expand(&self) -> Result<proc_macro2::TokenStream> {
        let mut result = TokenStream::new();
        for value in &self.values {
            let expanded = value.expand()?;
            result.extend(expanded);
        }
        Ok(result)
    }
}
impl Parse for ValueExprTypes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut values = Vec::new();
        while !input.is_empty() {
            values.push(input.parse()?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Self { values })
    }
}

#[cfg(test)]
mod tests {

    use syn::Macro;

    use crate::value_expr_type::ValueExprTypes;

    #[test]
    fn test() {
        let input = r#"
        value_expr_type!{
            sqlx::types::Json<T>: where T: serde::Serialize
        }
        "#;

        let macro_input = syn::parse_str::<Macro>(input).unwrap();
        let value: ValueExprTypes = macro_input.parse_body().unwrap();
        for value in value.values {
            let value = value.expand().unwrap().to_string();
            println!("{}", value);
            let syn_file = syn::parse_file(&value).unwrap();
            let prettyplease = prettyplease::unparse(&syn_file);
            println!("{}", prettyplease);
        }
    }
}
