//! 非 Anchor Types 模板
//!
//! 为非 Anchor 合约生成 Types 相关代码，使用 1 字节 discriminator 或无 discriminator

use proc_macro2::TokenStream;
use quote::quote;
use convert_case::{Case, Casing};
use heck::ToShoutySnakeCase;

use crate::idl_format::non_anchor_idl::NonAnchorIdl;
use crate::Args;
use crate::templates::{TemplateGenerator, TypesTemplateGenerator};
use crate::templates::common::{doc_generator::DocGenerator, naming_converter::NamingConverter};
use crate::templates::field_analyzer::{FieldAllocationAnalyzer, FieldAllocationMap};
use crate::utils::{generate_pubkey_serde_attr, generate_pubkey_array_serde_attr, generate_large_array_serde_attr, generate_big_array_import, generate_pubkey_array_serde_helpers, generate_option_pubkey_serde_helpers, is_pubkey_type, parse_array_size};
use std::cell::RefCell;

/// 非 Anchor Types 模板
pub struct NonAnchorTypesTemplate<'a> {
    pub idl: &'a NonAnchorIdl,
    pub args: &'a Args,
    pub field_allocation: FieldAllocationMap,
    naming_converter: RefCell<NamingConverter>,
}

impl<'a> NonAnchorTypesTemplate<'a> {
    /// 创建新的非 Anchor Types 模板
    pub fn new(idl: &'a NonAnchorIdl, args: &'a Args) -> Self {
        // 分析字段分配，排除被其他模块使用的类型
        let field_allocation = FieldAllocationAnalyzer::analyze_non_anchor_idl(idl);
        Self { 
            idl, 
            args, 
            field_allocation,
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
}

impl<'a> TypesTemplateGenerator for NonAnchorTypesTemplate<'a> {
    fn generate_type_structs(&self) -> TokenStream {
        let types = self.idl.types.as_deref().unwrap_or(&[]);
        if types.is_empty() {
            return quote! {};
        }

        // 只为剩余的类型生成结构体（排除已被其他模块使用的）
        log::debug!("🏭 NonAnchorTypesTemplate: 开始生成类型结构体");
        log::debug!("   - 总类型数: {}", types.len());
        log::debug!("   - 剩余类型: {:?}", self.field_allocation.types_remaining_fields.keys().collect::<Vec<_>>());
        log::debug!("   - 被实现的类型: {:?}", self.field_allocation.implemented_types);
        
        let remaining_type_names = FieldAllocationAnalyzer::get_remaining_type_names(&self.field_allocation);
        let r#typeinitions = types.iter().enumerate().filter(|(_, r#type)| {
            let should_include = remaining_type_names.contains(&r#type.name);
            if should_include {
                log::debug!("   ✅ 生成类型: {}", r#type.name);
            } else {
                log::debug!("   🗑️ 跳过类型: {} (已被其他模块使用)", r#type.name);
            }
            should_include
        }).map(|(index, r#type)| {
            let type_name = syn::Ident::new(&r#type.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let doc_comments = DocGenerator::generate_non_anchor_type_docs(r#type);
            
            match &r#type.type_def {
                crate::idl_format::non_anchor_idl::NonAnchorTypeKind::Struct { fields } => {
                        let struct_fields = fields.iter().map(|field| {
                            let (snake_field_name, serde_attr) = self.convert_field_name_with_serde(&field.name);
                            let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                            let field_type = self.convert_idl_type_to_rust(&field.field_type);
                            let field_docs = DocGenerator::generate_field_docs(&field.docs);
                            
                            // 检查字段类型并生成相应的 serde 属性
                            let serde_attr_for_type = Self::generate_field_serde_attr(&field.field_type);
                            
                            quote! {
                                #field_docs
                                #serde_attr
                                #serde_attr_for_type
                                pub #field_name: #field_type,
                            }
                        });

                        let default_field_values = fields.iter().map(|field| {
                            let (snake_field_name, _) = self.convert_field_name_with_serde(&field.name);
                            let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                            let default_value = self.generate_field_default_from_type(&field.field_type);
                            quote! { #field_name: #default_value }
                        });

                        // Types模块不需要discriminator字段 - 只保留纯数据字段

                        quote! {
                            #doc_comments
                            #[derive(Clone, Debug, borsh::BorshDeserialize, borsh::BorshSerialize, PartialEq, Eq)]
                            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                            pub struct #type_name {
                                #(#struct_fields)*
                            }
                            
                            impl Default for #type_name {
                                fn default() -> Self {
                                    Self {
                                        #(#default_field_values,)*
                                    }
                                }
                            }
                        }
                    },
                crate::idl_format::non_anchor_idl::NonAnchorTypeKind::Enum { variants } => {
                        let enum_variants = variants.iter().enumerate().map(|(variant_index, variant)| {
                            let variant_name = syn::Ident::new(&variant.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
                            let variant_docs = quote! {}; // EnumVariant doesn't have docs field
                            
                            if let Some(fields) = &variant.fields {
                                // Variant with fields - direct处理AnchorField列表
                                let variant_fields = fields.iter().map(|field| {
                                    let field_type = self.convert_idl_type_to_rust(&field.field_type);
                                    quote! { #field_type }
                                });
                                
                                quote! {
                                    #variant_docs
                                    #variant_name(#(#variant_fields),*),
                                }
                            } else {
                                // Simple variant - no fields, no explicit discriminator needed for repr(u8)
                                quote! {
                                    #variant_docs
                                    #variant_name,
                                }
                            }
                        });

                        // Generate Default implementation for enum (first variant)
                        let default_impl = if let Some(first_variant) = variants.first() {
                            let variant_name = syn::Ident::new(&first_variant.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
                            
                            let default_value = if first_variant.fields.is_some() {
                                quote! { Self::#variant_name(Default::default()) }
                            } else {
                                quote! { Self::#variant_name }
                            };
                            
                            quote! {
                                impl Default for #type_name {
                                    fn default() -> Self {
                                        #default_value
                                    }
                                }
                            }
                        } else {
                            quote! {}
                        };

                        quote! {
                            #doc_comments
                            #[derive(Clone, Debug, borsh::BorshDeserialize, borsh::BorshSerialize, PartialEq, Eq)]
                            #[borsh(use_discriminant = false)]
                            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                            #[repr(u8)]
                            pub enum #type_name {
                                #(#enum_variants)*
                            }
                            #default_impl
                        }
                    },
                crate::idl_format::non_anchor_idl::NonAnchorTypeKind::Alias { value } => {
                        // For type aliases, just generate a type alias declaration
                        let alias_type = self.convert_idl_type_to_rust(value);
                        
                        quote! {
                            #doc_comments
                            pub type #type_name = #alias_type;
                        }
                    },
                }
        });

        // 检查是否需要生成 serde 辅助函数
        let needs_option_pubkey_helpers = Self::has_option_pubkey_fields(types);
        let needs_pubkey_array_helpers = Self::has_pubkey_arrays(types);
        let needs_big_array_import = Self::has_large_arrays(types);
        
        let option_pubkey_helpers = if needs_option_pubkey_helpers {
            generate_option_pubkey_serde_helpers()
        } else {
            quote! {}
        };
        
        let pubkey_array_helpers = if needs_pubkey_array_helpers {
            generate_pubkey_array_serde_helpers()
        } else {
            quote! {}
        };
        
        let big_array_import = if needs_big_array_import {
            generate_big_array_import()
        } else {
            quote! {}
        };
        
        quote! {
            #big_array_import
            #option_pubkey_helpers
            #pubkey_array_helpers
            #(#r#typeinitions)*
        }
    }

    fn generate_type_constants(&self) -> TokenStream {
        let types = self.idl.types.as_deref().unwrap_or(&[]);
        if types.is_empty() {
            return quote! {};
        }

        // Generate 1-byte discriminator constants for non-Anchor contracts
        let constants = types.iter().enumerate().map(|(index, r#type)| {
            let const_name = syn::Ident::new(
                &format!("{}_TYPE_DISCM", r#type.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );
            
            // Use simple index-based discriminator for non-Anchor contracts
            let discriminator_value = index as u8;
            
            quote! {
                /// 1-byte discriminator constant for non-Anchor type identification
                pub const #const_name: u8 = #discriminator_value;
            }
        });

        quote! {
            #(#constants)*
        }
    }
}

impl<'a> NonAnchorTypesTemplate<'a> {
    /// 从字段类型生成默认值
    fn generate_field_default_from_type(&self, field_type: &crate::idl_format::non_anchor_idl::NonAnchorFieldType) -> TokenStream {
        match field_type {
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Basic(type_str) => {
                match type_str.as_str() {
                    "bool" => quote! { false },
                    "String" | "string" => quote! { String::new() },
                    _ => quote! { Default::default() },
                }
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Option { .. } => {
                quote! { None }
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Vec { .. } => {
                quote! { Vec::new() }
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Array { array } => {
                let (inner_type, size) = array;
                if *size > 32 {
                    // For large arrays, generate manual initialization
                    match inner_type.as_ref() {
                        crate::idl_format::non_anchor_idl::NonAnchorFieldType::Basic(type_str) if type_str == "u8" => {
                            let size_literal = proc_macro2::Literal::usize_unsuffixed(*size);
                            quote! { [0u8; #size_literal] }
                        },
                        crate::idl_format::non_anchor_idl::NonAnchorFieldType::Basic(type_str) if type_str == "u64" => {
                            let size_literal = proc_macro2::Literal::usize_unsuffixed(*size);
                            quote! { [0u64; #size_literal] }
                        },
                        crate::idl_format::non_anchor_idl::NonAnchorFieldType::Defined { .. } => {
                            // For arrays of custom types, use array initialization with Default
                            let size_literal = proc_macro2::Literal::usize_unsuffixed(*size);
                            quote! { [(); #size_literal].map(|_| Default::default()) }
                        },
                        _ => quote! { Default::default() }
                    }
                } else {
                    quote! { Default::default() }
                }
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Defined { .. } => {
                quote! { Default::default() }
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::HashMap { .. } => {
                quote! { std::collections::HashMap::new() }
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Complex { kind, params: _ } => {
                // Legacy支持
                match kind.as_str() {
                    "Vec" => quote! { Vec::new() },
                    "Option" => quote! { None },
                    _ => quote! { Default::default() },
                }
            },
        }
    }

    /// 解析泛型类型语法，如 SmallVec<u8,Pubkey>
    fn parse_generic_type(&self, type_str: &str) -> TokenStream {
        // 简单的泛型解析：类型名<参数1,参数2,...>
        if let Some(generic_open) = type_str.find('<') {
            if let Some(generic_close) = type_str.rfind('>') {
                let base_type = &type_str[..generic_open];
                let params_str = &type_str[generic_open + 1..generic_close];
                
                // 清理基础类型名，处理特殊情况
                let clean_base_type = match base_type {
                    "SmallVec" => "Vec", // SmallVec 映射为 Vec
                    _ => base_type,
                };
                
                // 解析参数
                let params: Vec<&str> = params_str.split(',').map(|p| p.trim()).collect();
                
                match clean_base_type {
                    "Vec" => {
                        // Vec<type> - 忽略第一个参数（SmallVec的capacity），使用第二个参数作为元素类型
                        if params.len() >= 2 {
                            let element_type = self.map_basic_type(params[1]);
                            let element_ident = syn::Ident::new(&element_type, proc_macro2::Span::call_site());
                            quote! { Vec<#element_ident> }
                        } else if params.len() == 1 {
                            let element_type = self.map_basic_type(params[0]);
                            let element_ident = syn::Ident::new(&element_type, proc_macro2::Span::call_site());
                            quote! { Vec<#element_ident> }
                        } else {
                            quote! { Vec<u8> } // fallback
                        }
                    },
                    _ => {
                        // 未知泛型类型，尝试生成合理的代替
                        log::warn!("未知泛型类型: {}, 回退为 Vec<u8>", type_str);
                        quote! { Vec<u8> }
                    }
                }
            } else {
                log::warn!("泛型类型语法错误: {}, 回退为 Vec<u8>", type_str);
                quote! { Vec<u8> }
            }
        } else {
            log::warn!("不是泛型类型: {}, 回退为 Vec<u8>", type_str);
            quote! { Vec<u8> }
        }
    }
    
    /// 映射基础类型名称
    fn map_basic_type(&self, type_str: &str) -> String {
        match type_str {
            "publicKey" | "pubkey" => "Pubkey".to_string(),
            "string" => "String".to_string(),
            "bytes" => "Vec<u8>".to_string(),
            _ => type_str.to_string(),
        }
    }

    /// 转换 IDL 类型为 Rust 类型 (非Anchor版本)
    fn convert_idl_type_to_rust(&self, idl_type: &crate::idl_format::non_anchor_idl::NonAnchorFieldType) -> TokenStream {
        match idl_type {
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Basic(type_str) => {
                let type_ident = match type_str.as_str() {
                    "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "u64" | "i64" | "u128" | "i128" => {
                        syn::Ident::new(type_str, proc_macro2::Span::call_site())
                    },
                    "bool" => syn::Ident::new("bool", proc_macro2::Span::call_site()),
                    "string" | "String" => syn::Ident::new("String", proc_macro2::Span::call_site()),
                    "publicKey" | "pubkey" | "Pubkey" => syn::Ident::new("Pubkey", proc_macro2::Span::call_site()),
                    "bytes" => return quote! { Vec<u8> }, // bytes类型映射为Vec<u8>
                    _ => syn::Ident::new(type_str, proc_macro2::Span::call_site()),
                };
                quote! { #type_ident }
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Option { option } => {
                let inner_type = self.convert_idl_type_to_rust(option);
                quote! { Option<#inner_type> }
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Vec { vec } => {
                let inner_type = self.convert_idl_type_to_rust(vec);
                quote! { Vec<#inner_type> }
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Array { array } => {
                let (inner_type, size) = array;
                let inner_type_token = self.convert_idl_type_to_rust(inner_type);
                let size_literal = proc_macro2::Literal::usize_unsuffixed(*size);
                quote! { [#inner_type_token; #size_literal] }
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Defined { defined } => {
                // 处理泛型类型语法，如 SmallVec<u8,Pubkey>
                if defined.contains('<') && defined.contains('>') {
                    // 这是一个泛型类型，需要解析
                    self.parse_generic_type(defined)
                } else {
                    // 普通的defined类型，使用完整路径
                    let type_path = format!("crate::types::{}", defined);
                    let type_path: syn::Path = syn::parse_str(&type_path).unwrap();
                    quote! { #type_path }
                }
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::HashMap { key, value } => {
                let key_type = self.convert_idl_type_to_rust(key);
                let value_type = self.convert_idl_type_to_rust(value);
                quote! { std::collections::HashMap<#key_type, #value_type> }
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Complex { kind, params } => {
                // 处理复合类型，如 Vec<T>, Option<T>, [T; N] 等 (Legacy支持)
                match kind.as_str() {
                    "Vec" => {
                        if let Some(params) = params {
                            if let Some(inner_type) = params.get(0) {
                                if let Some(inner_str) = inner_type.as_str() {
                                    let inner_type_token = self.convert_idl_type_to_rust(&crate::idl_format::non_anchor_idl::NonAnchorFieldType::Basic(inner_str.to_string()));
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
                                    let inner_type_token = self.convert_idl_type_to_rust(&crate::idl_format::non_anchor_idl::NonAnchorFieldType::Basic(inner_str.to_string()));
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

    /// 检查 NonAnchor 字段类型是否为 Pubkey
    fn is_non_anchor_field_pubkey_type(field_type: &crate::idl_format::non_anchor_idl::NonAnchorFieldType) -> bool {
        match field_type {
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Basic(s) => {
                matches!(s.as_str(), "publicKey" | "pubkey" | "Pubkey")
            },
            _ => false
        }
    }
    
    /// 为字段生成相应的 serde 属性
    fn generate_field_serde_attr(field_type: &crate::idl_format::non_anchor_idl::NonAnchorFieldType) -> TokenStream {
        match field_type {
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Basic(s) if is_pubkey_type(s) => {
                generate_pubkey_serde_attr()
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Option { option } => {
                match option.as_ref() {
                    crate::idl_format::non_anchor_idl::NonAnchorFieldType::Basic(s) if is_pubkey_type(s) => {
                        quote! {
                            #[cfg_attr(
                                feature = "serde",
                                serde(
                                    serialize_with = "serialize_option_pubkey_as_string",
                                    deserialize_with = "deserialize_option_pubkey_from_string"
                                )
                            )]
                        }
                    },
                    _ => quote! {}
                }
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Array { array } => {
                let (inner_type, size) = array;
                let is_pubkey = match inner_type.as_ref() {
                    crate::idl_format::non_anchor_idl::NonAnchorFieldType::Basic(s) => is_pubkey_type(s),
                    _ => false
                };
                
                if let Some(attr) = generate_pubkey_array_serde_attr(*size, is_pubkey) {
                    attr
                } else if let Some(attr) = generate_large_array_serde_attr(*size) {
                    attr
                } else {
                    quote! {}
                }
            },
            _ => quote! {}
        }
    }
    
    /// 检查类型中是否有 Option<Pubkey> 字段
    fn has_option_pubkey_fields(types: &[crate::idl_format::non_anchor_idl::NonAnchorType]) -> bool {
        types.iter().any(|t| {
            match &t.type_def {
                crate::idl_format::non_anchor_idl::NonAnchorTypeKind::Struct { fields } => {
                    fields.iter().any(|field| Self::field_needs_option_pubkey_helpers(&field.field_type))
                },
                _ => false
            }
        })
    }
    
    /// 检查类型中是否有 Pubkey 数组
    fn has_pubkey_arrays(types: &[crate::idl_format::non_anchor_idl::NonAnchorType]) -> bool {
        types.iter().any(|t| {
            match &t.type_def {
                crate::idl_format::non_anchor_idl::NonAnchorTypeKind::Struct { fields } => {
                    fields.iter().any(|field| Self::field_needs_pubkey_array_helpers(&field.field_type))
                },
                _ => false
            }
        })
    }
    
    /// 检查类型中是否有大数组 (>32)
    fn has_large_arrays(types: &[crate::idl_format::non_anchor_idl::NonAnchorType]) -> bool {
        types.iter().any(|t| {
            match &t.type_def {
                crate::idl_format::non_anchor_idl::NonAnchorTypeKind::Struct { fields } => {
                    fields.iter().any(|field| Self::field_needs_big_array(&field.field_type))
                },
                _ => false
            }
        })
    }
    
    /// 检查字段是否需要 Option<Pubkey> 辅助函数
    fn field_needs_option_pubkey_helpers(field_type: &crate::idl_format::non_anchor_idl::NonAnchorFieldType) -> bool {
        match field_type {
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Option { option } => {
                match option.as_ref() {
                    crate::idl_format::non_anchor_idl::NonAnchorFieldType::Basic(s) => is_pubkey_type(s),
                    _ => false
                }
            },
            _ => false
        }
    }
    
    /// 检查字段是否需要 Pubkey 数组辅助函数
    fn field_needs_pubkey_array_helpers(field_type: &crate::idl_format::non_anchor_idl::NonAnchorFieldType) -> bool {
        match field_type {
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Array { array } => {
                let (inner_type, _size) = array;
                match inner_type.as_ref() {
                    crate::idl_format::non_anchor_idl::NonAnchorFieldType::Basic(s) => is_pubkey_type(s),
                    _ => false
                }
            },
            _ => false
        }
    }
    
    /// 检查字段是否需要大数组支持
    fn field_needs_big_array(field_type: &crate::idl_format::non_anchor_idl::NonAnchorFieldType) -> bool {
        match field_type {
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Array { array } => {
                let (_inner_type, size) = array;
                *size > 32
            },
            _ => false
        }
    }

    /// 为单个type生成完整的文件内容（NonAnchor版本）
    pub fn generate_single_type_file(&self, type_def: &crate::idl_format::non_anchor_idl::NonAnchorType, index: usize) -> TokenStream {
        let type_name = syn::Ident::new(&type_def.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
        let const_name = syn::Ident::new(&format!("{}_TYPE_DISCM", type_def.name.to_shouty_snake_case()), proc_macro2::Span::call_site());
        let doc_comments = DocGenerator::generate_doc_comments(&type_def.docs);
        let type_name_str = &type_def.name;
        let discriminator_value = index as u8;
        
        let type_definition = match &type_def.type_def {
            crate::idl_format::non_anchor_idl::NonAnchorTypeKind::Struct { fields } => {
                let field_tokens = fields.iter().map(|field| {
                        let (snake_field_name, serde_attr) = self.convert_field_name_with_serde(&field.name);
                        let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                        let field_type = self.convert_idl_type_to_rust(&field.field_type);
                        let field_docs = DocGenerator::generate_field_docs(&field.docs);
                        
                        // 检查字段类型并生成相应的 serde 属性
                        let serde_attr_for_type = Self::generate_field_serde_attr(&field.field_type);
                        
                        quote! {
                            #field_docs
                            #serde_attr
                            #serde_attr_for_type
                            pub #field_name: #field_type,
                        }
                    });

                    let default_fields = fields.iter().map(|field| {
                        let (snake_field_name, _) = self.convert_field_name_with_serde(&field.name);
                        let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                        let default_value = self.generate_field_default_from_type(&field.field_type);
                        quote! { #field_name: #default_value, }
                    });

                    quote! {
                        #doc_comments
                        #[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Clone, Debug, PartialEq)]
                        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                        pub struct #type_name {
                            #(#field_tokens)*
                        }
                        
                        impl Default for #type_name {
                            fn default() -> Self {
                                Self {
                                    #(#default_fields)*
                                }
                            }
                        }
                    }
            },
            
            crate::idl_format::non_anchor_idl::NonAnchorTypeKind::Enum { variants } => {
                let variant_tokens = variants.iter().enumerate().map(|(variant_index, variant)| {
                        let variant_name = syn::Ident::new(&variant.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
                        let variant_docs = quote! {}; // EnumVariant doesn't have docs field
                        
                        match &variant.fields {
                            Some(fields) if !fields.is_empty() => {
                                let variant_fields = fields.iter().map(|field| {
                                    let field_type = self.convert_idl_type_to_rust(&field.field_type);
                                    quote! { #field_type }
                                });
                                quote! {
                                    #variant_docs
                                    #variant_name(#(#variant_fields),*),
                                }
                            },
                            _ => {
                                quote! {
                                    #variant_docs
                                    #variant_name,
                                }
                            }
                        }
                    });

                    let default_variant = variants.first().map(|first_variant| {
                        let variant_name = syn::Ident::new(&first_variant.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
                        let default_value = if first_variant.fields.as_ref().map_or(false, |f| !f.is_empty()) {
                            quote! { Self::#variant_name(Default::default()) }
                        } else {
                            quote! { Self::#variant_name }
                        };
                        quote! {
                            impl Default for #type_name {
                                fn default() -> Self {
                                    #default_value
                                }
                            }
                        }
                    }).unwrap_or_else(|| quote! {});

                    quote! {
                        #doc_comments
                        #[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Clone, Debug, PartialEq)]
                        #[borsh(use_discriminant = false)]
                        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                        #[repr(u8)]
                        pub enum #type_name {
                            #(#variant_tokens)*
                        }
                        #default_variant
                    }
            },
            
            crate::idl_format::non_anchor_idl::NonAnchorTypeKind::Alias { value } => {
                let rust_type = self.convert_idl_type_to_rust(value);
                    
                    quote! {
                        #doc_comments
                        pub type #type_name = #rust_type;
                    }
            }
        };

        // 检查单个类型是否需要生成 serde 辅助函数
        let single_type_slice = std::slice::from_ref(type_def);
        let needs_option_pubkey_helpers = Self::has_option_pubkey_fields(single_type_slice);
        let needs_pubkey_array_helpers = Self::has_pubkey_arrays(single_type_slice);
        let needs_big_array_import = Self::has_large_arrays(single_type_slice);
        
        let option_pubkey_helpers = if needs_option_pubkey_helpers {
            generate_option_pubkey_serde_helpers()
        } else {
            quote! {}
        };
        
        let pubkey_array_helpers = if needs_pubkey_array_helpers {
            generate_pubkey_array_serde_helpers()
        } else {
            quote! {}
        };
        
        let big_array_import = if needs_big_array_import {
            generate_big_array_import()
        } else {
            quote! {}
        };
        
        let type_doc_comment = format!("Type: {} (NonAnchor)", type_name_str);
        quote! {
            #![doc = #type_doc_comment]
            #doc_comments
            
            #[allow(unused_imports)]
            use solana_pubkey::Pubkey;
            
            #big_array_import
            #option_pubkey_helpers
            #pubkey_array_helpers
            
            // Constants
            pub const #const_name: u8 = #discriminator_value;
            
            #type_definition

            impl #type_name {
                pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
                    borsh::to_vec(self)
                }
                
                pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
                    borsh::from_slice(data).map_err(|e| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
                    })
                }
            }
        }
    }
}

impl<'a> TemplateGenerator for NonAnchorTypesTemplate<'a> {
    fn get_standard_module_name(&self) -> &'static str {
        "types"
    }

    fn gen_files(&self) -> Vec<(String, TokenStream)> {
        let types = self.idl.types.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if types.is_empty() {
            return vec![("mod.rs".to_string(), quote! {
                //! Empty types module - no types found in IDL
            })];
        }
        
        // 智能过滤：只为未被实现的类型生成文件（排除被实现的类型，保留被引用的类型）
        let mut types_to_generate: Vec<(usize, &crate::idl_format::non_anchor_idl::NonAnchorType)> = Vec::new();
        
        for (index, type_def) in types.iter().enumerate() {
            // 检查是否为被实现的类型（需要从types模块中移除）
            if self.field_allocation.implemented_types.contains(&type_def.name) {
                log::debug!("🗑️ 跳过被实现的类型: {}", type_def.name);
                continue;
            }
            
            // 包含所有未被实现的类型（包括被引用的类型和独立类型）
            if self.field_allocation.referenced_types.contains(&type_def.name) {
                log::debug!("🔗 生成被引用的类型: {}", type_def.name);
            } else {
                log::debug!("📄 生成独立的类型: {}", type_def.name);
            }
            types_to_generate.push((index, type_def));
        }
        
        if types_to_generate.is_empty() {
            return vec![("mod.rs".to_string(), quote! {
                //! Empty types module - all types implemented in other modules
            })];
        }
        
        let mut files = Vec::new();
        
        // 为过滤后的类型生成独立文件
        for (index, type_def) in types_to_generate.iter() {
            let file_name = format!("{}.rs", type_def.name.to_case(Case::Snake));
            let file_content = self.generate_single_type_file(type_def, *index);
            files.push((file_name, file_content));
        }
        
        files
    }

    fn gen_mod_file(&self) -> TokenStream {
        let types = self.idl.types.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if types.is_empty() {
            return quote! {
                //! Types module - no types found in IDL
            };
        }
        
        // 智能过滤：只为未被实现的类型生成模块声明（包括被引用的和独立的类型）
        let types_to_declare: Vec<&crate::idl_format::non_anchor_idl::NonAnchorType> = types.iter()
            .filter(|type_def| !self.field_allocation.implemented_types.contains(&type_def.name))
            .collect();
        
        if types_to_declare.is_empty() {
            return quote! {
                //! Empty types module - all types implemented in other modules
            };
        }
        
        // 生成模块声明和重新导出 - 只为未被实现的类型
        let module_declarations = types_to_declare.iter().map(|type_def| {
            let module_name = syn::Ident::new(&type_def.name.to_case(Case::Snake), proc_macro2::Span::call_site());
            quote! {
                pub mod #module_name;
            }
        });
        
        let re_exports = types_to_declare.iter().map(|type_def| {
            let module_name = syn::Ident::new(&type_def.name.to_case(Case::Snake), proc_macro2::Span::call_site());
            quote! {
                pub use #module_name::*;
            }
        });
        
        quote! {
            //! Non-Anchor types module
            //! Generated type definitions for non-Anchor contracts
            //! Each type is defined in its own file
            
            #(#module_declarations)*
            
            // Re-export all type items
            #(#re_exports)*
        }
    }
}