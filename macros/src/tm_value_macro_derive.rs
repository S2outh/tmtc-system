use quote::quote;
use proc_macro2::TokenStream;

pub fn impl_macro(ast: syn::DeriveInput) -> TokenStream {
    let type_name = &ast.ident;
    let syn::Data::Struct(tm_value_struct) = ast.data else {
        panic!("tm value is not a struct");
    };
    let struct_byte_parsers = tm_value_struct.fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let ident = &f.ident;
            quote! {{
                let field_bytes = self.#ident.to_bytes();
                let pos = Self::get_pos(#i);
                bytes[pos..(pos+Self::SIZES[#i])].copy_from_slice(&field_bytes);
            }}
        });
    let struct_type_parsers = tm_value_struct.fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let ident = &f.ident;
            let ty = &f.ty;
            quote! {
                #ident: {
                    let pos = Self::get_pos(#i);
                    let field_bytes: [u8; Self::SIZES[#i]] = bytes[pos..(pos+Self::SIZES[#i])].try_into().unwrap();
                    #ty::from_bytes(field_bytes)
                }
            }
        });
    let struct_types = tm_value_struct.fields.iter().map(|f| &f.ty);
    let struct_len = tm_value_struct.fields.len();
    quote! {
        impl #type_name {
            const SIZES: [usize; #struct_len] = [#(size_of::<<#struct_types as TMValue>::Bytes>()),*];
            const fn get_pos(index: usize) -> usize {
                let mut len = 0;
                let mut i = 0;
                while i < index {
                    len += Self::SIZES[i];
                    i += 1;
                }
                len
            }
            const fn len() -> usize {
                Self::get_pos(Self::SIZES.len())
            }
        }
        impl TMValue for #type_name {
            type Bytes = [u8; Self::len()];
            fn from_bytes(bytes: Self::Bytes) -> Self {
                Self {
                    #(#struct_type_parsers),*
                }
            }
            fn to_bytes(&self) -> Self::Bytes {
                let mut bytes: Self::Bytes = [0u8; {Self::len()}];
                #(#struct_byte_parsers)*
                bytes
            }
        }
    }
}
