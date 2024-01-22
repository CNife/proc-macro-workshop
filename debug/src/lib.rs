use proc_macro::TokenStream as StdTokenStream;

use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: StdTokenStream) -> StdTokenStream {
    let DeriveInput { ident, data, .. } = dbg!(parse_macro_input!(input as DeriveInput));
    let field_idents: Vec<_> = if let Data::Struct(DataStruct {
        fields: Fields::Named(fields),
        ..
    }) = data
    {
        fields.named.into_iter().map(|f| f.ident.unwrap()).collect()
    } else {
        todo!()
    };

    let result = quote! {
        impl ::std::fmt::Debug for #ident {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.debug_struct(stringify!(#ident))
                #( .field(stringify!(#field_idents), &self.#field_idents) )*
                .finish()
            }
        }
    };

    eprintln!("{result}");
    result.into()
}
