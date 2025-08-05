use heck::{ToPascalCase, ToShoutySnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::idl_format::anchor::typedefs::NamedType;
use crate::utils::conditional_pascal_case;

#[derive(Deserialize, Debug)]
pub struct NamedAccount(pub NamedType);

impl NamedAccount {
    pub fn to_token_stream(&self, cli_args: &crate::Args) -> TokenStream {
        // Default to Anchor-style for backward compatibility
        self.to_token_stream_with_context(cli_args, true)
    }

    pub fn to_token_stream_with_context(&self, cli_args: &crate::Args, is_anchor_contract: bool) -> TokenStream {
        self.to_token_stream_with_typedefs(cli_args, is_anchor_contract, &[])
    }

    pub fn to_token_stream_with_typedefs(&self, cli_args: &crate::Args, is_anchor_contract: bool, typedefs: &[NamedType]) -> TokenStream {
        let name = &self.0.name;
        let name_ident = format_ident!("{}", conditional_pascal_case(name));
        
        // Try to find struct definition either in self or in typedefs array
        let struct_def = if self.0.r#type.is_some() {
            // Use struct definition from self
            self.0.to_token_stream(cli_args)
        } else {
            // Try to find matching struct definition in typedefs array
            if let Some(typedef) = typedefs.iter().find(|t| t.name == *name) {
                log::debug!("  ├─ 从types数组找到账户'{}'的结构体定义", name);
                typedef.to_token_stream(cli_args)
            } else {
                log::debug!("  ├─ 账户'{}'没有找到结构体定义，生成占位符结构体", name);
                self.0.to_token_stream(cli_args)
            }
        };

        log::debug!("🏗️ 生成账户 '{}': is_anchor_contract={}", name, is_anchor_contract);

        if is_anchor_contract {
            // Generate Anchor-style account with 8-byte discriminator
            log::debug!("  └─ 使用Anchor样式(8字节discriminator)");
            self.generate_anchor_account_impl(name, &name_ident, &struct_def, typedefs)
        } else {
            // Generate yellowstone-vixen style account without discriminator
            log::debug!("  └─ 使用yellowstone-vixen样式(无discriminator)");
            self.generate_non_anchor_account_impl(name, &name_ident, &struct_def)
        }
    }

    fn generate_anchor_account_impl(&self, name: &str, name_ident: &proc_macro2::Ident, struct_def: &TokenStream, typedefs: &[NamedType]) -> TokenStream {
        let account_discm_ident = format_ident!("{}_ACCOUNT_DISCM", name.to_shouty_snake_case());
        
        // Use discriminator from IDL if available, otherwise calculate from name
        let discm = if let Some(discm_values) = &self.0.discriminator {
            // Use discriminator from IDL
            let discm_u8: [u8; 8] = discm_values.clone().try_into()
                .expect("Discriminator should be exactly 8 bytes");
            discm_u8
        } else {
            // Calculate discriminator from name (pre-image: "account:{AccountStructName}")
            <[u8; 8]>::try_from(
                &Sha256::digest(format!("account:{}", name.to_pascal_case()).as_bytes()).as_slice()
                    [..8],
            )
            .unwrap()
        };
        let discm_tokens: TokenStream = format!("{:?}", discm).parse().unwrap();

        // Check if this is a complete struct definition or just a reference
        // Also check if struct_def contains actual fields (not just empty/placeholder)
        let has_complete_definition = self.0.r#type.is_some() 
            && matches!(self.0.r#type.as_ref(), Some(crate::idl_format::anchor::typedefs::TypedefType::r#struct(_)))
            || !struct_def.to_string().contains("pub data: Vec<u8>");
        
        log::debug!("  ├─ has_complete_definition={}, self.0.r#type.is_some()={}", has_complete_definition, self.0.r#type.is_some());
        log::debug!("  ├─ struct_def contains placeholder data: {}", struct_def.to_string().contains("pub data: Vec<u8>"));
        
        // For Anchor accounts, we need to merge the discriminator field with the actual struct fields
        let needs_discriminator_field = has_complete_definition;
        
        if has_complete_definition {
            // Generate complete struct definition with discriminator field included
            // Need to modify the struct to include discriminator as first field
            let modified_struct_def = if needs_discriminator_field {
                // Extract fields from struct_def and prepend discriminator field
                self.generate_struct_with_discriminator(&struct_def, typedefs)
            } else {
                struct_def.clone()
            };
            
            quote! {
                pub const #account_discm_ident: [u8; 8] = #discm_tokens;

                #modified_struct_def

                impl #name_ident {
                    // 统一API: 长度常量 (struct includes discriminator)
                    pub const LEN: usize = std::mem::size_of::<Self>();

                    // 统一API: 序列化到Vec (直接序列化整个结构体)
                    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
                        borsh::to_vec(self)
                    }

                    // 统一API: 从字节解析 (验证discriminator + 反序列化完整数据)
                    pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
                        if data.len() < 8 {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Account data too short for discriminator"
                            ));
                        }

                        // Anchor合约: 验证discriminator
                        if &data[0..8] != #account_discm_ident {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Discriminator mismatch. Expected: {:?}, got: {:?}", 
                                       #account_discm_ident, &data[0..8])
                            ));
                        }

                        // 反序列化完整数据 (包含discriminator)
                        borsh::from_slice(data)
                    }
                }
            }
        } else {
            // This account type has no complete definition in IDL
            // Generate a placeholder struct with unified API and discriminator handling
            quote! {
                pub const #account_discm_ident: [u8; 8] = #discm_tokens;

                // Generate a placeholder struct since no definition is available
                #[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                pub struct #name_ident {
                    // Placeholder field - actual structure should be defined based on program analysis
                    pub data: Vec<u8>,
                }

                impl Default for #name_ident {
                    fn default() -> Self {
                        Self {
                            data: Vec::new(),
                        }
                    }
                }

                impl #name_ident {
                    // 统一API: 长度常量 (动态长度，最小8字节discriminator)
                    pub const LEN: usize = 8; // Minimum length for discriminator

                    // 统一API: 序列化到Vec (包含discriminator前缀)
                    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
                        let mut data = #account_discm_ident.to_vec();
                        data.extend_from_slice(&borsh::to_vec(self)?);
                        Ok(data)
                    }

                    // 统一API: 从字节解析 (验证discriminator + 反序列化)
                    pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
                        if data.len() < 8 {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Account data too short for discriminator"
                            ));
                        }

                        // Anchor合约: 验证discriminator
                        if &data[0..8] != #account_discm_ident {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Discriminator mismatch. Expected: {:?}, got: {:?}", 
                                       #account_discm_ident, &data[0..8])
                            ));
                        }

                        // 反序列化除discriminator外的数据
                        let account_data = &data[8..];
                        Ok(Self {
                            data: account_data.to_vec(),
                        })
                    }
                }
            }
        }
    }

    fn generate_struct_with_discriminator(&self, struct_def: &TokenStream, typedefs: &[NamedType]) -> TokenStream {
        let name = &self.0.name;
        let name_ident = format_ident!("{}", conditional_pascal_case(name));
        let account_discm_ident = format_ident!("{}_ACCOUNT_DISCM", name.to_shouty_snake_case());
        
        // Generate documentation comments if available
        let doc_comments = if let Some(docs) = &self.0.docs {
            let doc_tokens: Vec<TokenStream> = docs
                .iter()
                .filter(|doc| !doc.trim().is_empty())
                .map(|doc| {
                    let doc_str = doc.trim();
                    quote! { #[doc = #doc_str] }
                })
                .collect();
            quote! { #(#doc_tokens)* }
        } else {
            quote! {}
        };
        
        // Try to get the struct fields either from self or from typedefs array
        let fields_token = if let Some(typedef_type) = &self.0.r#type {
            // Account has its own type definition
            if let crate::idl_format::anchor::typedefs::TypedefType::r#struct(typedef_struct) = typedef_type {
                let fields = &typedef_struct.fields;
                quote! { #(#fields),* }
            } else {
                quote! {}
            }
        } else {
            // Try to find the typedef in the types array
            if let Some(typedef) = typedefs.iter().find(|t| t.name == *name) {
                if let Some(crate::idl_format::anchor::typedefs::TypedefType::r#struct(typedef_struct)) = &typedef.r#type {
                    let fields = &typedef_struct.fields;
                    quote! { #(#fields),* }
                } else {
                    quote! {}
                }
            } else {
                quote! {}
            }
        };
        
        // Generate default field values
        let default_fields = if let Some(typedef_type) = &self.0.r#type {
            // Account has its own type definition
            if let crate::idl_format::anchor::typedefs::TypedefType::r#struct(typedef_struct) = typedef_type {
                typedef_struct.fields.iter().map(|field| {
                    let field_name = format_ident!("{}", &field.name);
                    
                    // Generate appropriate default value based on field type
                    let default_value = match &field.r#type {
                        crate::idl_format::anchor::typedefs::TypedefFieldType::array(array_type) => {
                            let array_size = proc_macro2::Literal::usize_unsuffixed(array_type.1 as usize);
                            match &*array_type.0 {
                                crate::idl_format::anchor::typedefs::TypedefFieldType::PrimitiveOrPubkey(elem_type) => {
                                    match elem_type.as_str() {
                                        "u8" => {
                                            quote! { [0u8; #array_size] }
                                        },
                                        "u16" => {
                                            quote! { [0u16; #array_size] }
                                        },
                                        "u32" => {
                                            quote! { [0u32; #array_size] }
                                        },
                                        "u64" => {
                                            quote! { [0u64; #array_size] }
                                        },
                                        "i8" => {
                                            quote! { [0i8; #array_size] }
                                        },
                                        "i16" => {
                                            quote! { [0i16; #array_size] }
                                        },
                                        "i32" => {
                                            quote! { [0i32; #array_size] }
                                        },
                                        "i64" => {
                                            quote! { [0i64; #array_size] }
                                        },
                                        "bool" => {
                                            quote! { [false; #array_size] }
                                        },
                                        "publicKey" | "Pubkey" => {
                                            // For Pubkey arrays, we need special handling
                                            quote! { core::array::from_fn(|_| Pubkey::default()) }
                                        },
                                        _ => {
                                            quote! { core::array::from_fn(|_| Default::default()) }
                                        }
                                    }
                                },
                                _ => {
                                    quote! { core::array::from_fn(|_| Default::default()) }
                                }
                            }
                        },
                        _ => {
                            quote! { Default::default() }
                        }
                    };
                    
                    quote! { #field_name: #default_value }
                }).collect::<Vec<_>>()
            } else {
                vec![]
            }
        } else {
            // Try to find the typedef in the types array
            if let Some(typedef) = typedefs.iter().find(|t| t.name == *name) {
                if let Some(crate::idl_format::anchor::typedefs::TypedefType::r#struct(typedef_struct)) = &typedef.r#type {
                    typedef_struct.fields.iter().map(|field| {
                        let field_name = format_ident!("{}", &field.name);
                        
                        // Generate appropriate default value based on field type
                        let default_value = match &field.r#type {
                            crate::idl_format::anchor::typedefs::TypedefFieldType::array(array_type) => {
                                let array_size = proc_macro2::Literal::usize_unsuffixed(array_type.1 as usize);
                                match &*array_type.0 {
                                    crate::idl_format::anchor::typedefs::TypedefFieldType::PrimitiveOrPubkey(elem_type) => {
                                        match elem_type.as_str() {
                                            "u8" => {
                                                quote! { [0u8; #array_size] }
                                            },
                                            "u16" => {
                                                quote! { [0u16; #array_size] }
                                            },
                                            "u32" => {
                                                quote! { [0u32; #array_size] }
                                            },
                                            "u64" => {
                                                quote! { [0u64; #array_size] }
                                            },
                                            "i8" => {
                                                quote! { [0i8; #array_size] }
                                            },
                                            "i16" => {
                                                quote! { [0i16; #array_size] }
                                            },
                                            "i32" => {
                                                quote! { [0i32; #array_size] }
                                            },
                                            "i64" => {
                                                quote! { [0i64; #array_size] }
                                            },
                                            "bool" => {
                                                quote! { [false; #array_size] }
                                            },
                                            "publicKey" | "Pubkey" => {
                                                // For Pubkey arrays, we need special handling
                                                quote! { core::array::from_fn(|_| Pubkey::default()) }
                                            },
                                            _ => {
                                                quote! { core::array::from_fn(|_| Default::default()) }
                                            }
                                        }
                                    },
                                    _ => {
                                        quote! { core::array::from_fn(|_| Default::default()) }
                                    }
                                }
                            },
                            _ => {
                                quote! { Default::default() }
                            }
                        };
                        
                        quote! { #field_name: #default_value }
                    }).collect::<Vec<_>>()
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        };

        // Generate the struct with discriminator field and Default implementation
        quote! {
            #doc_comments
            #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, Eq)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            pub struct #name_ident {
                pub discriminator: [u8; 8],
                #fields_token
            }
            
            impl Default for #name_ident {
                fn default() -> Self {
                    Self {
                        discriminator: #account_discm_ident,
                        #(#default_fields),*
                    }
                }
            }
        }
    }

    fn generate_non_anchor_account_impl(&self, _name: &str, name_ident: &proc_macro2::Ident, struct_def: &TokenStream) -> TokenStream {
        // Generate yellowstone-vixen style implementation
        
        // For non-Anchor contracts, we check if this is a complete struct definition or just a reference
        let has_complete_definition = self.0.r#type.is_some() 
            && matches!(self.0.r#type.as_ref(), Some(crate::idl_format::anchor::typedefs::TypedefType::r#struct(_)));
        
        if has_complete_definition {
            // Generate complete struct definition with unified API
            quote! {
                #struct_def

                impl #name_ident {
                    // 统一API: 长度常量
                    pub const LEN: usize = std::mem::size_of::<Self>();

                    // 统一API: 序列化到Vec
                    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
                        borsh::to_vec(self)
                    }

                    // 统一API: 从字节解析 (非Anchor合约，无discriminator)
                    pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
                        if data.len() != Self::LEN {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Invalid data length. Expected: {}, got: {}", Self::LEN, data.len())
                            ));
                        }
                        
                        borsh::from_slice(data)
                    }
                }
            }
        } else {
            // This account type is defined elsewhere (in types module)
            // Just add unified API methods
            quote! {
                // Re-export the type from types module and add unified API
                pub use crate::types::#name_ident;

                impl #name_ident {
                    // 统一API: 长度常量
                    pub const LEN: usize = std::mem::size_of::<Self>();

                    // 统一API: 序列化到Vec
                    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
                        borsh::to_vec(self)
                    }

                    // 统一API: 从字节解析 (非Anchor合约，无discriminator)
                    pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
                        if data.len() != Self::LEN {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Invalid data length. Expected: {}, got: {}", Self::LEN, data.len())
                            ));
                        }
                        
                        borsh::from_slice(data)
                    }
                }
            }
        }
    }
}
