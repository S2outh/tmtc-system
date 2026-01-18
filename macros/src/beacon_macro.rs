use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Meta, Path, Token, punctuated::Punctuated};

pub fn impl_macro(args: Punctuated<Meta, Token![,]>) -> TokenStream {
    let mut args_iter = args.iter();
    let path_args_iter: Vec<_> = args_iter
        .by_ref()
        .take(3)
        .map(|m| {
            if let Meta::Path(path) = m {
                path
            } else {
                panic!("first 3 args should be valid paths")
            }
        })
        .collect();

    let beacon_name = path_args_iter
        .get(0)
        .expect("args should include beacon name");
    let beacon_module_name: TokenStream = beacon_name
        .to_token_stream()
        .to_string()
        .to_snake_case()
        .parse()
        .unwrap();
    let root_path = path_args_iter
        .get(1)
        .expect("args should include tm definition path");
    let timestamp_path = path_args_iter
        .get(2)
        .expect("args should include timestamp path");
    let timestamp_type = quote! { <#timestamp_path as InternalTelemetryDefinition>::TMValueType };

    let Meta::NameValue(id_nv) = args_iter.next().expect("args should contain id") else {
        panic!("third arg should be id");
    };
    if id_nv.path.get_ident().expect("third arg should be id") != "id" {
        panic!("args should contain id");
    };
    let id = &id_nv.value;

    let Meta::List(tm_definitions_arg) = args_iter
        .next()
        .expect("5th arg should contain tm definitions list ")
    else {
        panic!("args should contain tm definitions list");
    };

    let tm_definitions: Vec<_> = tm_definitions_arg
        .parse_args_with(Punctuated::<Path, Token![,]>::parse_separated_nonempty)
        .expect("could not parse header list")
        .into_iter()
        .collect();

    let fields: Vec<_> = tm_definitions
        .iter()
        .map(|p| {
            (
                p.segments
                    .iter()
                    .map(|v| v.to_token_stream().to_string().to_snake_case())
                    .collect::<Vec<String>>()
                    .join("_")
                    .parse::<TokenStream>()
                    .unwrap(),
                quote! { #root_path::#p },
            )
        })
        .collect();

    let itd_fields: Vec<_> = fields
        .iter()
        .map(|(name, path)| (name, quote! { <#path as InternalTelemetryDefinition> }))
        .collect();

    let types: Vec<_> = itd_fields
        .iter()
        .map(|(_, path)| quote! { #path::TMValueType})
        .collect();

    let field_defs: Vec<_> = itd_fields
        .iter()
        .map(|(name, path)| {
            quote! {
                pub #name: Option<#path::TMValueType>
            }
        })
        .collect();

    let field_defaults: Vec<_> = itd_fields
        .iter()
        .map(|(name, _)| {
            quote! {
                #name: None
            }
        })
        .collect();

    let field_set_defaults: Vec<_> = itd_fields
        .iter()
        .map(|(name, _)| {
            quote! {
                self.#name = None;
            }
        })
        .collect();

    let type_parsers = itd_fields.iter().enumerate().map(|(i, (name, path))| {
        quote! {
            if bitfield.get(#i) {
                let (len, value) = #path::TMValueType::read(&bytes[pos..]).map_err(|_| ParseError::OutOfMemory)?;
                pos += len;
                self.#name = Some(value);
            } else {
                self.#name = None;
            }
        }
    });
    let byte_parsers = itd_fields.iter().enumerate().map(|(i, (name, _))| {
        quote! {
            if let Some(value) = self.#name {
                pos += value.write(&mut self.storage[pos..]).unwrap();
                bitfield.set(#i);
            }
        }
    });
    let type_setters = itd_fields.iter().map(|(name, path)| {
        quote! {
           #path::ID => {
               let (_, value) = #path::TMValueType::read(bytes).map_err(|_| BeaconOperationError::OutOfMemory)?;
               self.#name = Some(value);
           }
        }
    });
    let serializers = fields.iter().map(|(name, path)| {
        quote! {
            if let Some(value) = self.#name {
                let nats_value = NatsTelemetry::new(timestamp, value);
                let bytes = nats_value.erased_serialize(serializer)?;
                serialized_values.push((#path.address(), bytes));
            }
        }
    });
    let serializer_func = if cfg!(feature = "serde") {
        quote! {
            use alloc::vec::Vec;
            use erased_serde::{self, Serializer};
            pub fn serialize(&self, serializer: &dyn Serializer) -> Result<Vec<(&'static str, Vec<u8>)>, erased_serde::Error> {
                let mut serialized_values = Vec::new();
                let timestamp = self.timestamp;
                #(#serializers)*
                Ok(serialized_values)
            }
        }
    } else {
        quote! {}
    };
    let bitfield_size: usize = (fields.len() as f32 / 8.).ceil() as usize;
    let header_size: usize = 1 + 2 + bitfield_size; // id + crc

    quote! {
        pub mod #beacon_module_name {
            use tmtc_system::{internal::*, *};
            const BEACON_ID: u8 = #id;
            pub struct #beacon_name {
                storage: [u8; Self::BYTE_SIZE],
                pub timestamp: #timestamp_type,
                #(#field_defs),*
            }
            impl #beacon_name {
                const BYTE_SIZE: usize = #header_size
                    + <#timestamp_type as TMValue>::BYTE_SIZE
                    + #(<#types as TMValue>::BYTE_SIZE)+*;

                pub fn new() -> Self {
                    Self {
                        storage: [0u8; Self::BYTE_SIZE],
                        timestamp: #timestamp_type::default(),
                        #(#field_defaults),*
                    }
                }
                #serializer_func
            }
            impl Beacon for #beacon_name {
                type Timestamp = #timestamp_type;
                fn from_bytes(&mut self, bytes: &[u8], crc_func: &mut dyn FnMut(&[u8]) -> u16) -> Result<(), ParseError> {
                    if bytes.len() < #header_size {
                        return Err(ParseError::OutOfMemory);
                    }
                    // Beacon ID
                    if bytes[0] != BEACON_ID {
                        return Err(ParseError::WrongId);
                    }
                    // Crc
                    let received_crc = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
                    let calculated_crc = (crc_func)(&bytes[3..]);
                    if calculated_crc != received_crc {
                        return Err(ParseError::BadCRC);
                    }
                    let mut pos = #header_size;
                    // Bitfield
                    let bitfield = Bitfield::<#bitfield_size>::new_from_bytes(bytes[3..#header_size].try_into().unwrap());
                    // Timestamp
                    let (len, timestamp_value) = #timestamp_type::read(&bytes[pos..]).map_err(|_| ParseError::OutOfMemory)?;
                    pos += len;
                    self.timestamp = timestamp_value;
                    // Parsers
                    #(#type_parsers)*
                    Ok(())
                }
                fn to_bytes(&mut self, crc_func: &mut dyn FnMut(&[u8]) -> u16) -> &[u8] {
                    // Beacon ID
                    self.storage[0] = BEACON_ID;
                    let mut pos = #header_size;
                    // Bitfield
                    let mut bitfield = Bitfield::<#bitfield_size>::new();
                    // Timestamp
                    pos += self.timestamp.write(&mut self.storage[pos..]).unwrap();
                    // Parsers
                    #(#byte_parsers)*
                    // Store Bitfield
                    self.storage[3..#header_size].copy_from_slice(bitfield.bytes());
                    // Crc
                    let crc = (crc_func)(&self.storage[3..pos]);
                    self.storage[1..3].copy_from_slice(&crc.to_le_bytes());
                    &self.storage[..pos]
                }
                fn set_timestamp(&mut self, timestamp: Self::Timestamp) {
                    self.timestamp = timestamp;
                }
                fn insert_slice(&mut self, telemetry_definition: &dyn TelemetryDefinition, bytes: &[u8]) -> Result<(), BeaconOperationError> {
                    match telemetry_definition.id() {
                        #(#type_setters)*
                        _ => return Err(BeaconOperationError::DefNotInBeacon),
                    };
                    Ok(())
                }
                fn flush(&mut self) {
                    #(#field_set_defaults)*
                }
            }
        }
    }
}
