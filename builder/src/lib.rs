use proc_macro::TokenStream as StdTokenStream;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: StdTokenStream) -> StdTokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let result = make_derive_builder(dbg!(derive_input)).unwrap_or_else(|e| e.to_compile_error());
    eprintln!("{result}");
    result.into()
}

type TokenStreamResult = syn::Result<TokenStream>;

fn make_derive_builder(input: DeriveInput) -> TokenStreamResult {
    Ok(quote! {})
}
