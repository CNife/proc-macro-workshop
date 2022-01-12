use proc_macro::TokenStream as StdTokenStream;

use proc_macro2::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: StdTokenStream) -> StdTokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let _ = dbg!(input);

    TokenStream::new().into()
}
