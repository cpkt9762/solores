//! 非 Anchor Accounts 模板
//!
//! 为非 Anchor 合约生成 Accounts 相关代码，基于长度识别

use proc_macro2::TokenStream;
use quote::quote;
use convert_case::{Case, Casing};
use heck::ToShoutySnakeCase;

use crate::idl_format::non_anchor_idl::{NonAnchorIdl, NonAnchorFieldType, NonAnchorAccount};
use crate::Args;
use crate::templates::TemplateGenerator;
use crate::templates::common::{doc_generator::DocGenerator, naming_converter::NamingConverter};
use crate::utils::{generate_pubkey_serde_attr};
use std::cell::RefCell;

/// 非 Anchor Accounts 模板
pub struct NonAnchorAccountsTemplate<'a> {
    pub idl: &'a NonAnchorIdl,
    pub args: &'a Args,
    naming_converter: RefCell<NamingConverter>,
}

impl<'a> NonAnchorAccountsTemplate<'a> {
    /// 创建新的非 Anchor Accounts 模板
    pub fn new(idl: &'a NonAnchorIdl, args: &'a Args) -> Self {
        Self { 
            idl, 
            args,
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

    /// 检测是否为 SPL Token 程序
    fn is_spl_token_program(&self) -> bool {
        matches!(self.idl.address.as_str(), 
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" | // SPL Token
            "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"   // SPL Token-2022
        )
    }

    /// 检查字段是否为 Option<Pubkey> 并且是 SPL Token 程序（需要转换为 COption<Pubkey>）
    fn is_spl_token_coption_field(&self, field_type: &NonAnchorFieldType) -> bool {
        if !self.is_spl_token_program() {
            return false;
        }
        
        match field_type {
            NonAnchorFieldType::Option { option } => {
                matches!(option.as_ref(), NonAnchorFieldType::Basic(s) if matches!(s.as_str(), "publicKey" | "pubkey" | "Pubkey"))
            },
            _ => false
        }
    }

    /// 生成 SPL Token 的自定义 COption<Pubkey> 序列化注解
    fn generate_spl_token_coption_attrs(&self) -> TokenStream {
        if self.is_spl_token_program() {
            quote! {
                #[borsh(serialize_with = "serialize_coption_pubkey")]
                #[borsh(deserialize_with = "deserialize_coption_pubkey")]
            }
        } else {
            quote! {}
        }
    }

    /// 生成 SPL Token 的 COption<Pubkey> 自定义序列化函数
    fn generate_spl_token_coption_serializers(&self) -> TokenStream {
        if self.is_spl_token_program() {
            quote! {
                // SPL Token COption<Pubkey> 自定义序列化，兼容 Pack trait
                fn serialize_coption_pubkey<W: std::io::Write>(
                    coption: &solana_program::program_option::COption<Pubkey>,
                    writer: &mut W
                ) -> std::io::Result<()> {
                    match coption {
                        solana_program::program_option::COption::None => {
                            // 写入 4 字节 0 (Pack trait 兼容)
                            borsh::BorshSerialize::serialize(&0u32, writer)?;
                            // 填充 32 字节全零
                            borsh::BorshSerialize::serialize(&[0u8; 32], writer)
                        }
                        solana_program::program_option::COption::Some(pubkey) => {
                            // 写入 4 字节 1
                            borsh::BorshSerialize::serialize(&1u32, writer)?;
                            // 写入 32 字节公钥
                            borsh::BorshSerialize::serialize(pubkey, writer)
                        }
                    }
                }

                fn deserialize_coption_pubkey(buf: &mut &[u8]) -> std::io::Result<solana_program::program_option::COption<Pubkey>> {
                    let discriminant = u32::deserialize(buf)?;
                    match discriminant {
                        0 => {
                            // 跳过 32 字节占位符
                            let _ = <[u8; 32]>::deserialize(buf)?;
                            Ok(solana_program::program_option::COption::None)
                        }
                        1 => {
                            let pubkey = Pubkey::deserialize(buf)?;
                            Ok(solana_program::program_option::COption::Some(pubkey))
                        }
                        _ => Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Invalid COption discriminant: {}", discriminant)
                        ))
                    }
                }

                use borsh::BorshDeserialize;
            }
        } else {
            quote! {}
        }
    }

    /// 检查 typedef 字段类型是否为 Pubkey
    fn is_typedef_field_pubkey_type(field_type: &NonAnchorFieldType) -> bool {
        match field_type {
            NonAnchorFieldType::Basic(s) => {
                matches!(s.as_str(), "publicKey" | "pubkey" | "Pubkey")
            },
            NonAnchorFieldType::Array { array } => {
                // 递归检查数组元素类型
                Self::is_typedef_field_pubkey_type(&array.0)
            },
            NonAnchorFieldType::Option { option } => {
                // 递归检查Option内部类型
                Self::is_typedef_field_pubkey_type(option)
            },
            _ => false
        }
    }
    
    /// 检查字符串字段类型是否为 Pubkey
    fn is_string_field_pubkey_type(type_str: &str) -> bool {
        matches!(type_str, "publicKey" | "pubkey" | "Pubkey")
    }


    /// 生成账户结构体
    pub fn generate_account_structs(&self) -> TokenStream {
        let accounts = self.idl.accounts.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if accounts.is_empty() {
            return quote! {};
        }

        // 强制初始化字段分配缓存
        let _force_init = self.idl.get_field_allocation();
        
        let structs = accounts.iter().filter_map(|account| {
            let struct_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            
            // NonAnchor字段生成优先级：direct fields → field allocation → empty structures
            let doc_comments = DocGenerator::generate_non_anchor_account_docs(account);
            let fields = if let Some(account_fields) = &account.fields {
                // 优先级1：直接使用account.fields
                let struct_fields = account_fields.iter().map(|field| {
                    let (snake_field_name, serde_attr) = self.convert_field_name_with_serde(&field.name);
                    let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                    let field_type = self.convert_typedef_field_type_to_rust_spl(&field.field_type);
                    let field_docs = DocGenerator::generate_field_docs(&field.docs);
                    
                    // 检查是否为 SPL Token COption<Pubkey> 字段
                    let coption_attrs = if self.is_spl_token_coption_field(&field.field_type) {
                        self.generate_spl_token_coption_attrs()
                    } else {
                        quote! {}
                    };
                    
                    // 检查是否为 Pubkey 类型，如果是则添加特殊的 serde 属性
                    let pubkey_serde_attr = if Self::is_typedef_field_pubkey_type(&field.field_type) {
                        generate_pubkey_serde_attr()
                    } else {
                        quote! {}
                    };
                    
                    quote! {
                        #field_docs
                        #serde_attr
                        #pubkey_serde_attr
                        #coption_attrs
                        pub #field_name: #field_type,
                    }
                });

                Some((doc_comments, quote! {
                    #(#struct_fields)*
                }))
            } else if let Some(allocated_fields) = self.idl.get_account_allocated_fields(&account.name) {
                // 优先级2：从字段分配获取字段
                let struct_fields = allocated_fields.iter().map(|field_def| {
                    let (snake_field_name, serde_attr) = self.convert_field_name_with_serde(&field_def.name);
                    let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                    let field_type = Self::convert_field_definition_type_to_rust(&field_def.field_type);
                    let field_docs = DocGenerator::generate_doc_comments(&Some(field_def.docs.clone()));
                    
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

                Some((doc_comments, quote! {
                    #(#struct_fields)*
                }))
            } else {
                // 优先级3：无任何字段定义，创建空结构体
                Some((doc_comments, quote! {}))
            };

            if let Some((doc_comments, fields)) = fields {
                Some(quote! {
                    #doc_comments
                    #[derive(Clone, Debug, borsh::BorshDeserialize, borsh::BorshSerialize, PartialEq, Eq)]
                    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                    pub struct #struct_name {
                        #fields
                    }
                })
            } else {
                None
            }
        });

        let serializers = self.generate_spl_token_coption_serializers();
        
        quote! {
            #serializers
            #(#structs)*
        }
    }

    /// 从 FieldDefinition 的字符串类型转换为 Rust 类型（用于字段分配机制）
    fn convert_field_definition_type_to_rust(type_str: &str) -> TokenStream {
        match type_str {
            "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "u64" | "i64" | "u128" | "i128" => {
                let type_ident = syn::Ident::new(type_str, proc_macro2::Span::call_site());
                quote! { #type_ident }
            },
            "bool" => quote! { bool },
            "String" | "string" => quote! { String },
            "Pubkey" | "publicKey" | "pubkey" => quote! { Pubkey },
            _ if type_str.starts_with("Vec<") && type_str.ends_with(">") => {
                let inner_type = &type_str[4..type_str.len()-1];
                let inner_token = Self::convert_field_definition_type_to_rust(inner_type);
                quote! { Vec<#inner_token> }
            },
            _ if type_str.starts_with("Option<") && type_str.ends_with(">") => {
                let inner_type = &type_str[7..type_str.len()-1];
                let inner_token = Self::convert_field_definition_type_to_rust(inner_type);
                quote! { Option<#inner_token> }
            },
            _ if type_str.starts_with("[") && type_str.contains(";") && type_str.ends_with("]") => {
                // 处理数组类型，如 [u8; 32]
                if let Some(semicolon_pos) = type_str.find(';') {
                    let inner_type = &type_str[1..semicolon_pos].trim();
                    let size_str = &type_str[semicolon_pos+1..type_str.len()-1].trim();
                    let inner_token = Self::convert_field_definition_type_to_rust(inner_type);
                    if let Ok(size) = size_str.parse::<usize>() {
                        let size_literal = proc_macro2::Literal::usize_unsuffixed(size);
                        quote! { [#inner_token; #size_literal] }
                    } else {
                        // 无法解析大小，回退到自定义类型
                        let type_ident = syn::Ident::new(type_str, proc_macro2::Span::call_site());
                        quote! { #type_ident }
                    }
                } else {
                    let type_ident = syn::Ident::new(type_str, proc_macro2::Span::call_site());
                    quote! { #type_ident }
                }
            },
            _ => {
                // 检查是否是基本类型
                let is_primitive = matches!(type_str, 
                    "bool" | "u8" | "u16" | "u32" | "u64" | "u128" | 
                    "i8" | "i16" | "i32" | "i64" | "i128" | 
                    "String" | "string" | "Pubkey" | "publicKey" | "pubkey"
                );
                
                if is_primitive {
                    // 基本类型直接使用
                    let type_ident = syn::Ident::new(type_str, proc_macro2::Span::call_site());
                    quote! { #type_ident }
                } else {
                    // 自定义类型，使用完整路径
                    let type_path = format!("crate::types::{}", type_str);
                    match syn::parse_str::<syn::Path>(&type_path) {
                        Ok(path) => quote! { #path },
                        Err(_) => {
                            // 解析失败，使用直接类型名
                            let type_ident = syn::Ident::new(type_str, proc_macro2::Span::call_site());
                            quote! { #type_ident }
                        }
                    }
                }
            },
        }
    }

    /// 转换 NonAnchorFieldType 为 Rust 类型（SPL Token 专用版本）
    fn convert_typedef_field_type_to_rust_spl(&self, field_type: &NonAnchorFieldType) -> TokenStream {
        let is_spl = self.is_spl_token_program();
        log::debug!("🔍 convert_typedef_field_type_to_rust_spl: is_spl_token_program={}, address={}", is_spl, self.idl.address);
        
        if is_spl {
            match field_type {
                NonAnchorFieldType::Option { option } => {
                    if let NonAnchorFieldType::Basic(s) = option.as_ref() {
                        if matches!(s.as_str(), "publicKey" | "pubkey" | "Pubkey") {
                            log::debug!("✅ Converting Option<Pubkey> to COption<Pubkey> for SPL Token");
                            // SPL Token 程序中 Option<Pubkey> 转换为 COption<Pubkey>
                            // 添加调试注释以验证此路径被执行
                            return quote! { 
                                // DEBUG: SPL Token COption<Pubkey> 转换已执行
                                solana_program::program_option::COption<Pubkey> 
                            };
                        }
                    }
                    // 其他 Option 类型保持不变
                    let inner_type = self.convert_typedef_field_type_to_rust_spl(option);
                    quote! { Option<#inner_type> }
                },
                _ => Self::convert_typedef_field_type_to_rust(field_type)
            }
        } else {
            Self::convert_typedef_field_type_to_rust(field_type)
        }
    }

    /// 转换 NonAnchorFieldType 为 Rust 类型
    fn convert_typedef_field_type_to_rust(field_type: &NonAnchorFieldType) -> TokenStream {
        match field_type {
            NonAnchorFieldType::Basic(type_str) => {
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
            NonAnchorFieldType::Option { option } => {
                let inner_type = Self::convert_typedef_field_type_to_rust(option);
                quote! { Option<#inner_type> }
            },
            NonAnchorFieldType::Vec { vec } => {
                let inner_type = Self::convert_typedef_field_type_to_rust(vec);
                quote! { Vec<#inner_type> }
            },
            NonAnchorFieldType::Array { array } => {
                let (inner_type, size) = array;
                let inner_type_token = Self::convert_typedef_field_type_to_rust(inner_type);
                let size_literal = proc_macro2::Literal::usize_unsuffixed(*size);
                quote! { [#inner_type_token; #size_literal] }
            },
            NonAnchorFieldType::Defined { defined } => {
                // 使用完整路径引用types模块中的类型
                let type_path = format!("crate::types::{}", defined);
                let type_path: syn::Path = syn::parse_str(&type_path).unwrap();
                quote! { #type_path }
            },
            NonAnchorFieldType::HashMap { key, value } => {
                let key_type = Self::convert_typedef_field_type_to_rust(key);
                let value_type = Self::convert_typedef_field_type_to_rust(value);
                quote! { std::collections::HashMap<#key_type, #value_type> }
            },
            NonAnchorFieldType::Complex { kind, params } => {
                // 处理复合类型，如 Vec<T>, Option<T>, [T; N] 等 (Legacy支持)
                match kind.as_str() {
                    "Vec" => {
                        if let Some(params) = params {
                            if let Some(inner_type) = params.get(0) {
                                if let Some(inner_str) = inner_type.as_str() {
                                    let inner_type_token = Self::convert_typedef_field_type_to_rust(&NonAnchorFieldType::Basic(inner_str.to_string()));
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
                                    let inner_type_token = Self::convert_typedef_field_type_to_rust(&NonAnchorFieldType::Basic(inner_str.to_string()));
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

    /// 生成长度常量 (已废弃，现在在单个文件生成中处理)
    pub fn generate_len_constants(&self) -> TokenStream {
        quote! {}
    }

    /// 生成 try_to_vec 方法 (已废弃，现在在单个文件生成中处理)
    pub fn generate_try_to_vec_method(&self) -> TokenStream {
        quote! {}
    }

    /// 生成 from_bytes 方法 (已废弃，现在在单个文件生成中处理)
    pub fn generate_from_bytes_method(&self) -> TokenStream {
        quote! {}
    }

    /// 生成 Default 实现
    pub fn generate_default_impl(&self) -> TokenStream {
        let accounts = self.idl.accounts.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if accounts.is_empty() {
            return quote! {};
        }

        // 强制初始化字段分配缓存
        let _force_init = self.idl.get_field_allocation();
        
        let default_impls = accounts.iter().map(|account| {
            let struct_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            
            // 生成默认字段赋值
            let field_defaults = if let Some(account_fields) = &account.fields {
                // 优先级1：直接使用account.fields
                let default_values = account_fields.iter().map(|field| {
                    let (snake_field_name, _) = self.convert_field_name_with_serde(&field.name);
                    let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                    let default_value = Self::generate_field_default_from_typedef_field_type(&field.field_type);
                    quote! { #field_name: #default_value, }
                });
                quote! { #(#default_values)* }
            } else if let Some(allocated_fields) = self.idl.get_account_allocated_fields(&account.name) {
                // 优先级2：从字段分配获取字段
                let default_values = allocated_fields.iter().map(|field_def| {
                    let (snake_field_name, _) = self.convert_field_name_with_serde(&field_def.name);
                    let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                    let default_value = Self::generate_field_default_from_field_definition_type(&field_def.field_type);
                    quote! { #field_name: #default_value, }
                });
                quote! { #(#default_values)* }
            } else {
                quote! {} // 无字段，空默认实现
            };

            quote! {
                impl Default for #struct_name {
                    fn default() -> Self {
                        Self {
                            #field_defaults
                        }
                    }
                }
            }
        });

        quote! {
            #(#default_impls)*
        }
    }


    /// 从字段定义字符串类型生成默认值（用于字段分配机制）
    fn generate_field_default_from_field_definition_type(type_str: &str) -> TokenStream {
        match type_str {
            "bool" => quote! { false },
            "String" | "string" => quote! { String::new() },
            "Pubkey" | "publicKey" | "pubkey" => quote! { Default::default() },
            _ if type_str.starts_with("Vec<") && type_str.ends_with(">") => {
                quote! { Vec::new() }
            },
            _ if type_str.starts_with("Option<") && type_str.ends_with(">") => {
                quote! { None }
            },
            _ if type_str.starts_with("[") && type_str.contains(";") && type_str.ends_with("]") => {
                // Handle large arrays with custom types
                if let Some(semicolon_pos) = type_str.find(';') {
                    let inner_type = &type_str[1..semicolon_pos].trim();
                    let size_str = &type_str[semicolon_pos+1..type_str.len()-1].trim();
                    if let Ok(size) = size_str.parse::<usize>() {
                        if size > 32 {
                            // For large arrays with custom types, use array initialization
                            let size_literal = proc_macro2::Literal::usize_unsuffixed(size);
                            match *inner_type {
                                "u8" => quote! { [0u8; #size_literal] },
                                "u64" => quote! { [0u64; #size_literal] },
                                _ => quote! { [(); #size_literal].map(|_| Default::default()) },
                            }
                        } else {
                            quote! { Default::default() }
                        }
                    } else {
                        quote! { Default::default() }
                    }
                } else {
                    quote! { Default::default() }
                }
            },
            _ => quote! { Default::default() },
        }
    }

    /// 从NonAnchorFieldType生成字段默认值
    fn generate_field_default_from_typedef_field_type(field_type: &NonAnchorFieldType) -> TokenStream {
        match field_type {
            NonAnchorFieldType::Basic(type_str) => {
                match type_str.as_str() {
                    "bool" => quote! { false },
                    "String" | "string" => quote! { String::new() },
                    "bytes" => quote! { Vec::new() }, // bytes类型默认值为空Vec<u8>
                    _ => quote! { Default::default() },
                }
            },
            NonAnchorFieldType::Option { .. } => {
                quote! { None }
            },
            NonAnchorFieldType::Vec { .. } => {
                quote! { Vec::new() }
            },
            NonAnchorFieldType::Array { array } => {
                let (inner_type, size) = array;
                if *size > 32 {
                    // For large arrays, generate manual initialization
                    match inner_type.as_ref() {
                        NonAnchorFieldType::Basic(type_str) if type_str == "u8" => {
                            let size_literal = proc_macro2::Literal::usize_unsuffixed(*size);
                            quote! { [0u8; #size_literal] }
                        },
                        NonAnchorFieldType::Basic(type_str) if type_str == "u64" => {
                            let size_literal = proc_macro2::Literal::usize_unsuffixed(*size);
                            quote! { [0u64; #size_literal] }
                        },
                        NonAnchorFieldType::Basic(type_str) if type_str == "u128" => {
                            let size_literal = proc_macro2::Literal::usize_unsuffixed(*size);
                            quote! { [0u128; #size_literal] }
                        },
                        NonAnchorFieldType::Basic(type_str) if type_str == "u32" => {
                            let size_literal = proc_macro2::Literal::usize_unsuffixed(*size);
                            quote! { [0u32; #size_literal] }
                        },
                        NonAnchorFieldType::Basic(type_str) if type_str == "u16" => {
                            let size_literal = proc_macro2::Literal::usize_unsuffixed(*size);
                            quote! { [0u16; #size_literal] }
                        },
                        NonAnchorFieldType::Basic(type_str) if type_str.starts_with("i") => {
                            // Handle signed integer types
                            let size_literal = proc_macro2::Literal::usize_unsuffixed(*size);
                            let zero_literal = format!("0{}", type_str);
                            let zero_token = syn::Ident::new(&zero_literal, proc_macro2::Span::call_site());
                            quote! { [#zero_token; #size_literal] }
                        },
                        NonAnchorFieldType::Defined { .. } => {
                            // For arrays of custom types, use array initialization with Default
                            let size_literal = proc_macro2::Literal::usize_unsuffixed(*size);
                            quote! { [(); #size_literal].map(|_| Default::default()) }
                        },
                        _ => {
                            // For other basic types, use explicit zero initialization
                            let size_literal = proc_macro2::Literal::usize_unsuffixed(*size);
                            quote! { [(); #size_literal].map(|_| Default::default()) }
                        }
                    }
                } else {
                    quote! { Default::default() }
                }
            },
            NonAnchorFieldType::Defined { .. } => {
                quote! { Default::default() }
            },
            NonAnchorFieldType::HashMap { .. } => {
                quote! { std::collections::HashMap::new() }
            },
            NonAnchorFieldType::Complex { kind, params: _ } => {
                // Legacy支持
                match kind.as_str() {
                    "Vec" => quote! { Vec::new() },
                    "Option" => quote! { None },
                    _ => quote! { Default::default() },
                }
            },
        }
    }


    /// 为单个account生成完整的文件内容（NonAnchor使用长度识别）
    pub fn generate_single_account_file(&self, account: &crate::idl_format::non_anchor_idl::NonAnchorAccount) -> TokenStream {
        let struct_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
        let _len_const = syn::Ident::new(
            &format!("{}_LEN", account.name.to_shouty_snake_case()),
            proc_macro2::Span::call_site(),
        );
        
        // 强制初始化字段分配缓存
        let _force_init = self.idl.get_field_allocation();
        
        let doc_comments = DocGenerator::generate_doc_comments(&account.docs);
        let account_name_str = &account.name;
        
        // 计算 PACKED_LEN
        let packed_size = self.calculate_account_packed_size(account);
        
        // NonAnchor字段生成优先级：direct fields → field allocation → empty structures
        let struct_fields = if let Some(account_fields) = &account.fields {
            // 优先级1：直接使用account.fields
            let field_tokens = account_fields.iter().map(|field| {
                let (snake_field_name, serde_attr) = self.convert_field_name_with_serde(&field.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                let field_type = self.convert_typedef_field_type_to_rust_spl(&field.field_type);
                let field_docs = DocGenerator::generate_field_docs(&field.docs);
                
                // 检查是否为 SPL Token COption<Pubkey> 字段
                let coption_attrs = if self.is_spl_token_coption_field(&field.field_type) {
                    self.generate_spl_token_coption_attrs()
                } else {
                    quote! {}
                };

                // 检查是否为 Pubkey 类型，如果是则添加特殊的 serde 属性
                let pubkey_serde_attr = if Self::is_typedef_field_pubkey_type(&field.field_type) {
                    generate_pubkey_serde_attr()
                } else {
                    quote! {}
                };
                
                // 检查是否为大数组，如果是则添加 BigArray serde 属性
                let large_array_serde_attr = if self.is_large_array_field(&field.field_type) {
                    use crate::utils::generate_large_array_serde_attr;
                    if let NonAnchorFieldType::Array { array } = &field.field_type {
                        let (_, array_size) = array;
                        generate_large_array_serde_attr(*array_size).unwrap_or_else(|| quote! {})
                    } else {
                        quote! {}
                    }
                } else {
                    quote! {}
                };
                
                quote! {
                    #field_docs
                    #serde_attr
                    #coption_attrs
                    #pubkey_serde_attr
                    #large_array_serde_attr
                    pub #field_name: #field_type,
                }
            });
            Some(quote! { #(#field_tokens)* })
        } else if let Some(allocated_fields) = self.idl.get_account_allocated_fields(&account.name) {
            // 优先级2：从字段分配获取字段
            let field_tokens = allocated_fields.iter().map(|field_def| {
                let (snake_field_name, serde_attr) = self.convert_field_name_with_serde(&field_def.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                let field_type = Self::convert_field_definition_type_to_rust(&field_def.field_type);
                let field_docs = DocGenerator::generate_doc_comments(&Some(field_def.docs.clone()));
                
                // 检查字符串字段类型是否为 Pubkey
                let pubkey_serde_attr = if Self::is_string_field_pubkey_type(&field_def.field_type) {
                    generate_pubkey_serde_attr()
                } else {
                    quote! {}
                };
                
                // 检查是否为大数组（从字符串类型解析）
                let large_array_serde_attr = if self.is_large_array_field_from_string(&field_def.field_type) {
                    use crate::utils::{generate_large_array_serde_attr, parse_array_size};
                    if let Some(array_size) = parse_array_size(&field_def.field_type) {
                        generate_large_array_serde_attr(array_size).unwrap_or_else(|| quote! {})
                    } else {
                        quote! {}
                    }
                } else {
                    quote! {}
                };
                
                quote! {
                    #field_docs
                    #serde_attr
                    #pubkey_serde_attr
                    #large_array_serde_attr
                    pub #field_name: #field_type,
                }
            });
            Some(quote! { #(#field_tokens)* })
        } else {
            None // 优先级3：无任何字段定义，使用空结构体
        };
        
        let actual_struct_fields = struct_fields.unwrap_or_else(|| quote! {});

        // 生成默认实现
        // 优先级：direct fields → field allocation → empty defaults
        let default_fields = if let Some(account_fields) = &account.fields {
            // 优先级1：直接使用account.fields
            let default_values = account_fields.iter().map(|field| {
                let (snake_field_name, _) = self.convert_field_name_with_serde(&field.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                let default_value = Self::generate_field_default_from_typedef_field_type(&field.field_type);
                quote! { #field_name: #default_value, }
            });
            quote! { #(#default_values)* }
        } else if let Some(allocated_fields) = self.idl.get_account_allocated_fields(&account.name) {
            // 优先级2：从字段分配获取字段
            let default_values = allocated_fields.iter().map(|field_def| {
                let (snake_field_name, _) = self.convert_field_name_with_serde(&field_def.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                let default_value = Self::generate_field_default_from_field_definition_type(&field_def.field_type);
                quote! { #field_name: #default_value, }
            });
            quote! { #(#default_values)* }
        } else {
            quote! {} // 优先级3：无字段，空默认实现
        };

        let account_doc_comment = format!("Account: {} (NonAnchor)", account_name_str);
        
        // 检查IDL是否有types字段
        let _has_types_module = self.idl.types.as_ref().map_or(false, |types| !types.is_empty());
        
        // 生成导入语句 - 不使用通配符导入，类型引用使用完整路径
        let big_array_import = if self.has_large_arrays(account) {
            use crate::utils::generate_big_array_import;
            generate_big_array_import()
        } else {
            quote! {}
        };
        
        let imports = quote! {
            #[allow(unused_imports)]
            use solana_pubkey::Pubkey;
            #big_array_import
        };
        
        quote! {
            #![doc = #account_doc_comment]
            #doc_comments
            
            #imports
            
            // Account Structure
            #doc_comments
            #[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Clone, Debug, PartialEq)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            pub struct #struct_name {
                #actual_struct_fields
            }

            impl Default for #struct_name {
                fn default() -> Self {
                    Self {
                        #default_fields
                    }
                }
            }

            impl #struct_name {
                pub const MEM_LEN: usize = std::mem::size_of::<Self>();
                pub const PACKED_LEN: usize = #packed_size;
                
                pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
                    borsh::to_vec(self)
                }
                
                pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
                    if data.len() != Self::PACKED_LEN {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "Account data length mismatch. Expected: {}, got: {}",
                                Self::PACKED_LEN, data.len()
                            ),
                        ));
                    }
                    
                    borsh::BorshDeserialize::deserialize(&mut &data[..])
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                }
            }
        }
    }
}

impl<'a> TemplateGenerator for NonAnchorAccountsTemplate<'a> {
    fn get_standard_module_name(&self) -> &'static str {
        "accounts"
    }

    fn gen_files(&self) -> Vec<(String, TokenStream)> {
        let accounts = self.idl.accounts.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if accounts.is_empty() {
            return vec![("mod.rs".to_string(), quote! {
                //! Empty accounts module - no accounts found in IDL
            })];
        }
        
        let mut files = Vec::new();
        
        // 为每个account生成独立文件
        for account in accounts {
            let file_name = format!("{}.rs", account.name.to_case(Case::Snake));
            let file_content = self.generate_single_account_file(account);
            files.push((file_name, file_content));
        }
        
        files
    }

    fn gen_mod_file(&self) -> TokenStream {
        let accounts = self.idl.accounts.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if accounts.is_empty() {
            return quote! {
                //! Accounts module - no accounts found in IDL
            };
        }
        
        // 生成模块声明和重新导出
        let module_declarations = accounts.iter().map(|account| {
            let module_name = syn::Ident::new(&account.name.to_case(Case::Snake), proc_macro2::Span::call_site());
            quote! {
                pub mod #module_name;
            }
        });
        
        let re_exports = accounts.iter().map(|account| {
            let module_name = syn::Ident::new(&account.name.to_case(Case::Snake), proc_macro2::Span::call_site());
            quote! {
                pub use #module_name::*;
            }
        });
        
        quote! {
            //! Non-Anchor accounts module
            //! Generated account definitions with length-based identification
            //! Each account is defined in its own file
            
            #(#module_declarations)*
            
            // Re-export all account items
            #(#re_exports)*
        }
    }
}

impl<'a> NonAnchorAccountsTemplate<'a> {
    /// 检查账户是否包含大数组（> 32 元素）
    fn has_large_arrays(&self, account: &NonAnchorAccount) -> bool {
        // 检查直接字段
        if let Some(fields) = &account.fields {
            for field in fields {
                if self.is_large_array_field(&field.field_type) {
                    return true;
                }
            }
        }
        
        // 检查分配的字段
        if let Some(allocated_fields) = self.idl.get_account_allocated_fields(&account.name) {
            for field_def in allocated_fields {
                if self.is_large_array_field_from_string(&field_def.field_type) {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// 检查 NonAnchorFieldType 是否为大数组
    fn is_large_array_field(&self, field_type: &NonAnchorFieldType) -> bool {
        if let NonAnchorFieldType::Array { array } = field_type {
            let (_, size) = array;
            return *size > 32;
        }
        false
    }
    
    /// 检查字符串类型是否为大数组（用于分配字段）
    fn is_large_array_field_from_string(&self, type_str: &str) -> bool {
        use crate::utils::parse_array_size;
        if let Some(size) = parse_array_size(type_str) {
            return size > 32;
        }
        false
    }

    /// 计算账户的 PACKED_LEN 大小（SPL Token 专用版本）
    fn calculate_account_packed_size(&self, account: &NonAnchorAccount) -> usize {
        let mut size = 0; // NonAnchor 没有 discriminator
        
        // 添加字段大小
        if let Some(fields) = &account.fields {
            for field in fields {
                size += self.calculate_field_size_spl(&field.field_type);
            }
        }
        
        size
    }

    /// 计算账户的 PACKED_LEN 大小（通用版本）
    #[allow(dead_code)]
    fn calculate_account_packed_size_static(account: &NonAnchorAccount) -> usize {
        let mut size = 0; // NonAnchor 没有 discriminator
        
        // 添加字段大小
        if let Some(fields) = &account.fields {
            for field in fields {
                size += Self::calculate_field_size(&field.field_type);
            }
        }
        
        size
    }
    
    /// 计算单个字段的序列化大小（SPL Token 专用版本）
    fn calculate_field_size_spl(&self, field_type: &NonAnchorFieldType) -> usize {
        if self.is_spl_token_program() {
            match field_type {
                NonAnchorFieldType::Option { option } => {
                    if let NonAnchorFieldType::Basic(s) = option.as_ref() {
                        if matches!(s.as_str(), "publicKey" | "pubkey" | "Pubkey") {
                            // SPL Token 程序中 Option<Pubkey> -> COption<Pubkey> = 36 字节
                            return 36; // 4字节标志位 + 32字节公钥
                        }
                    }
                    // 其他 Option 类型：1字节标志 + 内部类型大小
                    1 + self.calculate_field_size_spl(option)
                },
                _ => Self::calculate_field_size(field_type)
            }
        } else {
            Self::calculate_field_size(field_type)
        }
    }

    /// 计算单个字段的序列化大小
    fn calculate_field_size(field_type: &NonAnchorFieldType) -> usize {
        match field_type {
            NonAnchorFieldType::Basic(type_name) => {
                match type_name.as_str() {
                    "bool" => 1,
                    "u8" | "i8" => 1,
                    "u16" | "i16" => 2,
                    "u32" | "i32" => 4,
                    "u64" | "i64" => 8,
                    "u128" | "i128" => 16,
                    "f32" => 4,
                    "f64" => 8,
                    "publicKey" | "pubkey" | "Pubkey" => 32,
                    "string" => 4 + 0, // Vec<u8> prefix (4 bytes) + variable content (估算为0)
                    _ => 8, // 默认大小
                }
            },
            NonAnchorFieldType::Array { array: (inner_type, size) } => {
                Self::calculate_field_size(inner_type) * size
            },
            NonAnchorFieldType::Option { option: inner_type } => {
                1 + Self::calculate_field_size(inner_type) // Option flag (1 byte) + inner type size
            },
            NonAnchorFieldType::Vec { vec: _inner_type } => {
                4 + 0 // Vec length prefix (4 bytes) + variable content (估算为0)
            },
            NonAnchorFieldType::Defined { .. } => {
                8 // 自定义类型默认估算
            },
            NonAnchorFieldType::HashMap { .. } => {
                4 + 0 // HashMap length prefix (4 bytes) + variable content (估算为0)
            },
            NonAnchorFieldType::Complex { .. } => {
                8 // 复合类型默认大小
            },
        }
    }
}