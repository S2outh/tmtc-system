use std::iter::once;

use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Item, Meta, Token, punctuated::Punctuated
};

const TM_VALUE_MACRO_NAME: &str = "tmv";

fn generate_module_recursive(address: Vec<syn::Ident>, items: &Vec<Item>) -> (TokenStream, TokenStream, TokenStream) {
    items.iter()
        .map(|v| {
            match v {
                syn::Item::Struct(v) => {
                    let attr_content = v.attrs
                        .iter()
                        .find(|attr| 
                            attr.path()
                            .is_ident(TM_VALUE_MACRO_NAME)
                        )
                        .expect(&format!("Enum variant {} has no {} attribute", &v.ident, TM_VALUE_MACRO_NAME))
                        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_separated_nonempty)
                        .expect(&format!("Could not parse {} attribute parameters", TM_VALUE_MACRO_NAME));
                    let ty = &v.ident;
                    let ty_addr: TokenStream = address
                        .iter()
                        .skip(1)
                        .chain(once(ty))
                        .map(|i| i.to_token_stream())
                        .intersperse(quote!(::))
                        .collect();
                    let tmty = attr_content.get(0)
                        .expect(&format!("{} attribute must contain at least a type and an id parameter", TM_VALUE_MACRO_NAME))
                        .require_path_only()
                        .expect(&format!("first {} attribute parameter must be a type path", TM_VALUE_MACRO_NAME));
                    let id = attr_content
                        .iter()
                        .filter_map(|m| m.require_name_value().ok())
                        .filter(|m| m.path.get_ident().filter(|p| *p == "id").is_some())
                        .map(|m| if let syn::Expr::Lit(value) = &m.value { value } else { panic!("unexpected attribute value type") })
                        .next().expect("no id specified");
                    let str_base_addr: String = address
                        .iter()
                        .map(|i| i.to_string())
                        .intersperse(String::from("."))
                        .collect();
                    let address = format!("{}.{}", str_base_addr, attr_content
                        .iter()
                        .filter_map(|m| m.require_name_value().ok())
                        .filter(|m| m.path.get_ident().filter(|p| *p == "address").is_some())
                        .map(|m| if let syn::Expr::Lit(value) = &m.value { value } else { panic!("unexpected attribute value type") })
                        .map(|m| if let syn::Lit::Str(str) = &m.lit { str } else { panic!("address should be a string") })
                        .next().map(|str| str.value()).unwrap_or_else(|| ty.to_string().to_snake_case()));
                    (
                        quote!{
                            pub struct #ty;
                            impl const DynTelemetryDefinition for #ty {
                                fn id(&self) -> u32 { <Self as TelemetryDefinition>::ID }
                                fn address(&self) -> &str { #address }
                            }
                            impl TelemetryDefinition for #ty {
                                type TMValueType = #tmty;
                                const ID: u32 = #id;
                            }

                        },
                        quote!{
                            #id => &#ty_addr,
                        },
                        quote!{
                            #address => &#ty_addr,
                        },
                    )
                },
                syn::Item::Mod(v) => {
                    let module_name = v.ident.clone();
                    let mut address = address.clone();
                    address.push(module_name.clone());
                    let (module_content, id_getters, address_getters)
                        = generate_module_recursive(address, &v.content.as_ref().expect("module sould not be empty").1);
                    (
                        quote! { 
                            pub mod #module_name {
                                use super::*;
                                #module_content
                            }
                        },
                        id_getters,
                        address_getters,
                    )
                },
                _ => panic!("module should only contain other modules and structs"),
            }
        }).map(|v| (v.0.to_token_stream(), v.1.to_token_stream(), v.2.to_token_stream())).collect()
}

pub fn impl_macro(ast: syn::Item) -> TokenStream {
    let syn::Item::Mod(telem_defnition) = ast else {
        panic!("telemetry defintion is not a module");
    };

    let root_mod_ident = telem_defnition.ident;
    let Some(root_mod_content) = telem_defnition.content else {
        panic!("module is empty");
    };

    let (module_content, id_getters, address_getters)
        = generate_module_recursive(vec![root_mod_ident.clone()], &root_mod_content.1);

    quote! {
        pub mod #root_mod_ident {
            use tmtc_system::{internal::TelemetryDefinition, DynTelemetryDefinition};
            pub const fn from_id(id: u32) -> &'static dyn DynTelemetryDefinition {
                match id {
                    #id_getters
                    _ => panic!("id does not exist")
                }
            }

            pub fn from_address(address: &str) -> &'static dyn DynTelemetryDefinition {
                match address {
                    #address_getters
                    _ => panic!("address does not exist")
                }
            }
            #module_content
        }
    }
}
