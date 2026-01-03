use std::iter::once;

use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Meta, Path, Token, punctuated::Punctuated};

pub fn impl_macro(args: Punctuated::<Meta, Token![,]>) -> TokenStream {
    let mut args_iter = args.iter();
    let path_args_iter: Vec<_> = args_iter
        .by_ref()
        .take(3)
        .map(|m| if let Meta::Path(path) = m { path } else { panic!("first 3 args should be valid paths") })
        .collect();
    
    let beacon_name = path_args_iter.get(0).expect("args should include beacon name");
    let beacon_module_name: TokenStream = beacon_name.to_token_stream().to_string().to_snake_case().parse().unwrap();
    let root_path = path_args_iter.get(1).expect("args should include tm definition path");
    let timestamp_path = path_args_iter.get(2).expect("args should include timestamp path");

    let Meta::NameValue(id_nv) = args_iter.next().expect("args should contain id") else { panic!("third arg should be id"); };
    if id_nv.path.get_ident().expect("third arg should be id") != "id" { panic!("args should contain id"); };
    let id = &id_nv.value;

    let Meta::List(tm_definitions_arg) = args_iter.next().expect("5th arg should contain tm definitions list ") else { panic!("args should contain tm definitions list"); };

    let tm_definitions: Vec<_> = tm_definitions_arg
        .parse_args_with(Punctuated::<Path, Token![,]>::parse_separated_nonempty)
        .expect("could not parse header list").into_iter()
        .collect();

    let serializable_fields: Vec<_> = tm_definitions
        .iter()
        .map(|p| (
                p.segments
                    .iter()
                    .map(|v| v.to_token_stream().to_string().to_snake_case())
                    .collect::<Vec<String>>()
                    .join("_")
                    .parse::<TokenStream>().unwrap(),
                quote!{ #root_path::#p }
                ))
        .collect();

    let timestamp_field = (quote!{timestamp}, quote!{#timestamp_path});
    let fields: Vec<_> = once(&timestamp_field)
        .chain(serializable_fields.iter())
        .map(|(name, path)| (name, quote!{ <#path as InternalTelemetryDefinition> }))
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
                pos += self.#name.read(&bytes[pos..]).map_err(|_| ParseError::OutOfMemory)?;
            }
        });
    let byte_parsers = fields
        .iter()
        .map(|(name, _)| {
            quote! {
                pos += self.#name.write(&mut self.storage[pos..]).unwrap();
            }
        });
    let type_setters = fields
        .iter()
        .map(|(name, path)| {
            quote! {
               #path::ID => self.#name.read(bytes).map_err(|_| BeaconOperationError::OutOfMemory)?,
            }
        });
    let type_getters = fields
        .iter()
        .map(|(name, path)| {
            quote! {
               #path::ID => self.#name.write(&mut self.storage[..]).unwrap(),
            }
        });
    let serializers = serializable_fields
        .iter()
        .map(|(name, path)| {
            quote! {
                let nats_value = NatsTelemetry::new(timestamp, self.#name);
                let bytes = serde_cbor::to_vec(&nats_value).unwrap();
                serialized_values.push((#path.address(), bytes));
            }
        });
    let serializer_func = if cfg!(feature = "serde") {
       quote! {
           pub fn serialize(&self) -> alloc::vec::Vec<(&'static str, alloc::vec::Vec<u8>)> {
               let mut serialized_values = alloc::vec::Vec::new();
               let timestamp = self.timestamp;
               #(#serializers)*
               serialized_values 
           }
       } 
    } else {quote! {}};
    let header_size: usize = 3;

    quote! {
        pub mod #beacon_module_name {
            use tmtc_system::{internal::*, *};
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
                #serializer_func
            }
            impl Beacon for #beacon_name {
                fn from_bytes(&mut self, bytes: &[u8], crc_func: &mut dyn FnMut(&[u8]) -> u16) -> Result<(), ParseError> {
                    if bytes.len() < #header_size {
                        return Err(ParseError::OutOfMemory);
                    }
                    if bytes[0] != BEACON_ID {
                        return Err(ParseError::WrongId);
                    }
                    let received_crc = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                    let calculated_crc = (crc_func)(&bytes[#header_size..]);
                    if calculated_crc != received_crc {
                        return Err(ParseError::BadCRC);
                    }
                    let mut pos = #header_size;
                    #(#type_parsers)*
                    Ok(())
                }
                fn bytes(&mut self, crc_func: &mut dyn FnMut(&[u8]) -> u16) -> &[u8] {
                    self.storage[0] = BEACON_ID;
                    let mut pos = #header_size;
                    #(#byte_parsers)*
                    let crc = (crc_func)(&self.storage[#header_size..pos]);
                    self.storage[1..3].copy_from_slice(&crc.to_le_bytes());
                    &self.storage[..pos]
                }

                fn insert_slice(&mut self, telemetry_definition: &dyn TelemetryDefinition, bytes: &[u8]) -> Result<(), BeaconOperationError> {
                    match telemetry_definition.id() {
                        #(#type_setters)*
                        _ => return Err(BeaconOperationError::DefNotInBeacon),
                    };
                    Ok(())
                }
                fn get_slice<'b>(&'b mut self, telemetry_definition: &dyn TelemetryDefinition) -> Result<&'b [u8], BeaconOperationError> {
                    let length = match telemetry_definition.id() {
                        #(#type_getters)*
                        _ => return Err(BeaconOperationError::DefNotInBeacon),
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
