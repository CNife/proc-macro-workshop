use proc_macro::TokenStream as StdTokenStream;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

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

    let fields = if let Data::Struct(DataStruct {
        fields: Fields::Named(fields),
        ..
    }) = data
    {
        fields.named
        // .into_iter()
        // .map(|field| (field.ident.unwrap(), field.ty))
        // .collect()
    } else {
        return Err(syn::Error::new_spanned(
            &ident,
            "only named struct is supported",
        ));
    };

    let builder_struct_ident = format_ident!("{ident}Builder");

    let mut builder_field_declarations = TokenStream::new();
    let mut builder_field_inits = TokenStream::new();
    let mut builder_setters = TokenStream::new();
    for field in fields {
        let field_ident = &field.ident.unwrap();
        let field_type = &field.ty;
        builder_field_declarations.extend(quote!(#field_ident: std::option::Option<#field_type>,));
        builder_field_inits.extend(quote!(#field_ident: None,));
        builder_setters.extend(quote! {
            pub fn #field_ident(&mut self, #field_ident: #field_type) -> &mut Self {
                self.#field_ident = Some(#field_ident);
                self
            }
        });
    }

    Ok(quote! {
        impl #ident {
            pub fn builder() -> #builder_struct_ident {
                #builder_struct_ident {
                    #builder_field_inits
                }
            }
        }
        struct #builder_struct_ident {
            #builder_field_declarations
        }
        impl #builder_struct_ident {
            #builder_setters
        }
    })
}
