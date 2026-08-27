use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Ident, Index};

fn impl_struct(tm_value_struct: syn::DataStruct) -> TokenStream {
    let struct_type_parsers = tm_value_struct.fields.iter().enumerate().map(|(i, f)| {
        let ident = f
            .ident
            .as_ref()
            .map(Ident::to_token_stream)
            .unwrap_or(Index::from(i).to_token_stream());
        let ty = &f.ty;
        quote! {
            #ident: {
                let (len, value) = <#ty>::read(&bytes[pos..])?;
                pos += len;
                value
            }
        }
    });
    let struct_byte_parsers = tm_value_struct.fields.iter().enumerate().map(|(i, f)| {
        let ident = f
            .ident
            .as_ref()
            .map(Ident::to_token_stream)
            .unwrap_or(Index::from(i).to_token_stream());
        quote! {
            pos += self.#ident.write(&mut mem[pos..])?;
        }
    });
    let struct_types = tm_value_struct.fields.iter().map(|f| &f.ty);
    quote! {
        const MAX_BYTE_SIZE: usize = #(<#struct_types as ChellValue>::MAX_BYTE_SIZE)+*;
        fn read(bytes: &[u8]) -> Result<(usize, Self), ChellValueError> {
            let mut pos = 0;
            let value = Self {
                #(#struct_type_parsers),*
            };
            Ok((pos, value))
        }
        fn write(&self, mem: &mut [u8]) -> Result<usize, ChellValueError> {
            let mut pos = 0;
            #(#struct_byte_parsers)*
            Ok(pos)
        }
    }
}

fn impl_enum(tm_value_enum: syn::DataEnum) -> TokenStream {
    let enum_variant_size_cmp = tm_value_enum.variants.iter().map(|v| {
        let iter: Box<dyn Iterator<Item = _>> = match &v.fields {
            syn::Fields::Unit => Box::new(std::iter::empty()),
            syn::Fields::Unnamed(unnamed_fields) => Box::new(unnamed_fields.unnamed.iter()),
            syn::Fields::Named(named_fields) => Box::new(named_fields.named.iter()),
        };
        let sizes = iter
            .map(|f| &f.ty)
            .map(|ty| quote! { <#ty>::MAX_BYTE_SIZE });
        quote! {
            let variant_size = 1usize #(+ #sizes)*;
            if variant_size > m {
                m = variant_size;
            }
        }
    });
    let enum_variant_parsers = tm_value_enum.variants.iter().enumerate().map(|(i, v)| {
        let index = i as u8;
        let ident = &v.ident;
        match &v.fields {
            syn::Fields::Unit => {
                quote! {
                    #index => {
                        Self::#ident
                    }
                }
            }
            syn::Fields::Unnamed(unnamed_fields) => {
                let field_parsers = unnamed_fields.unnamed.iter().map(|v| {
                    let ty = &v.ty;
                    quote! {{
                        let (len, value) = <#ty>::read(&bytes[pos..])?;
                        pos += len;
                        value
                    }}
                });
                quote! {
                    #index => {
                        Self::#ident(#(#field_parsers),*)
                    }
                }
            }
            syn::Fields::Named(named_fields) => {
                let field_parsers = named_fields.named.iter().map(|v| {
                    let ty = &v.ty;
                    let field_name = &v.ident;
                    quote! {#field_name: {
                        let (len, value) = <#ty>::read(&bytes[pos..])?;
                        pos += len;
                        value
                    }}
                });
                quote! {
                    #index => {
                        Self::#ident{#(#field_parsers),*}
                    }
                }
            }
        }
    });
    let enum_byte_parsers = tm_value_enum.variants.iter().enumerate().map(|(i, v)| {
        let ident = &v.ident;
        let index = i as u8;
        match &v.fields {
            syn::Fields::Unit => {
                quote! {
                    Self::#ident => {
                        *(mem.first_mut().ok_or(ChellValueError::OutOfMemory)?) = #index;
                    }
                }
            }
            syn::Fields::Unnamed(unnamed_fields) => {
                let field_idents = (0..unnamed_fields.unnamed.len())
                    .map(|i| Ident::new(&format!("v{}", i), proc_macro2::Span::call_site()));
                let field_parsers = field_idents.clone().map(|ident| {
                    quote! {
                        pos += #ident.write(&mut mem[pos..])?;
                    }
                });
                quote! {
                    Self::#ident(#(#field_idents),*) => {
                        *(mem.first_mut().ok_or(ChellValueError::OutOfMemory)?) = #index;
                        #(#field_parsers)*
                    }
                }
            }
            syn::Fields::Named(named_fields) => {
                let field_idents = named_fields.named.iter().map(|v| &v.ident);
                let field_parsers = field_idents.clone().map(|ident| {
                    quote! {
                        pos += #ident.write(&mut mem[pos..])?;
                    }
                });
                quote! {
                    Self::#ident{#(#field_idents),*} => {
                        *(mem.first_mut().ok_or(ChellValueError::OutOfMemory)?) = #index;
                        #(#field_parsers)*
                    }
                }
            }
        }
    });
    quote! {
        const MAX_BYTE_SIZE: usize = {
            let mut m = 0;
            #(#enum_variant_size_cmp)*
            m
        };
        fn read(bytes: &[u8]) -> Result<(usize, Self), ChellValueError> {
            let mut pos = 1;
            let value = match bytes.first().ok_or(ChellValueError::OutOfMemory)? {
                #(#enum_variant_parsers)*
                _ => return Err(ChellValueError::BadEnumVariant)
            };
            Ok((pos, value))
        }
        fn write(&self, mem: &mut [u8]) -> Result<usize, ChellValueError> {
            let mut pos = 1;
            match self {
                #(#enum_byte_parsers)*
            }
            Ok(pos)
        }
    }
}

pub fn impl_macro(ast: syn::DeriveInput) -> TokenStream {
    let type_name = ast.ident.clone();
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let body = match ast.data {
        syn::Data::Struct(tm_value_struct) => impl_struct(tm_value_struct),
        syn::Data::Enum(tm_value_enum) => impl_enum(tm_value_enum),
        syn::Data::Union(_) => unimplemented!("unions are not supported as chell values"),
    };

    quote! {
        impl #impl_generics ChellValue for #type_name #ty_generics #where_clause {
            #body
        }
    }
}
