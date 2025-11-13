use quote::quote;
use proc_macro2::TokenStream;

pub fn impl_macro(ast: syn::DeriveInput) -> TokenStream {
    let type_name = &ast.ident;
    quote! {
        impl TMValue for #type_name {
            type Bytes = [u8; 1];
            fn from_bytes(bytes: Self::Bytes) -> Self {
                todo!();
            }
            fn to_bytes(&self) -> Self::Bytes {
                todo!();
            }
        }
    }
}
