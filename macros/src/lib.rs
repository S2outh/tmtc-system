#![feature(iter_intersperse)]
mod beacon_macro;
mod tm_definition_macro_attribute;
mod tm_value_macro_derive;
use std::panic;

use proc_macro::TokenStream;
use syn::{Meta, Token, punctuated::Punctuated};

#[proc_macro_derive(TMValue)]
pub fn tm_value(item: TokenStream) -> TokenStream {
    let ast = syn::parse(item).unwrap();
    
    // Build the TMValue and DynTMValue trait implementations
    tm_value_macro_derive::impl_macro(ast).into()
}

#[proc_macro]
pub fn beacon(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input with Punctuated<Meta, Token![,]>::parse_separated_nonempty);

    // Build the beacon definition and implementation
    beacon_macro::impl_macro(args).into()
}


#[proc_macro_attribute]
pub fn telemetry_definition(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = syn::parse(item).unwrap();
    let name_value_pair = syn::parse_macro_input!(attr as syn::MetaNameValue);
    if name_value_pair.path.get_ident().expect("invalid attribute") != "id" {
        panic!("telemetry definition should only contain id attr");
    }
    let syn::Expr::Lit(id_expr) = name_value_pair.value else { panic!("wrong macro attributes") };
    let syn::Lit::Int(id_lit) = id_expr.lit else { panic!("wrong macro attributes") };
    let id = id_lit.base10_parse::<u16>().expect("macro input should be an u16");

    // Build the telemetry definition recursive module
    tm_definition_macro_attribute::impl_macro(ast, id).into()
}
