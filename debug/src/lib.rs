use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    let tt = parse_macro_input!(input as DeriveInput);
    eprintln!("{tt:#?}");
    quote!().into()
}
