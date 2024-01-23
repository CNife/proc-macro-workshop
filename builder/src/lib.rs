use proc_macro::TokenStream as StdTokenStream;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Type};

#[proc_macro_derive(Builder)]
pub fn derive(input: StdTokenStream) -> StdTokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let result = make_derive_builder(dbg!(derive_input)).unwrap_or_else(|e| e.to_compile_error());
    eprintln!("{result}");
    result.into()
}

type TokenStreamResult = syn::Result<TokenStream>;

fn make_derive_builder(input: DeriveInput) -> TokenStreamResult {
    let DeriveInput { ident, data, .. } = input;

    let field_ident_types: Vec<(Ident, Type)> = if let Data::Struct(DataStruct {
        fields: Fields::Named(fields),
        ..
    }) = data
    {
        fields
            .named
            .into_iter()
            .map(|field| (field.ident.unwrap(), field.ty))
            .collect()
    } else {
        return Err(syn::Error::new_spanned(
            &ident,
            "only named struct is supported",
        ));
    };

    let builder_struct_ident = format_ident!("{ident}Builder");
    let builder_field_declarations = field_ident_types
        .iter()
        .map(|(ident, ty)| quote!(#ident: std::option::Option<#ty>,))
        .collect::<TokenStream>();
    let builder_field_inits = field_ident_types
        .iter()
        .map(|(ident, ty)| quote!(#ident: None,))
        .collect::<TokenStream>();

    Ok(quote! {
        struct #builder_struct_ident {
            #builder_field_declarations
        }

        impl #ident {
            pub fn builder() -> #builder_struct_ident {
                #builder_struct_ident {
                    #builder_field_inits
                }
            }
        }
    })
}
