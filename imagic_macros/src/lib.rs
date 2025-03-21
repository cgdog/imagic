mod ecs;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

#[proc_macro]
pub fn batch_process_tuples(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as crate::ecs::TupleItemRangeInfo);
    let len = input.end - input.start;
    let mut ident_tuples = Vec::with_capacity(len);
    for i in input.start..=input.end {
        let ident = format_ident!("{}{}", input.generic_type_id, i);
        ident_tuples.push(quote! {
            #ident
        });
    }

    let macro_ident = &input.macro_name;
    let invocations = (input.start..=input.end).map(|i| {
        let ident_tuples = &ident_tuples[..i];
        quote! {
            #macro_ident!(#(#ident_tuples),*);
        }
    });
    TokenStream::from(quote! {
        #(
            #invocations
        )*
    })
}
