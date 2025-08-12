//! Anchor Events 模板
//!
//! 为 Anchor 合约生成 Events 相关代码，使用 8 字节 discriminator

use proc_macro2::TokenStream;
use quote::quote;
use convert_case::{Case, Casing};
use heck::ToShoutySnakeCase;
use sha2::{Digest};

use crate::idl_format::anchor_idl::Event;
use crate::idl_format::anchor_idl::AnchorType;
use crate::templates::{TemplateGenerator, EventsTemplateGenerator};
use crate::templates::common::{doc_generator::DocGenerator};
use crate::utils::{to_snake_case_with_serde, generate_pubkey_serde_attr};

/// Anchor Events 模板
pub struct AnchorEventsTemplate<'a> {
    pub idl: &'a crate::idl_format::anchor_idl::AnchorIdl,
}

impl<'a> AnchorEventsTemplate<'a> {
    /// 创建 Anchor Events 模板（统一使用字段分配机制）
    pub fn new(idl: &'a crate::idl_format::anchor_idl::AnchorIdl) -> Self {
        Self { idl }
    }

    /// 生成事件结构体
    pub fn generate_event_structs(&self) -> TokenStream {
        let events = self.idl.events.as_deref().unwrap_or(&[]);
        if events.is_empty() {
            return quote! {};
        }

        let event_structs = events.iter().filter_map(|event| {
            let struct_name = syn::Ident::new(&event.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            
            // 开始处理事件
            log::debug!("🎭 Events: 开始处理Event: {}", event.name);
            log::debug!("🎭 Events: Event '{}' fields状态: {:?}", event.name, 
                event.fields.as_ref().map(|f| format!("Some({}个字段)", f.len())).unwrap_or("None".to_string()));
            
            // 统一使用字段分配机制：优先使用事件直接字段，否则从字段分配中获取
            if let Some(event_fields) = &event.fields {
                // 事件有直接字段定义
                log::debug!("🎭 Events: Event '{}' 有直接字段定义，使用直接字段", event.name);
                let doc_comments = DocGenerator::generate_doc_comments(&event.docs);
                let struct_fields = event_fields.iter().map(|field| {
                    let (snake_field_name, serde_attr) = to_snake_case_with_serde(&field.name);
                    let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                    let field_type = Self::convert_idl_type_to_rust(&field.field_type);
                    let field_docs = DocGenerator::generate_field_docs(&field.docs);
                    
                    // 检查是否为 Pubkey 类型，如果是则添加特殊的 serde 属性
                    let pubkey_serde_attr = if Self::is_anchor_field_pubkey_type(&field.field_type) {
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

                Some(quote! {
                    #doc_comments
                    #[derive(Clone, Debug, borsh::BorshDeserialize, borsh::BorshSerialize, PartialEq, Eq)]
                    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                    pub struct #struct_name {
                        #(#struct_fields)*
                    }
                })
            } else {
                // 事件没有直接字段，使用IDL字段分配机制
                log::debug!("🎭 Events: Event '{}' 没有直接字段，尝试从字段分配获取", event.name);
                log::debug!("🎭 Events: Event '{}' 查询字段分配结果...", event.name);
                if let Some(allocated_fields) = self.idl.get_event_allocated_fields(&event.name) {
                    log::debug!("✅ Events: Event '{}' 从字段分配获取{}个字段: {:?}", 
                        event.name, allocated_fields.len(), 
                        allocated_fields.iter().map(|f| &f.name).collect::<Vec<_>>());
                    let doc_comments = DocGenerator::generate_doc_comments(&event.docs);
                    let struct_fields = allocated_fields.iter().map(|field_def| {
                        let (snake_field_name, serde_attr) = to_snake_case_with_serde(&field_def.name);
                        let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                        // 使用改进的类型转换逻辑
                        let field_type = Self::convert_field_definition_type_to_rust(&field_def.field_type);
                        let field_docs = if field_def.docs.is_empty() { 
                            quote! {} 
                        } else { 
                            DocGenerator::generate_doc_comments(&Some(field_def.docs.clone())) 
                        };
                        
                        // 检查字符串类型是否为 Pubkey
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

                    Some(quote! {
                        #doc_comments
                        #[derive(Clone, Debug, borsh::BorshDeserialize, borsh::BorshSerialize, PartialEq, Eq)]
                        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                        pub struct #struct_name {
                            #(#struct_fields)*
                        }
                    })
                } else {
                    log::debug!("❌ Events: Event '{}' 字段分配查询失败", event.name);
                    // 进一步检查字段分配映射的内容
                    let allocation = self.idl.get_field_allocation();
                    log::debug!("🔍 Events: 当前字段分配映射包含events: {:?}", 
                        allocation.events_fields.keys().collect::<Vec<_>>());
                    None
                }
            }
        });

        quote! {
            #(#event_structs)*
        }
    }
    
    /// 转换 FieldDefinition 的字符串类型为 Rust 类型（改进版）
    fn convert_field_definition_type_to_rust(type_str: &str) -> TokenStream {
        match type_str {
            // 基本类型
            "bool" => quote! { bool },
            "u8" => quote! { u8 },
            "u16" => quote! { u16 },
            "u32" => quote! { u32 },
            "u64" => quote! { u64 },
            "u128" => quote! { u128 },
            "i8" => quote! { i8 },
            "i16" => quote! { i16 },
            "i32" => quote! { i32 },
            "i64" => quote! { i64 },
            "i128" => quote! { i128 },
            "String" | "string" => quote! { String },
            "Pubkey" | "publicKey" | "pubkey" => quote! { Pubkey },
            
            // 数组类型：[type; size] 格式
            s if s.starts_with('[') && s.ends_with(']') => {
                // 解析 [u64; 16] 这样的格式
                let inner = &s[1..s.len()-1];
                if let Some(semicolon_pos) = inner.find(';') {
                    let element_type = inner[..semicolon_pos].trim();
                    let size_str = inner[semicolon_pos+1..].trim();
                    let element_type_token = Self::convert_field_definition_type_to_rust(element_type);
                    if let Ok(size) = size_str.parse::<usize>() {
                        let size_literal = proc_macro2::Literal::usize_unsuffixed(size);
                        quote! { [#element_type_token; #size_literal] }
                    } else {
                        // 解析失败，使用字符串本身
                        let type_ident = syn::Ident::new(type_str, proc_macro2::Span::call_site());
                        quote! { #type_ident }
                    }
                } else {
                    // 格式不正确，使用字符串本身
                    let type_ident = syn::Ident::new(type_str, proc_macro2::Span::call_site());
                    quote! { #type_ident }
                }
            },
            
            // Vec类型：Vec<type> 格式
            s if s.starts_with("Vec<") && s.ends_with('>') => {
                let inner_type = &s[4..s.len()-1];
                let inner_type_token = Self::convert_field_definition_type_to_rust(inner_type);
                quote! { Vec<#inner_type_token> }
            },
            
            // Option类型：Option<type> 格式  
            s if s.starts_with("Option<") && s.ends_with('>') => {
                let inner_type = &s[7..s.len()-1];
                let inner_type_token = Self::convert_field_definition_type_to_rust(inner_type);
                quote! { Option<#inner_type_token> }
            },
            
            // 其他自定义类型
            _ => {
                // 检查是否是基本类型（这些不应该在上面匹配的情况下到这里，但为了安全起见）
                let is_primitive = matches!(type_str, 
                    "bool" | "u8" | "u16" | "u32" | "u64" | "u128" | 
                    "i8" | "i16" | "i32" | "i64" | "i128" | 
                    "String" | "string" | "Pubkey" | "publicKey" | "pubkey"
                );
                
                if is_primitive {
                    let type_ident = syn::Ident::new(type_str, proc_macro2::Span::call_site());
                    quote! { #type_ident }
                } else {
                    // 自定义类型，使用完整路径
                    let type_path = format!("crate::types::{}", type_str);
                    match syn::parse_str::<syn::Path>(&type_path) {
                        Ok(path) => quote! { #path },
                        Err(_) => {
                            // 如果解析失败，尝试直接使用
                            let type_ident = syn::Ident::new(type_str, proc_macro2::Span::call_site());
                            quote! { #type_ident }
                        }
                    }
                }
            }
        }
    }

    /// 转换 IDL 类型为 Rust 类型
    fn convert_idl_type_to_rust(idl_type: &crate::idl_format::anchor_idl::AnchorFieldType) -> TokenStream {
        match idl_type {
            crate::idl_format::anchor_idl::AnchorFieldType::Basic(s) => {
                match s.as_str() {
                    "bool" => quote! { bool },
                    "u8" => quote! { u8 },
                    "u16" => quote! { u16 },
                    "u32" => quote! { u32 },
                    "u64" => quote! { u64 },
                    "u128" => quote! { u128 },
                    "i8" => quote! { i8 },
                    "i16" => quote! { i16 },
                    "i32" => quote! { i32 },
                    "i64" => quote! { i64 },
                    "i128" => quote! { i128 },
                    "publicKey" | "Pubkey" => quote! { Pubkey },
                    "string" | "String" => quote! { String },
                    _ => {
                        let type_name = syn::Ident::new(s, proc_macro2::Span::call_site());
                        quote! { #type_name }
                    }
                }
            },
            crate::idl_format::anchor_idl::AnchorFieldType::defined(type_name) => {
                // 使用完整路径引用types模块中的类型
                let type_path = format!("crate::types::{}", type_name);
                let type_path: syn::Path = syn::parse_str(&type_path).unwrap();
                quote! { #type_path }
            },
            crate::idl_format::anchor_idl::AnchorFieldType::array(inner_type, size) => {
                let inner_type = Self::convert_idl_type_to_rust(inner_type);
                let size_literal = proc_macro2::Literal::usize_unsuffixed(*size);
                quote! { [#inner_type; #size_literal] }
            },
            crate::idl_format::anchor_idl::AnchorFieldType::vec(inner_type) => {
                let inner_type = Self::convert_idl_type_to_rust(inner_type);
                quote! { Vec<#inner_type> }
            },
            crate::idl_format::anchor_idl::AnchorFieldType::option(inner_type) => {
                let inner_type = Self::convert_idl_type_to_rust(inner_type);
                quote! { Option<#inner_type> }
            },
            crate::idl_format::anchor_idl::AnchorFieldType::PrimitiveOrPubkey(type_str) => {
                match type_str.as_str() {
                    "bool" => quote! { bool },
                    "u8" => quote! { u8 },
                    "u16" => quote! { u16 },
                    "u32" => quote! { u32 },
                    "u64" => quote! { u64 },
                    "u128" => quote! { u128 },
                    "i8" => quote! { i8 },
                    "i16" => quote! { i16 },
                    "i32" => quote! { i32 },
                    "i64" => quote! { i64 },
                    "i128" => quote! { i128 },
                    "publicKey" | "Pubkey" => quote! { Pubkey },
                    "string" | "String" => quote! { String },
                    _ => {
                        let type_name = syn::Ident::new(type_str, proc_macro2::Span::call_site());
                        quote! { #type_name }
                    }
                }
            },
            crate::idl_format::anchor_idl::AnchorFieldType::Complex { kind, params: _ } => {
                let type_ident = syn::Ident::new(kind, proc_macro2::Span::call_site());
                quote! { #type_ident }
            },
        }
    }

    /// 生成事件包装器
    pub fn generate_event_wrappers(&self) -> TokenStream {
        let events = self.idl.events.as_deref().unwrap_or(&[]);
        if events.is_empty() {
            return quote! {};
        }

        let event_wrappers = events.iter().map(|event| {
            let struct_name = syn::Ident::new(&event.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let wrapper_name = syn::Ident::new(&format!("{}Event", event.name.to_case(Case::Pascal)), proc_macro2::Span::call_site());
            let discm_const_name = syn::Ident::new(
                &format!("{}_EVENT_DISCM", event.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );
            
            quote! {
                /// Event wrapper for #struct_name with discriminator for serialization
                #[derive(Clone, Debug, PartialEq)]
                pub struct #wrapper_name(pub #struct_name);
                
                impl borsh::BorshSerialize for #wrapper_name {
                    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                        #discm_const_name.serialize(writer)?;
                        self.0.serialize(writer)
                    }
                }
                
                impl #wrapper_name {
                    pub fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
                        let maybe_discm = <[u8; 8]>::deserialize(buf)?;
                        if maybe_discm != #discm_const_name {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!(
                                    "discm does not match. Expected: {:?}. Received: {:?}",
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

    /// 生成 discriminator 常量
    pub fn generate_discriminator_constants(&self) -> TokenStream {
        let events = self.idl.events.as_deref().unwrap_or(&[]);
        if events.is_empty() {
            return quote! {};
        }

        let constants = events.iter().map(|event| {
            let const_name = syn::Ident::new(
                &format!("{}_EVENT_DISCM", event.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );
            
            // Use the discriminator from IDL (8-byte array for Anchor)
            let discriminator = {
                let bytes = event.discriminator.iter().map(|&b| b).collect::<Vec<_>>();
                quote! { [#(#bytes),*] }
            };

            quote! {
                pub const #const_name: [u8; 8] = #discriminator;
            }
        });

        quote! {
            #(#constants)*
        }
    }
}

impl<'a> EventsTemplateGenerator for AnchorEventsTemplate<'a> {
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

impl<'a> AnchorEventsTemplate<'a> {
    /// 为单个event生成完整的文件内容
    pub fn generate_single_event_file(&self, event: &crate::idl_format::anchor_idl::AnchorEvent) -> TokenStream {
        let event_name = syn::Ident::new(&event.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
        // 修复命名重复：只用事件名称，不加Event后缀
        let const_name = syn::Ident::new(&format!("{}_EVENT_DISCM", event.name.to_shouty_snake_case()), proc_macro2::Span::call_site());
        
        let discriminator = {
            let bytes = event.discriminator.iter().map(|&b| b).collect::<Vec<_>>();
            quote! { [#(#bytes),*] }
        };

        let doc_comments = DocGenerator::generate_doc_comments(&event.docs);
        let event_name_str = &event.name;

        // 强制初始化字段分配缓存
        let _force_init = self.idl.get_field_allocation();

        // 生成事件结构体字段 - discriminator是第一个字段
        let event_fields = if let Some(fields) = &event.fields {
            // 路径1: 事件有直接字段定义
            let fields_tokens = fields.iter().map(|field| {
                let (snake_field_name, serde_attr) = to_snake_case_with_serde(&field.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                let field_type = self.convert_event_field_type(&field.field_type);
                let field_docs = DocGenerator::generate_field_docs(&field.docs);
                
                // 检查是否为 Pubkey 类型，如果是则添加特殊的 serde 属性
                let pubkey_serde_attr = if Self::is_anchor_field_pubkey_type(&field.field_type) {
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
            quote! { 
                pub discriminator: [u8; 8],
                #(#fields_tokens)* 
            }
        } else if let Some(allocated_fields) = self.idl.get_event_allocated_fields(&event.name) {
            // 路径2: 从字段分配获取字段
            let struct_fields = allocated_fields.iter().map(|field_def| {
                let (snake_field_name, serde_attr) = to_snake_case_with_serde(&field_def.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                // 使用改进的类型转换逻辑
                let field_type = Self::convert_field_definition_type_to_rust(&field_def.field_type);
                let field_docs = if field_def.docs.is_empty() { 
                    quote! {} 
                } else { 
                    DocGenerator::generate_doc_comments(&Some(field_def.docs.clone())) 
                };
                
                // 检查字符串类型是否为 Pubkey
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
            quote! {
                pub discriminator: [u8; 8],
                #(#struct_fields)*
            }
        } else {
            quote! {
                pub discriminator: [u8; 8],
            }
        };

        // 生成默认实现
        let default_fields = if let Some(fields) = &event.fields {
            // 路径1: 事件有直接字段定义
            let default_values = fields.iter().map(|field| {
                let (snake_field_name, _) = to_snake_case_with_serde(&field.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                quote! { #field_name: Default::default(), }
            });
            quote! { 
                discriminator: #const_name,
                #(#default_values)* 
            }
        } else if let Some(allocated_fields) = self.idl.get_event_allocated_fields(&event.name) {
            // 路径2: 从字段分配获取字段的默认值
            let default_values = allocated_fields.iter().map(|field_def| {
                let (snake_field_name, _) = to_snake_case_with_serde(&field_def.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                quote! { #field_name: Default::default(), }
            });
            quote! { 
                discriminator: #const_name,
                #(#default_values)* 
            }
        } else {
            quote! {
                discriminator: #const_name,
            }
        };

        let event_doc_comment = format!("Event: {}", event_name_str);
        quote! {
            #![doc = #event_doc_comment]
            #doc_comments
            
            #[allow(unused_imports)]
            use solana_pubkey::Pubkey;
            
            // Constants
            pub const #const_name: [u8; 8] = #discriminator;
            
            // Event Structure - 统一结构，discriminator是第一个字段
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

            impl #event_name {
                pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
                    borsh::to_vec(self)
                }
                
                pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
                    borsh::BorshDeserialize::deserialize(&mut &data[..])
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                }
            }
        }
    }

    /// 转换事件字段类型
    fn convert_event_field_type(&self, field_type: &crate::idl_format::anchor_idl::AnchorFieldType) -> TokenStream {
        // 复用已有的类型转换逻辑
        match field_type {
            crate::idl_format::anchor_idl::AnchorFieldType::Basic(primitive) => {
                let type_ident = match primitive.as_str() {
                    "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "u64" | "i64" | "u128" | "i128" => {
                        syn::Ident::new(primitive, proc_macro2::Span::call_site())
                    },
                    "bool" => syn::Ident::new("bool", proc_macro2::Span::call_site()),
                    "string" | "String" => syn::Ident::new("String", proc_macro2::Span::call_site()),
                    "publicKey" | "pubkey" | "Pubkey" => syn::Ident::new("Pubkey", proc_macro2::Span::call_site()),
                    _ => syn::Ident::new(primitive, proc_macro2::Span::call_site()),
                };
                quote! { #type_ident }
            },
            crate::idl_format::anchor_idl::AnchorFieldType::defined(type_name) => {
                let type_ident = syn::Ident::new(type_name, proc_macro2::Span::call_site());
                quote! { #type_ident }
            },
            crate::idl_format::anchor_idl::AnchorFieldType::array(inner_type, size) => {
                let inner_type_token = self.convert_event_field_type(inner_type);
                let size_literal = proc_macro2::Literal::usize_unsuffixed(*size);
                quote! { [#inner_type_token; #size_literal] }
            },
            crate::idl_format::anchor_idl::AnchorFieldType::vec(inner_type) => {
                let inner_type_token = self.convert_event_field_type(inner_type);
                quote! { Vec<#inner_type_token> }
            },
            crate::idl_format::anchor_idl::AnchorFieldType::option(inner_type) => {
                let inner_type_token = self.convert_event_field_type(inner_type);
                quote! { Option<#inner_type_token> }
            },
            crate::idl_format::anchor_idl::AnchorFieldType::PrimitiveOrPubkey(primitive) => {
                let type_ident = match primitive.as_str() {
                    "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "u64" | "i64" | "u128" | "i128" => {
                        syn::Ident::new(primitive, proc_macro2::Span::call_site())
                    },
                    "bool" => syn::Ident::new("bool", proc_macro2::Span::call_site()),
                    "string" | "String" => syn::Ident::new("String", proc_macro2::Span::call_site()),
                    "publicKey" | "pubkey" | "Pubkey" => syn::Ident::new("Pubkey", proc_macro2::Span::call_site()),
                    _ => syn::Ident::new(primitive, proc_macro2::Span::call_site()),
                };
                quote! { #type_ident }
            },
            crate::idl_format::anchor_idl::AnchorFieldType::Complex { kind, params: _ } => {
                let type_ident = syn::Ident::new(kind, proc_macro2::Span::call_site());
                quote! { #type_ident }
            },
        }
    }
}

impl<'a> AnchorEventsTemplate<'a> {
    /// 检查 Anchor 字段类型是否为 Pubkey
    fn is_anchor_field_pubkey_type(field_type: &crate::idl_format::anchor_idl::AnchorFieldType) -> bool {
        match field_type {
            crate::idl_format::anchor_idl::AnchorFieldType::Basic(s) => {
                matches!(s.as_str(), "publicKey" | "pubkey" | "Pubkey")
            },
            crate::idl_format::anchor_idl::AnchorFieldType::PrimitiveOrPubkey(s) => {
                matches!(s.as_str(), "publicKey" | "pubkey" | "Pubkey")
            },
            _ => false
        }
    }
    
    /// 检查字符串字段类型是否为 Pubkey
    fn is_string_field_pubkey_type(type_str: &str) -> bool {
        matches!(type_str, "publicKey" | "pubkey" | "Pubkey")
    }
}

impl<'a> TemplateGenerator for AnchorEventsTemplate<'a> {
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
        for event in events {
            let file_name = format!("{}.rs", event.name.to_case(Case::Snake));
            let file_content = self.generate_single_event_file(event);
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
            //! Anchor events module
            //! Generated event definitions with 8-byte discriminator support
            //! Each event is defined in its own file
            
            #(#module_declarations)*
            
            // Re-export all event items
            #(#re_exports)*
        }
    }
}