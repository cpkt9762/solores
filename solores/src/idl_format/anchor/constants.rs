use proc_macro2::TokenStream;
use quote::{quote, format_ident};

use crate::idl_format::IdlCodegenModule;
use super::Constant;

pub struct ConstantsCodegenModule<'a> {
    pub constants: &'a [Constant],
}

impl<'a> IdlCodegenModule for ConstantsCodegenModule<'a> {
    fn name(&self) -> &str {
        "constants"
    }

    fn gen_head(&self) -> TokenStream {
        quote! {
            //! Program constants
        }
    }

    fn gen_body(&self) -> TokenStream {
        let const_defs = self.constants.iter().map(|c| {
            let name = format_ident!("{}", c.name);
            let value = &c.value;
            
            // Handle different type formats
            match &c.const_type {
                serde_json::Value::String(type_str) => {
                    match type_str.as_str() {
                        "i32" => {
                            let value_int: i32 = value.parse().unwrap_or(0);
                            quote! { pub const #name: i32 = #value_int; }
                        },
                        "u32" => {
                            let value_int: u32 = value.parse().unwrap_or(0);
                            quote! { pub const #name: u32 = #value_int; }
                        },
                        "i64" => {
                            let value_int: i64 = value.parse().unwrap_or(0);
                            quote! { pub const #name: i64 = #value_int; }
                        },
                        "u64" => {
                            let value_int: u64 = value.parse().unwrap_or(0);
                            quote! { pub const #name: u64 = #value_int; }
                        },
                        _ => {
                            quote! {
                                // Unsupported type: #type_str
                            }
                        }
                    }
                },
                serde_json::Value::Object(obj) => {
                    // Handle {"defined": "usize"} format
                    if let Some(serde_json::Value::String(defined_type)) = obj.get("defined") {
                        match defined_type.as_str() {
                            "usize" => {
                                let value_int: usize = value.parse().unwrap_or(0);
                                quote! { pub const #name: usize = #value_int; }
                            },
                            _ => {
                                quote! {
                                    // Unsupported defined type: #defined_type
                                }
                            }
                        }
                    } else {
                        quote! {
                            // Unsupported object type format
                        }
                    }
                },
                _ => {
                    quote! {
                        // Unsupported type format
                    }
                }
            }
        });

        quote! {
            #(#const_defs)*
        }
    }
}