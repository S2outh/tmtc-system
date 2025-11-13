mod beacon_macro_attribute;
mod tm_value_macro_derive;
use proc_macro::TokenStream;

#[proc_macro_derive(TMValue)]
pub fn tm_value_macro_derive(item: TokenStream) -> TokenStream {
    let ast = syn::parse(item).unwrap();
    
    // Build the trait implementation
    tm_value_macro_derive::impl_macro(ast).into()
}

#[proc_macro_attribute]
pub fn beacon(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = syn::parse(item).unwrap();
    let path = syn::parse_macro_input!(attr as syn::Path);

    // Build the beacon definition and implementation
    beacon_macro_attribute::impl_macro(path, ast).into()
}
