use syn::{Ident, parse::{Parse, ParseStream}, token::Comma, LitInt};

pub(crate) struct TupleItemRangeInfo {
    pub macro_name: Ident,
    pub start: usize,
    pub end: usize,
    pub generic_type_id: Ident,
}

impl Parse for TupleItemRangeInfo {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let macro_ident = input.parse::<Ident>()?;
        input.parse::<Comma>()?;
        let start = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<Comma>()?;
        let end = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<Comma>()?;
        let ident =input.parse::<Ident>()?;

        Ok(TupleItemRangeInfo {
            macro_name: macro_ident,
            start,
            end,
            generic_type_id: ident,
        })
    }
}