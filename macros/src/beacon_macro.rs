use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Meta, Path, Token, punctuated::Punctuated};

pub fn impl_macro(args: Punctuated::<Meta, Token![,]>) -> TokenStream {
    let mut args_iter = args.iter();
    let path_args_iter: Vec<_> = args_iter
        .by_ref()
        .take(2)
        .map(|m| if let Meta::Path(path) = m { path } else { panic!("first two args should be valid paths") })
        .collect();
    
    let beacon_name = path_args_iter.get(0).expect("args should include beacon name");
    let beacon_module_name: TokenStream = beacon_name.to_token_stream().to_string().to_snake_case().parse().unwrap();
    let root_path = path_args_iter.get(1).expect("args should include tm definition path");

    let Meta::NameValue(id_nv) = args_iter.next().expect("args should contain id") else { panic!("third arg should be id"); };
    if id_nv.path.get_ident().expect("third arg should be id") != "id" { panic!("args should contain id"); };
    let id = &id_nv.value;

    let Meta::List(tm_definitions_arg) = args_iter.next().expect("4th arg should contain tm definitions list ") else { panic!("args should contain tm definitions list"); };

    let tm_definitions: Vec<_> = tm_definitions_arg
        .parse_args_with(Punctuated::<Path, Token![,]>::parse_separated_nonempty)
        .expect("could not parse header list").into_iter()
        .collect();

    let fields: Vec<_> = tm_definitions
        .iter()
        .map(|p| (
                p.segments.last().to_token_stream().to_string().to_snake_case().parse::<TokenStream>().unwrap(),
                quote!{ <#root_path::#p as TelemetryDefinition>}
                ))
        .collect();

    let types: Vec<_> = fields.iter().map(|(_, path)| quote!{ #path::TMValueType} ).collect();

    let field_defs: Vec<_> = fields
        .iter()
        .map(|(name, path)| quote!{
            pub #name: #path::TMValueType
        }).collect();

    let field_defaults: Vec<_> = fields
        .iter()
        .map(|(name, path)| quote!{
            #name: #path::TMValueType::default()
        }).collect();

    let field_set_defaults: Vec<_> = fields
        .iter()
        .map(|(name, path)| quote!{
            self.#name = #path::TMValueType::default();
        }).collect();

    let type_parsers = fields
        .iter()
        .map(|(name, _)| {
            quote! {
                pos += self.#name.read(&bytes[pos..]);
            }
        });
    let byte_parsers = fields
        .iter()
        .map(|(name, _)| {
            quote! {
                pos += self.#name.write(&mut self.storage[pos..]);
            }
        });
    let type_setters = fields
        .iter()
        .map(|(name, path)| {
            quote! {
               #path::ID => self.#name.read(bytes),
            }
        });
    let type_getters = fields
        .iter()
        .map(|(name, path)| {
            quote! {
               #path::ID => self.#name.write(&mut self.storage[..]),
            }
        });
    let header_size: usize = 3;

    quote! {
        pub mod #beacon_module_name {
            use tmtc_system::{internal::TelemetryDefinition, *};
            const BEACON_ID: u8 = #id;
            pub struct #beacon_name {
                storage: [u8; Self::BYTE_SIZE],
                #(#field_defs),*
            }
            impl #beacon_name {
                const BYTE_SIZE: usize = #header_size + #(<#types as TMValue>::BYTE_SIZE)+*;

                pub fn new() -> Self {
                    Self {
                        storage: [0u8; Self::BYTE_SIZE],
                        #(#field_defaults),*
                    }
                }
            }
            impl DynBeacon for #beacon_name {
                fn from_bytes(&mut self, bytes: &[u8]) -> Result<(), ParseError> {
                    if bytes.get(0).ok_or(ParseError::TooShort)? != &BEACON_ID {
                        return Err(ParseError::WrongId);
                    }
                    // check CRC
                    let mut pos = #header_size;
                    #(#type_parsers)*
                    Ok(())
                }
                fn bytes(&mut self) -> &[u8] {
                    self.storage[0] = BEACON_ID;
                    // set CRC
                    let mut pos = #header_size;
                    #(#byte_parsers)*
                    &self.storage[..pos]
                }

                fn insert_slice(&mut self, telemetry_definition: &dyn DynTelemetryDefinition, bytes: &[u8]) -> Result<(), BoundsError> {
                    match telemetry_definition.id() {
                        #(#type_setters)*
                        _ => return Err(BoundsError),
                    };
                    Ok(())
                }
                fn get_slice<'a>(&'a mut self, telemetry_definition: &dyn DynTelemetryDefinition) -> Result<&'a [u8], BoundsError> {
                    let length = match telemetry_definition.id() {
                        #(#type_getters)*
                        _ => return Err(BoundsError),
                    };
                    Ok(&self.storage[..length])
                }
                fn flush(&mut self) {
                    #(#field_set_defaults)*
                }
            }
        }
    }
}
