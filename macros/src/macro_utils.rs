use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::Type;

pub fn parse_type_path(ty: &Type) -> TokenStream {
    match ty {
        Type::Path(syn::TypePath { path, .. }) => path
            .segments
            .iter()
            .map(|s| {
                let ident = &s.ident;
                match &s.arguments {
                    syn::PathArguments::None => s.ident.to_token_stream(),
                    syn::PathArguments::AngleBracketed(args) => quote! {#ident::#args},
                    syn::PathArguments::Parenthesized(_) => {
                        panic!("Parenthesized types are unsupported")
                    }
                }
            })
            .collect(),
        Type::Array(a) => quote! {<#a>},
        _ => panic!("unsupported type"),
    }
}
