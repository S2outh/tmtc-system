use std::array::from_fn;
use std::iter::{once, zip};

use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Item, Meta, Token, punctuated::Punctuated};
use syn::{MetaNameValue, Type};

const TM_VALUE_MACRO_NAME: &str = "tmv";
const TM_MODULE_MACRO_NAME: &str = "tmm";

struct TmValueMacroInput {
    pub ty: Type,
    pub metas: Punctuated<MetaNameValue, Token![,]>,
}

impl Parse for TmValueMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse first argument as a Type
        let ty: Type = input.parse()?;

        // If there's nothing else, return early
        if input.is_empty() {
            return Ok(Self {
                ty,
                metas: Punctuated::new(),
            });
        }

        // Expect comma after type
        input.parse::<Token![,]>()?;

        // Parse remaining key-value pairs
        let metas = Punctuated::<MetaNameValue, Token![,]>::parse_terminated(input)?;

        Ok(Self { ty, metas })
    }
}

fn generate_module_recursive(
    address: Vec<syn::Ident>,
    id: &mut u16,
    items: &Vec<Item>,
) -> [TokenStream; 4] {
    items
        .iter()
        .map(|v| {
            match v {
                syn::Item::Struct(v) => {
                    // Parse "tmv" attribute
                    let args: TmValueMacroInput = v
                        .attrs
                        .iter()
                        .find(|attr| attr.path().is_ident(TM_VALUE_MACRO_NAME))
                        .expect(&format!(
                            "Struct {} has no {} attribute",
                            &v.ident, TM_VALUE_MACRO_NAME
                        ))
                        .parse_args()
                        .expect(&format!(
                            "Could not parse {} attribute parameters",
                            TM_VALUE_MACRO_NAME
                        ));

                    let tmty: Type = args.ty;

                    let (address_endings, funcs): (Vec<_>, Vec<_>) = args.metas
                        .into_iter()
                        .map(|v| (v.path, v.value)).unzip();

                    // this definitions name
                    let def = &v.ident;
                    // Parse rust address of the struct inside the telemetry module tree
                    let def_addr: TokenStream = address
                        .iter()
                        .skip(1)
                        .chain(once(def))
                        .map(|i| i.to_token_stream())
                        .intersperse(quote!(::))
                        .collect();
                    // Parse type of the TMValue the struct references
                    // Increment id
                    let tm_id = *id;
                    *id += 1;
                    // calculate string address based on module tree
                    let str_base_addr: String = address
                        .iter()
                        .map(|i| i.to_string())
                        .intersperse(String::from("."))
                        .collect();
                    // Parse address
                    let address = format!("{}.{}", str_base_addr, def.to_string().to_snake_case());

                    // Serializer func
                    let serializer_func = if cfg!(feature = "ground") {
                        quote!{
                            impl SerializableTMValue<#def> for #tmty {
                                fn serialize_ground<T, S>(self, _def: &#def, timestamp: T, serializer: &S)
                                    -> Result<alloc::vec::Vec<(&'static str, alloc::vec::Vec<u8>)>, S::Error>
                                    where T: serde::Serialize + Clone + Copy,
                                          S: Serializer
                                {
                                    let mut serialized_pairs = alloc::vec::Vec::new();
                                    #({
                                        let nats_value = GroundTelemetry::new(timestamp, (#funcs)(&self));
                                        let bytes = serializer.serialize_value(&nats_value)?;
                                        serialized_pairs.push((concat!(#address, ".", stringify!(#address_endings)), bytes));
                                    })*

                                    let raw_nats_value = GroundTelemetry::new(timestamp, self);
                                    let raw_bytes = serializer.serialize_value(&raw_nats_value)?;
                                    serialized_pairs.push((#address, raw_bytes));

                                    Ok(serialized_pairs)
                                }
                            }
                        }
                    } else {
                        quote!{}
                    };
                    [
                        quote! {
                            pub struct #def;
                            impl InternalTelemetryDefinition for #def {
                                type TMValueType = #tmty;
                                const ID: u16 = #tm_id;
                            }
                            impl const TelemetryDefinition for #def {
                                fn id(&self) -> u16 { Self::ID }
                                fn address(&self) -> &str { #address }
                            }
                            #serializer_func
                        },
                        quote! {
                            #tm_id => Ok(&#def_addr),
                        },
                        quote! {
                            #address => Ok(&#def_addr),
                        },
                        quote! {
                            #def::BYTE_SIZE,
                        },
                    ]
                }
                syn::Item::Mod(v) => {
                    // Parse "tmm" attribute
                    if let Some(module_id) = v
                        .attrs
                        .iter()
                        .find(|attr| attr.path().is_ident(TM_MODULE_MACRO_NAME))
                        .map(|v| {
                            v.parse_args_with(
                                Punctuated::<Meta, Token![,]>::parse_separated_nonempty,
                            )
                            .expect(&format!(
                                "Could not parse {} attribute parameters",
                                TM_MODULE_MACRO_NAME
                            ))
                            .iter()
                            .filter_map(|m| m.require_name_value().ok())
                            .filter(|m| m.path.get_ident().filter(|p| *p == "id").is_some())
                            .map(|m| {
                                if let syn::Expr::Lit(value) = &m.value {
                                    value
                                } else {
                                    panic!("unexpected attribute value type")
                                }
                            })
                            .map(|m| {
                                if let syn::Lit::Int(value) = &m.lit {
                                    value
                                } else {
                                    panic!("unexpected attribute value type")
                                }
                            })
                            .next()
                            .map(|lit| lit.base10_parse().unwrap())
                        })
                        .flatten()
                    {
                        if *id > module_id {
                            panic!("like schedules, ids should only move in one direction");
                        }
                        *id = module_id;
                    }
                    let start_id = *id;

                    let module_name = v.ident.clone();
                    let mut address = address.clone();
                    address.push(module_name.clone());
                    let [module_content, id_getters, address_getters, byte_lengths] =
                        generate_module_recursive(
                            address,
                            id,
                            &v.content.as_ref().expect("module sould not be empty").1,
                        );

                    [
                        quote! {
                            pub mod #module_name {
                                use super::*;
                                pub const fn id_range() -> (u16, u16) {
                                    (#start_id, #id)
                                }
                                pub const MAX_BYTE_SIZE: usize = {
                                    let SIZES = [#byte_lengths];
                                    let mut max = 0;
                                    let mut i = 0;
                                    while i < SIZES.len() {
                                        if SIZES[i] > max {
                                            max = SIZES[i];
                                        }
                                        i += 1;
                                    }
                                    max
                                };
                                #module_content
                            }
                        },
                        id_getters,
                        address_getters,
                        quote! {
                            #module_name::MAX_BYTE_SIZE,
                        },
                    ]
                }
                _ => panic!("module should only contain other modules and structs"),
            }
        })
        .fold(from_fn(|_| TokenStream::new()), |acc, src| {
            zip(acc, src)
                .map(|(mut acc, src)| {
                    acc.extend(src);
                    acc
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap()
        })
}

pub fn impl_macro(ast: syn::Item, mut id: u16) -> TokenStream {
    let syn::Item::Mod(telem_defnition) = ast else {
        panic!("telemetry defintion is not a module");
    };

    let root_mod_ident = telem_defnition.ident;
    let Some(root_mod_content) = telem_defnition.content else {
        panic!("module is empty");
    };
    let start_id = id;
    let id_ref = &mut id;

    let [module_content, id_getters, address_getters, byte_lengths] =
        generate_module_recursive(vec![root_mod_ident.clone()], id_ref, &root_mod_content.1);

    quote! {
        pub mod #root_mod_ident {
            use tmtc_system::{TelemetryDefinition, _internal::*, NotFoundError};
            pub const fn from_id(id: u16) -> Result<&'static dyn TelemetryDefinition, NotFoundError> {
                match id {
                    #id_getters
                    _ => Err(NotFoundError)
                }
            }
            pub const fn from_address(address: &str) -> Result<&'static dyn TelemetryDefinition, NotFoundError> {
                match address {
                    #address_getters
                    _ => Err(NotFoundError)
                }
            }
            pub const fn id_range() -> (u16, u16) {
                (#start_id, #id_ref)
            }
            pub const MAX_BYTE_SIZE: usize = {
                let SIZES = [#byte_lengths];
                let mut max = 0;
                let mut i = 0;
                while i < SIZES.len() {
                    if SIZES[i] > max {
                        max = SIZES[i];
                    }
                    i += 1;
                }
                max
            };
            #module_content
        }
    }
}
