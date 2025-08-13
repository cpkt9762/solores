//! 非 Anchor Events 模板
//!
//! 为非 Anchor 合约生成 Events 相关代码，使用 1 字节 discriminator 或无 discriminator

use proc_macro2::TokenStream;
use quote::quote;
use convert_case::{Case, Casing};
use heck::ToShoutySnakeCase;

use crate::idl_format::non_anchor_idl::NonAnchorIdl;
use crate::templates::{TemplateGenerator, EventsTemplateGenerator};
use crate::templates::common::{doc_generator::DocGenerator, naming_converter::NamingConverter};
use crate::utils::{generate_pubkey_serde_attr};
use std::cell::RefCell;

/// 非 Anchor Events 模板
pub struct NonAnchorEventsTemplate<'a> {
    pub idl: &'a NonAnchorIdl,
    naming_converter: RefCell<NamingConverter>,
}

impl<'a> NonAnchorEventsTemplate<'a> {
    /// 创建新的非 Anchor Events 模板
    pub fn new(idl: &'a NonAnchorIdl) -> Self {
        Self { 
            idl,
            naming_converter: RefCell::new(NamingConverter::new()),
        }
    }

    /// 使用NamingConverter转换字段名并生成serde属性
    fn convert_field_name_with_serde(&self, original_name: &str) -> (String, TokenStream) {
        let snake_field_name = self.naming_converter.borrow_mut().convert_field_name(original_name);
        let serde_attr = if snake_field_name != original_name {
            quote! { #[cfg_attr(feature = "serde", serde(rename = #original_name))] }
        } else { 
            quote! {} 
        };
        (snake_field_name, serde_attr)
    }

    /// 检查字段类型是否为 Pubkey
    fn is_field_pubkey_type(field_type: &crate::idl_format::non_anchor_idl::NonAnchorFieldType) -> bool {
        match field_type {
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Basic(s) => {
                matches!(s.as_str(), "publicKey" | "pubkey" | "Pubkey")
            },
            _ => false
        }
    }
    
    /// 检查字符串字段类型是否为 Pubkey
    fn is_string_field_pubkey_type(type_str: &str) -> bool {
        matches!(type_str, "publicKey" | "pubkey" | "Pubkey")
    }

    /// 生成事件结构体
    pub fn generate_event_structs(&self) -> TokenStream {
        let events = self.idl.events.as_deref().unwrap_or(&[]);
        if events.is_empty() {
            return quote! {};
        }

        let event_structs = events.iter().map(|event| {
            let struct_name = syn::Ident::new(&event.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let doc_comments = DocGenerator::generate_non_anchor_event_docs(event);
            
            // 强制初始化字段分配缓存
            let _force_init = self.idl.get_field_allocation();
            
            // NonAnchor字段生成优先级：direct fields → field allocation → empty structures
            let struct_fields = if let Some(event_fields) = &event.fields {
                // 优先级1：直接使用event.fields
                let field_tokens = event_fields.iter().map(|field| {
                    let (snake_field_name, serde_attr) = self.convert_field_name_with_serde(&field.name);
                    let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                    let field_type = Self::convert_idl_type_to_rust(&field.field_type);
                    let field_docs = DocGenerator::generate_field_docs(&field.docs);
                    
                    // 检查是否为 Pubkey 类型，如果是则添加特殊的 serde 属性
                    let pubkey_serde_attr = if Self::is_field_pubkey_type(&field.field_type) {
                        generate_pubkey_serde_attr()
                    } else {
                        quote! {}
                    };
                    
                    quote! {
                        #field_docs
                        #serde_attr
                        #pubkey_serde_attr
                        pub #field_name: #field_type,
                    }
                });
                Some(quote! { #(#field_tokens)* })
            } else if let Some(allocated_fields) = self.idl.get_event_allocated_fields(&event.name) {
                // 优先级2：从字段分配获取字段
                let field_tokens = allocated_fields.iter().map(|field_def| {
                    let (snake_field_name, serde_attr) = self.convert_field_name_with_serde(&field_def.name);
                    let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                    // 使用改进的类型转换逻辑
                    let field_type = Self::convert_field_definition_type_to_rust(&field_def.field_type);
                    let field_docs = if field_def.docs.is_empty() { 
                        quote! {} 
                    } else { 
                        DocGenerator::generate_doc_comments(&Some(field_def.docs.clone())) 
                    };
                    
                    // 检查字符串字段类型是否为 Pubkey
                    let pubkey_serde_attr = if Self::is_string_field_pubkey_type(&field_def.field_type) {
                        generate_pubkey_serde_attr()
                    } else {
                        quote! {}
                    };
                    
                    quote! {
                        #field_docs
                        #serde_attr
                        #pubkey_serde_attr
                        pub #field_name: #field_type,
                    }
                });
                Some(quote! { #(#field_tokens)* })
            } else {
                None // 优先级3：无任何字段定义，使用空结构体
            };

            if let Some(fields) = struct_fields {
                quote! {
                    #doc_comments
                    #[derive(Clone, Debug, borsh::BorshDeserialize, borsh::BorshSerialize, PartialEq, Eq)]
                    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                    pub struct #struct_name {
                        // NonAnchor Events不需要discriminator字段
                        #fields
                    }
                }
            } else {
                // 优先级3：创建空结构体
                quote! {
                    #doc_comments
                    #[derive(Clone, Debug, borsh::BorshDeserialize, borsh::BorshSerialize, PartialEq, Eq, Default)]
                    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                    pub struct #struct_name;
                }
            }
        });

        quote! {
            #(#event_structs)*
        }
    }
    
    /// 转换 IDL 类型为 Rust 类型 (非Anchor版本)
    fn convert_idl_type_to_rust(idl_type: &crate::idl_format::non_anchor_idl::NonAnchorFieldType) -> TokenStream {
        match idl_type {
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Basic(type_str) => {
                let type_ident = match type_str.as_str() {
                    "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "u64" | "i64" | "u128" | "i128" => {
                        syn::Ident::new(type_str, proc_macro2::Span::call_site())
                    },
                    "bool" => syn::Ident::new("bool", proc_macro2::Span::call_site()),
                    "string" | "String" => syn::Ident::new("String", proc_macro2::Span::call_site()),
                    "publicKey" | "pubkey" | "Pubkey" => syn::Ident::new("Pubkey", proc_macro2::Span::call_site()),
                    _ => syn::Ident::new(type_str, proc_macro2::Span::call_site()),
                };
                quote! { #type_ident }
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Option { option } => {
                let inner_type = Self::convert_idl_type_to_rust(option);
                quote! { Option<#inner_type> }
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Vec { vec } => {
                let inner_type = Self::convert_idl_type_to_rust(vec);
                quote! { Vec<#inner_type> }
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Array { array } => {
                let (inner_type, size) = array;
                let inner_type_token = Self::convert_idl_type_to_rust(inner_type);
                let size_literal = proc_macro2::Literal::usize_unsuffixed(*size);
                quote! { [#inner_type_token; #size_literal] }
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Defined { defined } => {
                // 使用完整路径引用types模块中的类型
                let type_path = format!("crate::types::{}", defined);
                let type_path: syn::Path = syn::parse_str(&type_path).unwrap();
                quote! { #type_path }
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Complex { kind, params } => {
                // 处理复合类型，如 Vec<T>, Option<T>, [T; N] 等 (Legacy支持)
                match kind.as_str() {
                    "Vec" => {
                        if let Some(params) = params {
                            if let Some(inner_type) = params.get(0) {
                                if let Some(inner_str) = inner_type.as_str() {
                                    let inner_type_token = Self::convert_idl_type_to_rust(&crate::idl_format::non_anchor_idl::NonAnchorFieldType::Basic(inner_str.to_string()));
                                    quote! { Vec<#inner_type_token> }
                                } else {
                                    quote! { Vec<u8> } // fallback
                                }
                            } else {
                                quote! { Vec<u8> } // fallback
                            }
                        } else {
                            quote! { Vec<u8> } // fallback
                        }
                    },
                    "Option" => {
                        if let Some(params) = params {
                            if let Some(inner_type) = params.get(0) {
                                if let Some(inner_str) = inner_type.as_str() {
                                    let inner_type_token = Self::convert_idl_type_to_rust(&crate::idl_format::non_anchor_idl::NonAnchorFieldType::Basic(inner_str.to_string()));
                                    quote! { Option<#inner_type_token> }
                                } else {
                                    quote! { Option<u8> } // fallback
                                }
                            } else {
                                quote! { Option<u8> } // fallback
                            }
                        } else {
                            quote! { Option<u8> } // fallback
                        }
                    },
                    _ => {
                        let type_ident = syn::Ident::new(kind, proc_macro2::Span::call_site());
                        quote! { #type_ident }
                    }
                }
            },
        }
    }

    /// 生成事件包装器 (非Anchor版本)
    pub fn generate_event_wrappers(&self) -> TokenStream {
        let events = self.idl.events.as_deref().unwrap_or(&[]);
        if events.is_empty() {
            return quote! {};
        }

        let event_wrappers = events.iter().enumerate().map(|(index, event)| {
            let struct_name = syn::Ident::new(&event.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let wrapper_name = syn::Ident::new(&format!("{}Event", event.name.to_case(Case::Pascal)), proc_macro2::Span::call_site());
            let discm_const_name = syn::Ident::new(
                &format!("{}_EVENT_DISCM", event.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );
            
            quote! {
                /// Event wrapper for #struct_name with 1-byte discriminator for non-Anchor serialization
                #[derive(Clone, Debug, PartialEq)]
                pub struct #wrapper_name(pub #struct_name);
                
                impl borsh::BorshSerialize for #wrapper_name {
                    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                        // Serialize 1-byte discriminator for non-Anchor events
                        #discm_const_name.serialize(writer)?;
                        self.0.serialize(writer)
                    }
                }
                
                impl #wrapper_name {
                    pub fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
                        let maybe_discm = u8::deserialize(buf)?;
                        if maybe_discm != #discm_const_name {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!(
                                    "1-byte discriminator does not match. Expected: {}. Received: {}",
                                    #discm_const_name, maybe_discm
                                ),
                            ));
                        }
                        Ok(Self(#struct_name::deserialize(buf)?))
                    }
                }
            }
        });

        quote! {
            #(#event_wrappers)*
        }
    }

    /// 生成 discriminator 常量 (非Anchor版本)
    pub fn generate_discriminator_constants(&self) -> TokenStream {
        let events = self.idl.events.as_deref().unwrap_or(&[]);
        if events.is_empty() {
            return quote! {};
        }

        let constants = events.iter().enumerate().map(|(index, event)| {
            let const_name = syn::Ident::new(
                &format!("{}_EVENT_DISCM", event.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );
            
            // Use simple index-based 1-byte discriminator for non-Anchor events
            let discriminator_value = index as u8;

            quote! {
                /// 1-byte discriminator constant for non-Anchor event identification
                pub const #const_name: u8 = #discriminator_value;
            }
        });

        quote! {
            #(#constants)*
        }
    }

    /// 为单个event生成完整的文件内容（NonAnchor版本）
    pub fn generate_single_event_file(&self, event: &crate::idl_format::non_anchor_idl::NonAnchorEvent, index: usize) -> TokenStream {
        let event_name = syn::Ident::new(&event.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
        let wrapper_name = syn::Ident::new(&format!("{}Event", event.name.to_case(Case::Pascal)), proc_macro2::Span::call_site());
        let const_name = syn::Ident::new(&format!("{}_EVENT_DISCM", event.name.to_shouty_snake_case()), proc_macro2::Span::call_site());
        
        let discriminator_value = index as u8;
        let doc_comments = DocGenerator::generate_doc_comments(&event.docs);
        let event_name_str = &event.name;

        // 强制初始化字段分配缓存
        let _force_init = self.idl.get_field_allocation();

        // 生成事件结构体字段（NonAnchor版本不使用discriminator）
        // 优先级：direct fields → field allocation → empty structures
        let (event_fields, default_fields) = if let Some(fields) = &event.fields {
            // 优先级1：直接使用event.fields
            let fields_tokens = fields.iter().map(|field| {
                let (snake_field_name, serde_attr) = self.convert_field_name_with_serde(&field.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                let field_type = self.convert_event_field_type(&field.field_type);
                let field_docs = DocGenerator::generate_field_docs(&field.docs);
                
                // 检查是否为 Pubkey 类型，如果是则添加特殊的 serde 属性
                let pubkey_serde_attr = if Self::is_field_pubkey_type(&field.field_type) {
                    generate_pubkey_serde_attr()
                } else {
                    quote! {}
                };
                
                quote! {
                    #field_docs
                    #serde_attr
                    #pubkey_serde_attr
                    pub #field_name: #field_type,
                }
            });
            let default_values = fields.iter().map(|field| {
                let (snake_field_name, _) = self.convert_field_name_with_serde(&field.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                quote! { #field_name: Default::default(), }
            });
            (
                quote! { #(#fields_tokens)* },
                quote! { #(#default_values)* }
            )
        } else if let Some(allocated_fields) = self.idl.get_event_allocated_fields(&event.name) {
            // 优先级2：从字段分配获取字段
            let fields_tokens = allocated_fields.iter().map(|field_def| {
                let (snake_field_name, serde_attr) = self.convert_field_name_with_serde(&field_def.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                // 使用改进的类型转换逻辑
                let field_type = Self::convert_field_definition_type_to_rust(&field_def.field_type);
                let field_docs = if field_def.docs.is_empty() { 
                    quote! {} 
                } else { 
                    DocGenerator::generate_doc_comments(&Some(field_def.docs.clone())) 
                };
                
                // 检查字符串字段类型是否为 Pubkey
                let pubkey_serde_attr = if Self::is_string_field_pubkey_type(&field_def.field_type) {
                    generate_pubkey_serde_attr()
                } else {
                    quote! {}
                };
                
                quote! {
                    #field_docs
                    #serde_attr
                    #pubkey_serde_attr
                    pub #field_name: #field_type,
                }
            });
            let default_values = allocated_fields.iter().map(|field_def| {
                let (snake_field_name, _) = self.convert_field_name_with_serde(&field_def.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                quote! { #field_name: Default::default(), }
            });
            (
                quote! { #(#fields_tokens)* },
                quote! { #(#default_values)* }
            )
        } else {
            // 优先级3：无任何字段定义，使用空结构体
            (quote! {}, quote! {})
        };

        let event_doc_comment = format!("Event: {} (NonAnchor)", event_name_str);
        
        // 检查IDL是否有types字段
        let has_types_module = self.idl.types.as_ref().map_or(false, |types| !types.is_empty());
        
        // 生成导入语句
        let imports = if has_types_module {
            quote! {
                use crate::types::*;
                use solana_pubkey::Pubkey;
            }
        } else {
            quote! {
                use solana_pubkey::Pubkey;
            }
        };
        
        quote! {
            #![doc = #event_doc_comment]
            #doc_comments
            
            #imports
            
            // Constants
            pub const #const_name: u8 = #discriminator_value;
            
            // Event Structure
            #doc_comments
            #[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Clone, Debug, PartialEq)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            pub struct #event_name {
                #event_fields
            }

            impl Default for #event_name {
                fn default() -> Self {
                    Self {
                        #default_fields
                    }
                }
            }

            // Event Wrapper
            #doc_comments
            #[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Clone, Debug, PartialEq)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            pub struct #wrapper_name {
                pub discriminator: u8,
                pub data: #event_name,
            }

            impl Default for #wrapper_name {
                fn default() -> Self {
                    Self {
                        discriminator: #const_name,
                        data: #event_name::default(),
                    }
                }
            }

            impl #event_name {
                pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
                    borsh::to_vec(self)
                }
                
                pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
                    borsh::BorshDeserialize::deserialize(&mut &data[..])
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                }
            }

            impl #wrapper_name {
                pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
                    borsh::to_vec(self)
                }
                
                pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
                    if data.is_empty() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Event data too short for discriminator",
                        ));
                    }
                    
                    if data[0] != #const_name {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "Discriminator mismatch. Expected: {}, got: {}",
                                #const_name,
                                data[0]
                            ),
                        ));
                    }
                    
                    borsh::BorshDeserialize::deserialize(&mut &data[..])
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                }
            }
        }
    }

    /// 转换事件字段类型（NonAnchor版本）
    fn convert_event_field_type(&self, field_type: &crate::idl_format::non_anchor_idl::NonAnchorFieldType) -> TokenStream {
        Self::convert_idl_type_to_rust(field_type)
    }
}

impl<'a> EventsTemplateGenerator for NonAnchorEventsTemplate<'a> {
    fn generate_event_structures(&self) -> TokenStream {
        self.generate_event_structs()
    }

    fn generate_event_wrappers(&self) -> TokenStream {
        self.generate_event_wrappers()
    }

    fn generate_event_constants(&self) -> TokenStream {
        self.generate_discriminator_constants()
    }
}

impl<'a> TemplateGenerator for NonAnchorEventsTemplate<'a> {
    fn get_standard_module_name(&self) -> &'static str {
        "events"
    }

    fn gen_files(&self) -> Vec<(String, TokenStream)> {
        let events = self.idl.events.as_deref().unwrap_or(&[]);
        if events.is_empty() {
            return vec![("mod.rs".to_string(), quote! {
                //! Empty events module - no events found in IDL
            })];
        }
        
        let mut files = Vec::new();
        
        // 为每个event生成独立文件
        for (index, event) in events.iter().enumerate() {
            let file_name = format!("{}.rs", event.name.to_case(Case::Snake));
            let file_content = self.generate_single_event_file(event, index);
            files.push((file_name, file_content));
        }
        
        files
    }

    fn gen_mod_file(&self) -> TokenStream {
        let events = self.idl.events.as_deref().unwrap_or(&[]);
        if events.is_empty() {
            return quote! {
                //! Events module - no events found in IDL
            };
        }
        
        // 生成模块声明和重新导出
        let module_declarations = events.iter().map(|event| {
            let module_name = syn::Ident::new(&event.name.to_case(Case::Snake), proc_macro2::Span::call_site());
            quote! {
                pub mod #module_name;
            }
        });
        
        let re_exports = events.iter().map(|event| {
            let module_name = syn::Ident::new(&event.name.to_case(Case::Snake), proc_macro2::Span::call_site());
            quote! {
                pub use #module_name::*;
            }
        });
        
        quote! {
            //! Non-Anchor events module
            //! Generated event definitions with 1-byte discriminator support
            //! Each event is defined in its own file
            
            #(#module_declarations)*
            
            // Re-export all event items
            #(#re_exports)*
        }
    }
}

impl<'a> NonAnchorEventsTemplate<'a> {
    /// 将字段分配的字符串类型转换为Rust类型
    fn convert_field_definition_type_to_rust(type_str: &str) -> TokenStream {
        // 处理简单类型映射
        let rust_type = match type_str {
            "bool" => "bool",
            "u8" => "u8",
            "u16" => "u16", 
            "u32" => "u32",
            "u64" => "u64",
            "u128" => "u128",
            "i8" => "i8",
            "i16" => "i16",
            "i32" => "i32", 
            "i64" => "i64",
            "i128" => "i128",
            "String" | "string" => "String",
            "Pubkey" | "publicKey" | "pubkey" => "Pubkey",
            _ => type_str, // 保持原有类型名称
        };

        let type_ident = syn::Ident::new(rust_type, proc_macro2::Span::call_site());
        quote! { #type_ident }
    }
}