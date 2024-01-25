use proc_macro::TokenStream as StdTokenStream;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, Data, DataStruct, DeriveInput, Fields, GenericArgument, Path,
    PathArguments, Type, TypePath,
};

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

    let (field_idents, (field_types, field_is_option)): (Vec<Ident>, (Vec<Type>, Vec<bool>)) =
        match data {
            Data::Struct(DataStruct {
                fields: Fields::Named(fields),
                ..
            }) => fields
                .named
                .into_iter()
                .map(|field| {
                    let is_option = match &field.ty {
                        Type::Path(tp) => tp.path.segments[0].ident == "Option",
                        _ => false,
                    };
                    (field.ident.unwrap(), (field.ty, is_option))
                })
                .unzip(),
            _ => {
                return Err(syn::Error::new_spanned(
                    &ident,
                    "only named struct is supported",
                ));
            }
        };

    let builder_struct_ident = format_ident!("{ident}Builder");
    let builder_field_types: Vec<Type> =
        Iterator::zip(field_types.iter(), field_is_option.iter().copied())
            .map(|(ty, is_option)| {
                if is_option {
                    ty.clone()
                } else {
                    parse_quote!(std::option::Option<#ty>)
                }
            })
            .collect();
    let builder_setter_types: Vec<Type> = builder_field_types.iter().try_fold(
        Vec::with_capacity(builder_field_types.len()),
        |mut vector, ty| {
            let inner_type = extract_option_inner_type(ty)?;
            vector.push(inner_type);
            Ok::<Vec<Type>, syn::Error>(vector)
        },
    )?;

    let builder_build_field_checkers: TokenStream =
        Iterator::zip(field_idents.iter(), field_is_option.iter().copied())
            .map(|(field_ident, is_option)| {
                let check_code = if !is_option {
                    quote!(.ok_or_else(|| std::boxed::Box::<dyn std::error::Error>::from(concat!("field ", stringify!(#field_ident), " not set").to_string()))?)
                } else { quote!() };
                quote!(let #field_ident = self.#field_ident.take() #check_code ;)
            })
            .collect();

    Ok(quote! {
        impl #ident {
            pub fn builder() -> #builder_struct_ident {
                #builder_struct_ident {
                    #( #field_idents: std::option::Option::None, )*
                }
            }
        }
        struct #builder_struct_ident {
            #( #field_idents: #builder_field_types, )*
        }
        impl #builder_struct_ident {
            #(
                pub fn #field_idents(&mut self, #field_idents: #builder_setter_types) -> &mut Self {
                    self.#field_idents = std::option::Option::Some(#field_idents);
                    self
                }
            )*
            pub fn build(&mut self) -> std::result::Result<#ident, Box<dyn std::error::Error>> {
                #builder_build_field_checkers
                std::result::Result::Ok(#ident{
                    #( #field_idents, )*
                })
            }
        }
    })
}

fn extract_option_inner_type(ty: &Type) -> syn::Result<Type> {
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = ty
    {
        for seg in segments {
            if let PathArguments::AngleBracketed(args) = &seg.arguments {
                if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                    return Ok(inner_type.clone());
                }
            }
        }
    }
    Err(syn::Error::new_spanned(ty, "invalid option type"))
}
