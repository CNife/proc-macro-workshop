use proc_macro::TokenStream as StdTokenStream;
use std::fmt::Display;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use quote::ToTokens;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Field, GenericParam, Generics, Meta,
    MetaNameValue,
};

type TokenStreamResult = syn::Result<TokenStream>;

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: StdTokenStream) -> StdTokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let result =
        make_debug_derive_code(dbg!(derive_input)).unwrap_or_else(|e| e.to_compile_error());
    eprintln!("{result}");
    result.into()
}

fn make_debug_derive_code(input: DeriveInput) -> TokenStreamResult {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input;

    let impl_header_code = make_impl_header(&ident, generics)?;
    let fmt_code = make_fmt_code(&ident, &data)?;

    Ok(quote! {
        #impl_header_code {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #fmt_code
            }
        }
    })
}

fn make_impl_header(ident: &Ident, mut generics: Generics) -> TokenStreamResult {
    for param in generics.params.iter_mut() {
        if let GenericParam::Type(tp) = param {
            tp.bounds.push(parse_quote!(std::fmt::Debug));
        }
    }
    let (impl_generics, type_generics, where_clause) = dbg!(generics.split_for_impl());

    Ok(quote! {
        impl #impl_generics std::fmt::Debug for #ident #type_generics #where_clause
    })
}

fn make_fmt_code(ident: &Ident, data: &Data) -> TokenStreamResult {
    match data {
        Data::Struct(ds) => {
            let fields_code =
                ds.fields
                    .iter()
                    .try_fold(TokenStream::new(), |mut fields_code, field| {
                        make_fmt_field_code(field).map(|field_code| {
                            fields_code.extend(field_code);
                            fields_code
                        })
                    })?;
            Ok(quote! {
                f.debug_struct(stringify!(#ident))
                #fields_code
                .finish()
            })
        }
        _ => return error(ident, "only struct is supported"),
    }
}

fn make_fmt_field_code(field: &Field) -> TokenStreamResult {
    let Some(ident) = &field.ident else {
        return error(field, "only named struct is supported");
    };
    let value_code = field
        .attrs
        .iter()
        .find_map(|attr| match &attr.meta {
            Meta::NameValue(MetaNameValue { path, value, .. }) if path.is_ident("debug") => {
                Some(value)
            }
            _ => None,
        })
        .map_or_else(
            || quote!(&self.#ident),
            |format| quote!(&format_args!(#format, &self.#ident)),
        );
    Ok(quote! {
        .field(stringify!(#ident), #value_code)
    })
}

fn error<S, T: ToTokens, U: Display>(item: T, msg: U) -> syn::Result<S> {
    Err(syn::Error::new_spanned(item, msg))
}
