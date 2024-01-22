use proc_macro::TokenStream as StdTokenStream;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Field, Fields, Meta, MetaNameValue};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: StdTokenStream) -> StdTokenStream {
    let DeriveInput { ident, data, .. } = dbg!(parse_macro_input!(input as DeriveInput));
    let fields_code: TokenStream = match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields
            .named
            .into_iter()
            .map(|field| {
                let Field { ident, attrs, .. } = field;
                let field_ident = ident.unwrap();
                let field_value_code = attrs
                    .into_iter()
                    .find_map(|attr| match attr.meta {
                        Meta::NameValue(MetaNameValue { path, value, .. })
                            if path.is_ident("debug") =>
                        {
                            Some(value)
                        }
                        _ => None,
                    })
                    .map_or_else(
                        || quote!(&self.#field_ident),
                        |format| quote!(&format_args!(#format, &self.#field_ident)),
                    );
                quote!(.field(stringify!(#field_ident), #field_value_code))
            })
            .collect(),
        _ => quote!(),
    };

    let result = quote! {
        impl ::std::fmt::Debug for #ident {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.debug_struct(stringify!(#ident))
                #fields_code
                .finish()
            }
        }
    };

    eprintln!("{result}");
    result.into()
}
