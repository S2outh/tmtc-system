use quote::quote;
use proc_macro2::TokenStream;

fn impl_struct(type_name: syn::Ident, tm_value_struct: syn::DataStruct) -> TokenStream {
    let struct_type_parsers = tm_value_struct.fields
        .iter()
        .map(|f| {
            let ident = &f.ident;
            quote! {
                pos += self.#ident.read(&bytes[pos..])?;
            }
        });
    let struct_byte_parsers = tm_value_struct.fields
        .iter()
        .map(|f| {
            let ident = &f.ident;
            quote! {
                pos += self.#ident.write(&mut mem[pos..])?;
            }
        });
    let struct_types = tm_value_struct.fields.iter().map(|f| &f.ty);
    quote! {
        impl DynTMValue for #type_name {
            fn read(&mut self, bytes: &[u8]) -> Result<usize, OutOfMemory> {
                let mut pos = 0;
                #(#struct_type_parsers)*
                Ok(pos)
            }
            fn write(&self, mem: &mut [u8]) -> Result<usize, OutOfMemory> {
                let mut pos = 0;
                #(#struct_byte_parsers)*
                Ok(pos)
            }
        }
        impl TMValue for #type_name {
            const BYTE_SIZE: usize = #(<#struct_types as TMValue>::BYTE_SIZE)+*;
        }
    }
}

fn impl_enum(type_name: syn::Ident, tm_value_enum: syn::DataEnum) -> TokenStream {
    let enum_variant_size_cmp = tm_value_enum.variants
        .iter()
        .map(|v| v.fields
                .iter()
                .map(|f| &f.ty)
                .map(|ty| quote!{ #ty::BYTE_SIZE }))
        .map(|sizes| quote! {
            let variant_size = #(#sizes)+*;
            if variant_size > m {
                m = variant_size;
            }
        });
    let enum_variant_parsers = tm_value_enum.variants
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let index = i as u8;
            let field_idents = v.fields
                .iter()
                .map(|v| &v.ident);
            let field_parsers = field_idents.clone()
                .map(|ident| {
                    quote! {
                        pos += #ident.read(&bytes[pos..])?;
                    }
                });
            quote! {
                #index => {
                    #(#field_parsers)*
                }
            }
        });
    let enum_byte_parsers = tm_value_enum.variants
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let ident = &v.ident;
            let index = i as u8;
            let field_idents = v.fields
                .iter()
                .map(|v| &v.ident);
            let field_parsers = field_idents.clone()
                .map(|ident| {
                    quote! {
                        pos += #ident.write(&mut mem[pos..])?;
                    }
                });
            quote! {
                Self::#ident(#(#field_idents),*) => {
                    mem[0] = #index;
                    #(#field_parsers)*
                }
            }
        });
    quote! {
        impl DynTMValue for #type_name {
            fn read(&mut self, bytes: &[u8]) -> Result<usize, OutOfMemory> {
                let mut pos = 1;
                match bytes[0] {
                    #(#enum_variant_parsers)*
                }
                Ok(pos)
            }
            fn write(&self, mem: &mut [u8]) -> Result<usize, OutOfMemory> {
                let mut pos = 1;
                match self {
                    #(#enum_byte_parsers)*
                }
                Ok(pos)
            }
        }
        impl TMValue for #type_name {
            const BYTE_SIZE: usize = {
                let mut m = 0;
                #(#enum_variant_size_cmp)*
                m
            };
        }
    }
}

pub fn impl_macro(ast: syn::DeriveInput) -> TokenStream {
    let type_name = ast.ident.clone();

    match ast.data {
        syn::Data::Struct(tm_value_struct) => impl_struct(type_name, tm_value_struct),
        syn::Data::Enum(tm_value_enum) => impl_enum(type_name, tm_value_enum),
        syn::Data::Union(_) => unimplemented!("unions are not supported as tmvalues"),
    }
}
