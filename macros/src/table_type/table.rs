use syn::{
    parse::{Parse, ParseStream},
    LitStr, Result,
};

use crate::utils::keywords;

pub struct TableAttr {
    pub name: LitStr,
}

impl Parse for TableAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut table_name: Option<LitStr> = None;
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(keywords::name) {
                let _: keywords::name = input.parse()?;
                let _: syn::Token![=] = input.parse()?;
                table_name = Some(input.parse()?);
            } else {
                return Err(lookahead.error());
            }
        }
        let name =
            table_name.ok_or_else(|| syn::Error::new(input.span(), "table name is required"))?;
        Ok(Self { name: name })
    }
}
