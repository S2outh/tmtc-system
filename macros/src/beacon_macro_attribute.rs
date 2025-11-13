use proc_macro2::TokenStream;
use quote::{quote};
use syn::{Token,
    punctuated::Punctuated,
    Meta
};

const TM_VALUE_MACRO_NAME: &str = "tmv";

pub fn impl_macro(beacon_path: syn::Path, ast: syn::DeriveInput) -> TokenStream {
    let beacon_definition_name = &ast.ident;
    let beacon_name = &beacon_path.get_ident().unwrap();
    let syn::Data::Enum(bd_enum) = ast.data else {
        panic!("beacon defintion is not an enum");
    };
    
    let bd_enum_variants = bd_enum.variants
        .iter()
        .map(|v| {
            let mut var = v.clone();
            var.attrs = vec![];
            var
        });
    
    let attrs_metas = bd_enum.variants
        .iter()
        .map(|v| 
            v.attrs
                .iter()
                .find(|attr| 
                    attr.path()
                    .is_ident(TM_VALUE_MACRO_NAME)
                )
                .expect(&format!("Enum variant {} has no {} attribute", &v.ident, TM_VALUE_MACRO_NAME))
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_separated_nonempty)
                .expect(&format!("Could not parse {} attribute parameters", TM_VALUE_MACRO_NAME))
        );
    let enum_types = bd_enum.variants
        .iter()
        .map(|v| {
            assert_eq!(v.fields.len(), 1);
            v.fields.iter().next().unwrap()
        });
    let get_cell_functions = bd_enum.variants
        .iter()
        .map(|v| &v.ident )
        .enumerate()
        .map(|(i, v)| quote!{
            #v(tm_value) => {
                let bytes = tm_value.to_bytes();
                let pos = Self::get_pos(#i);
                storage[pos..(pos+bytes.len())].copy_from_slice(&bytes);
            }
        });
    let enum_len: usize = bd_enum.variants.len();

    quote! {
        enum #beacon_definition_name {
            #(#bd_enum_variants),*
        }

        impl #beacon_definition_name {
            const SIZES: [usize; #enum_len] = [#(size_of::<<#enum_types as TMValue>::Bytes>()),*];
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

        type #beacon_name = Beacon<#beacon_definition_name, {#beacon_definition_name::len()}>;

        impl BeaconDefinition for #beacon_definition_name {
            fn transfer_cell(&self, storage: &mut [u8]) {
                match self {
                    #(#get_cell_functions),*
                }
            }
        }
    }
}
