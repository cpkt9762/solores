//! Anchor Accounts 模板
//!
//! 为 Anchor 合约生成 Accounts 相关代码，使用 8 字节 discriminator

use proc_macro2::TokenStream;
use quote::quote;
use convert_case::{Case, Casing};
use heck::ToShoutySnakeCase;
use sha2::{Digest};

use crate::idl_format::anchor_idl::AnchorIdl;
use crate::idl_format::anchor_idl::AnchorAccount;
use crate::idl_format::anchor_idl::{AnchorType, AnchorTypeKind, AnchorFieldType};
use crate::Args;
use crate::templates::TemplateGenerator;
use crate::templates::common::{doc_generator::DocGenerator};
use crate::utils::{to_snake_case_with_serde, generate_pubkey_serde_attr};

/// Anchor Accounts 模板
pub struct AnchorAccountsTemplate<'a> {
    pub idl: &'a AnchorIdl,
    pub args: &'a Args,
}

impl<'a> AnchorAccountsTemplate<'a> {
    /// 创建 Anchor Accounts 模板（统一使用字段分配机制）
    pub fn new(idl: &'a AnchorIdl, args: &'a Args) -> Self {
        Self { idl, args }
    }

    /// 生成智能的默认值，处理大数组等特殊情况
    fn generate_smart_default_value(field_type: &str) -> TokenStream {
        // 检查是否是大数组类型
        if field_type.starts_with("[u8; ") && field_type.ends_with("]") {
            // 提取数组大小
            if let Some(size_str) = field_type.strip_prefix("[u8; ").and_then(|s| s.strip_suffix("]")) {
                if let Ok(size) = size_str.parse::<usize>() {
                    if size > 32 {
                        // 大数组需要特殊处理，因为Rust不为大于32的数组实现Default
                        let size_literal = proc_macro2::Literal::usize_unsuffixed(size);
                        return quote! { [0u8; #size_literal] };
                    }
                }
            }
        }
        // 其他所有情况使用Default::default()
        quote! { Default::default() }
    }

    /// 生成账户结构体
    pub fn generate_account_structs(&self) -> TokenStream {
        let accounts = self.idl.accounts.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if accounts.is_empty() {
            return quote! {};
        }

        let structs = accounts.iter().filter_map(|account| {
            let struct_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            
            // 开始处理账户
            log::debug!("🏦 Accounts: 开始处理Account: {}", account.name);
            log::debug!("🏦 Accounts: Account '{}' fields状态: {:?}", account.name,
                account.fields.as_ref().map(|f| format!("Some({}个字段)", f.len())).unwrap_or("None".to_string()));
            
            // 统一使用字段分配机制：优先使用账户直接字段，否则从字段分配中获取
            let fields = if let Some(account_fields) = &account.fields {
                log::debug!("🏦 Accounts: Account '{}' 有直接字段定义，使用直接字段", account.name);
                let doc_comments = DocGenerator::generate_doc_comments(&account.docs);
                let struct_fields = account_fields.iter().map(|field| {
                    let (snake_field_name, serde_attr) = to_snake_case_with_serde(&field.name);
                    let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                    let field_type = Self::convert_typedef_field_type_to_rust(&field.field_type);
                    let field_docs = DocGenerator::generate_field_docs(&field.docs);
                    
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
                        pub #field_name: #field_type,
                    }
                });

                Some((doc_comments, quote! {
                    pub discriminator: [u8; 8],
                    #(#struct_fields)*
                }))
            } else {
                // 账户没有直接字段，使用IDL字段分配机制
                log::debug!("🏦 Accounts: Account '{}' 没有直接字段，尝试从字段分配获取", account.name);
                log::debug!("🏦 Accounts: Account '{}' 查询字段分配结果...", account.name);
                if let Some(allocated_fields) = self.idl.get_account_allocated_fields(&account.name) {
                    log::debug!("✅ Accounts: Account '{}' 从字段分配获取{}个字段: {:?}",
                        account.name, allocated_fields.len(),
                        allocated_fields.iter().map(|f| &f.name).collect::<Vec<_>>());
                    let doc_comments = DocGenerator::generate_doc_comments(&account.docs);
                    let struct_fields = allocated_fields.iter().map(|field_def| {
                        let (snake_field_name, serde_attr) = to_snake_case_with_serde(&field_def.name);
                        let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                        // 改进类型转换逻辑
                        let field_type = Self::convert_field_definition_type_to_rust(&field_def.field_type);
                        let field_docs = if field_def.docs.is_empty() { 
                            quote! {} 
                        } else { 
                            DocGenerator::generate_doc_comments(&Some(field_def.docs.clone())) 
                        };
                        
                        quote! {
                            #field_docs
                            #serde_attr
                            pub #field_name: #field_type,
                        }
                    });

                    Some((doc_comments, quote! {
                        pub discriminator: [u8; 8],
                        #(#struct_fields)*
                    }))
                } else {
                    log::debug!("❌ Accounts: Account '{}' 字段分配查询失败", account.name);
                    // 进一步检查字段分配映射的内容
                    let allocation = self.idl.get_field_allocation();
                    log::debug!("🔍 Accounts: 当前字段分配映射包含accounts: {:?}",
                        allocation.accounts_fields.keys().collect::<Vec<_>>());
                    
                    // 回退到只有discriminator
                    let doc_comments = DocGenerator::generate_account_docs(account);
                    Some((doc_comments, quote! { pub discriminator: [u8; 8], }))
                }
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

        quote! {
            #(#structs)*
        }
    }

    /// 生成账户字段（保留用于向后兼容）
    fn generate_account_fields(_account_type: &str) -> TokenStream {
        // This method is kept for backward compatibility but is no longer used
        // All account field generation is now handled through named_types lookup
        quote! { pub discriminator: [u8; 8], }
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
                        // 解析失败，使用u8作为fallback
                        log::warn!("⚠️  数组大小解析失败: '{}', 使用u8作为fallback", type_str);
                        let type_ident = syn::Ident::new("u8", proc_macro2::Span::call_site());
                        quote! { #type_ident }
                    }
                } else {
                    // 格式不正确，使用u8作为fallback
                    log::warn!("⚠️  数组格式不正确: '{}', 使用u8作为fallback", type_str);
                    let type_ident = syn::Ident::new("u8", proc_macro2::Span::call_site());
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
                // 检查类型字符串是否为空
                if type_str.is_empty() {
                    log::warn!("⚠️  类型字符串为空，使用u8作为fallback");
                    quote! { u8 }
                } else if type_str.contains('[') || type_str.contains('<') || type_str.contains(';') {
                    // 复合类型表达式（如 [[u64; 8]; 12], Vec<String>, Option<u32>）
                    log::debug!("🔄 解析复合类型表达式: '{}'", type_str);
                    match syn::parse_str::<syn::Type>(type_str) {
                        Ok(ty) => {
                            log::debug!("✅ 成功解析类型表达式: '{}'", type_str);
                            quote! { #ty }
                        },
                        Err(e) => {
                            log::warn!("⚠️  无效的类型表达式: '{}', 错误: {}, 使用u8作为fallback", type_str, e);
                            quote! { u8 }
                        }
                    }
                } else {
                    // 简单标识符（如 String, u64, CustomType）
                    log::debug!("🔄 解析简单标识符: '{}'", type_str);
                    
                    // 检查是否是基本类型
                    let is_primitive = matches!(type_str, 
                        "bool" | "u8" | "u16" | "u32" | "u64" | "u128" | 
                        "i8" | "i16" | "i32" | "i64" | "i128" | 
                        "String" | "string" | "Pubkey" | "publicKey" | "pubkey"
                    );
                    
                    if is_primitive {
                        // 基本类型直接使用
                        match syn::parse_str::<syn::Ident>(type_str) {
                            Ok(type_ident) => {
                                log::debug!("✅ 成功解析基本类型标识符: '{}'", type_str);
                                quote! { #type_ident }
                            },
                            Err(e) => {
                                log::warn!("⚠️  无效的Rust标识符: '{}', 错误: {}, 使用u8作为fallback", type_str, e);
                                quote! { u8 }
                            }
                        }
                    } else {
                        // 自定义类型，使用完整路径
                        let type_path = format!("crate::types::{}", type_str);
                        match syn::parse_str::<syn::Path>(&type_path) {
                            Ok(path) => {
                                log::debug!("✅ 成功解析自定义类型路径: '{}'", type_path);
                                quote! { #path }
                            },
                            Err(e) => {
                                log::warn!("⚠️  无效的类型路径: '{}', 错误: {}, 使用u8作为fallback", type_path, e);
                                quote! { u8 }
                            }
                        }
                    }
                }
            }
        }
    }

    /// 转换 AnchorFieldType 为 Rust 类型
    fn convert_typedef_field_type_to_rust(field_type: &AnchorFieldType) -> TokenStream {
        match field_type {
            AnchorFieldType::Basic(type_str) => {
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
            AnchorFieldType::defined(type_name) => {
                // 使用完整路径引用types模块中的类型
                let type_path = format!("crate::types::{}", type_name);
                let type_path: syn::Path = syn::parse_str(&type_path).unwrap();
                quote! { #type_path }
            },
            AnchorFieldType::array(inner_type, size) => {
                let inner_type = Self::convert_typedef_field_type_to_rust(inner_type);
                let size_literal = proc_macro2::Literal::usize_unsuffixed(*size);
                quote! { [#inner_type; #size_literal] }
            },
            AnchorFieldType::vec(inner_type) => {
                let inner = Self::convert_typedef_field_type_to_rust(inner_type);
                quote! { Vec<#inner> }
            },
            AnchorFieldType::option(inner_type) => {
                let inner = Self::convert_typedef_field_type_to_rust(inner_type);
                quote! { Option<#inner> }
            },
            AnchorFieldType::PrimitiveOrPubkey(type_str) => {
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
            AnchorFieldType::Complex { kind, params: _ } => {
                let type_ident = syn::Ident::new(kind, proc_macro2::Span::call_site());
                quote! { #type_ident }
            },
        }
    }

    /// 生成 discriminator 常量
    pub fn generate_discriminator_constants(&self) -> TokenStream {
        let accounts = self.idl.accounts.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if accounts.is_empty() {
            return quote! {};
        }

        let constants = accounts.iter().map(|account| {
            let const_name = syn::Ident::new(
                &format!("{}_ACCOUNT_DISCM", account.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );
            
            // Use the discriminator from IDL (8-byte array for Anchor)
            let discriminator = {
                let bytes = account.discriminator.iter().map(|&b| b).collect::<Vec<_>>();
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

    /// 生成长度常量
    pub fn generate_len_constants(&self) -> TokenStream {
        let accounts = self.idl.accounts.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if accounts.is_empty() {
            return quote! {};
        }

        let len_constants = accounts.iter().map(|account| {
            let struct_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            
            quote! {
                impl #struct_name {
                    pub const LEN: usize = std::mem::size_of::<Self>();
                }
            }
        });

        quote! {
            #(#len_constants)*
        }
    }

    /// 生成 try_to_vec 方法
    pub fn generate_try_to_vec_method(&self) -> TokenStream {
        let accounts = self.idl.accounts.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if accounts.is_empty() {
            return quote! {};
        }

        let try_to_vec_methods = accounts.iter().map(|account| {
            let struct_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            
            quote! {
                impl #struct_name {
                    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
                        borsh::to_vec(self)
                    }
                }
            }
        });

        quote! {
            #(#try_to_vec_methods)*
        }
    }

    /// 生成 from_bytes 方法
    pub fn generate_from_bytes_method(&self) -> TokenStream {
        let accounts = self.idl.accounts.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if accounts.is_empty() {
            return quote! {};
        }

        let from_bytes_methods = accounts.iter().map(|account| {
            let struct_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let const_name = syn::Ident::new(
                &format!("{}_ACCOUNT_DISCM", account.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );
            
            quote! {
                impl #struct_name {
                    pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
                        if data.len() < 8 {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Account data too short for discriminator",
                            ));
                        }
                        if &data[0..8] != #const_name {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!(
                                    "Discriminator mismatch. Expected: {:?}, got: {:?}",
                                    #const_name,
                                    &data[0..8]
                                ),
                            ));
                        }
                        borsh::from_slice(data)
                    }
                }
            }
        });

        quote! {
            #(#from_bytes_methods)*
        }
    }

    /// 生成 Default 实现
    pub fn generate_default_impl(&self) -> TokenStream {
        let accounts = self.idl.accounts.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if accounts.is_empty() {
            return quote! {};
        }

        let default_impls = accounts.iter().map(|account| {
            let struct_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let const_name = syn::Ident::new(
                &format!("{}_ACCOUNT_DISCM", account.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );
            
            // Generate default field assignments using field allocation
            let field_defaults = if let Some(allocated_fields) = self.idl.get_account_allocated_fields(&account.name) {
                let default_assignments = allocated_fields.iter().map(|field_def| {
                    let (snake_field_name, _) = to_snake_case_with_serde(&field_def.name);
                    let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                    let default_value = Self::generate_smart_default_value(&field_def.field_type);
                    quote! { #field_name: #default_value, }
                });
                quote! { #(#default_assignments)* }
            } else {
                quote! {}
            };

            quote! {
                impl Default for #struct_name {
                    fn default() -> Self {
                        Self {
                            discriminator: #const_name,
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


    /// 从named_type生成默认字段赋值
    fn generate_default_field_assignments_from_type(named_type: &AnchorType) -> TokenStream {
        if let Some(type_def) = &named_type.kind {
            if let AnchorTypeKind::Struct(typedef_struct) = type_def {
                let field_assignments = typedef_struct.iter().map(|field| {
                    let (snake_field_name, _) = to_snake_case_with_serde(&field.name);
                    let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                    
                    // Generate appropriate default values based on field type
                    let default_value = Self::generate_field_default_from_typedef_field_type(&field.field_type);
                    
                    quote! {
                        #field_name: #default_value,
                    }
                });
                
                quote! { #(#field_assignments)* }
            } else {
                quote! {}
            }
        } else {
            quote! {}
        }
    }


    /// 从AnchorFieldType生成字段默认值
    fn generate_field_default_from_typedef_field_type(field_type: &AnchorFieldType) -> TokenStream {
        match field_type {
            AnchorFieldType::array(inner_type, size) => {
                // Handle specific array types based on inner type
                let size_literal = proc_macro2::Literal::usize_unsuffixed(*size);
                match &**inner_type {
                    AnchorFieldType::Basic(primitive) => {
                        match primitive.as_str() {
                            "u64" => quote! { [0u64; #size_literal] },
                            "u32" => quote! { [0u32; #size_literal] },
                            "u16" => quote! { [0u16; #size_literal] },
                            "u8" => quote! { [0u8; #size_literal] },
                            _ => quote! { Default::default() },
                        }
                    },
                    _ => quote! { Default::default() },
                }
            },
            AnchorFieldType::vec(_) => quote! { Vec::new() },
            AnchorFieldType::option(_) => quote! { None },
            _ => quote! { Default::default() },
        }
    }

    /// 为单个account生成完整的文件内容
    pub fn generate_single_account_file(&self, account: &crate::idl_format::anchor_idl::AnchorAccount) -> TokenStream {
        // 生成该account的常量
        let const_name = syn::Ident::new(
            &format!("{}_ACCOUNT_DISCM", account.name.to_shouty_snake_case()),
            proc_macro2::Span::call_site(),
        );
        let len_const = syn::Ident::new(
            &format!("{}_LEN", account.name.to_shouty_snake_case()),
            proc_macro2::Span::call_site(),
        );
        
        let discriminator = {
            let bytes = account.discriminator.iter().map(|&b| b).collect::<Vec<_>>();
            quote! { [#(#bytes),*] }
        };

        // 生成结构体
        let struct_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
        let doc_comments = DocGenerator::generate_doc_comments(&account.docs);
        
        log::debug!("📄 SingleFile: Account '{}' 开始生成独立文件", account.name);
        
        // 确保字段分配缓存已初始化
        let _force_init = self.idl.get_field_allocation();
        log::debug!("📄 SingleFile: Account '{}' 字段分配缓存强制初始化完成", account.name);
        
        // 统一使用字段分配机制：优先使用账户直接字段，否则从字段分配中获取
        let (struct_fields, default_fields) = if let Some(fields) = &account.fields {
            log::debug!("📄 SingleFile: Account '{}' 有直接字段定义，使用直接字段", account.name);
            let field_tokens = fields.iter().map(|field| {
                let (snake_field_name, serde_attr) = to_snake_case_with_serde(&field.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                let field_type = Self::convert_typedef_field_type_to_rust(&field.field_type);
                let field_docs = DocGenerator::generate_field_docs(&field.docs);
                
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
                    pub #field_name: #field_type,
                }
            });
            let default_values = fields.iter().map(|field| {
                let (snake_field_name, _) = to_snake_case_with_serde(&field.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                let default_value = Self::generate_field_default_from_typedef_field_type(&field.field_type);
                quote! { #field_name: #default_value, }
            });
            (
                quote! {
                    pub discriminator: [u8; 8],
                    #(#field_tokens)*
                },
                quote! {
                    discriminator: #const_name,
                    #(#default_values)*
                }
            )
        } else {
            // 账户没有直接字段，使用字段分配机制
            log::debug!("📄 SingleFile: Account '{}' 没有直接字段，尝试从字段分配获取", account.name);
            if let Some(allocated_fields) = self.idl.get_account_allocated_fields(&account.name) {
                log::debug!("✅ SingleFile: Account '{}' 从字段分配获取{}个字段", account.name, allocated_fields.len());
                let field_tokens = allocated_fields.iter().map(|field_def| {
                    let (snake_field_name, serde_attr) = to_snake_case_with_serde(&field_def.name);
                    let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                    let field_type = Self::convert_field_definition_type_to_rust(&field_def.field_type);
                    let field_docs = if field_def.docs.is_empty() { 
                        quote! {} 
                    } else { 
                        DocGenerator::generate_doc_comments(&Some(field_def.docs.clone())) 
                    };
                    
                    // 检查字符串类型是否为 Pubkey
                    let pubkey_serde_attr = if Self::is_field_definition_pubkey_type(&field_def.field_type) {
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
                    let (snake_field_name, _) = to_snake_case_with_serde(&field_def.name);
                    let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                    let default_value = Self::generate_smart_default_value(&field_def.field_type);
                    quote! { #field_name: #default_value, }
                });
                (
                    quote! {
                        pub discriminator: [u8; 8],
                        #(#field_tokens)*
                    },
                    quote! {
                        discriminator: #const_name,
                        #(#default_values)*
                    }
                )
            } else {
                log::debug!("❌ SingleFile: Account '{}' 字段分配查询失败，只生成discriminator", account.name);
                (
                    quote! {
                        pub discriminator: [u8; 8],
                    },
                    quote! {
                        discriminator: #const_name,
                    }
                )
            }
        };

        // 计算 PACKED_LEN 
        let packed_size = self.calculate_account_packed_size(account);
        log::debug!("🎯 最终 PACKED_LEN 计算结果：{} 字节 (账户: {})", packed_size, account.name);
        
        let _account_name_str = &account.name;

        quote! {
            #doc_comments
            
                        use solana_pubkey::Pubkey;
            
            // Constants
            pub const #const_name: [u8; 8] = #discriminator;
            
            // Account Structure
            #doc_comments
            #[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Clone, Debug, PartialEq)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            pub struct #struct_name {
                #struct_fields
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
                
                pub fn discriminator() -> [u8; 8] {
                    #const_name
                }
                
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
                    
                    let expected_discriminator = Self::discriminator();
                    if &data[0..8] != expected_discriminator {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "Discriminator mismatch. Expected: {:?}, got: {:?}",
                                expected_discriminator,
                                &data[0..8]
                            ),
                        ));
                    }
                    
                    borsh::BorshDeserialize::deserialize(&mut &data[..])
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
                }
            }
        }
    }

    /// 检查字段类型是否为 Pubkey (typedef field)
    fn is_typedef_field_pubkey_type(field_type: &AnchorFieldType) -> bool {
        match field_type {
            AnchorFieldType::Basic(s) => {
                matches!(s.as_str(), "publicKey" | "pubkey" | "Pubkey")
            },
            AnchorFieldType::PrimitiveOrPubkey(s) => {
                matches!(s.as_str(), "publicKey" | "pubkey" | "Pubkey")
            },
            _ => false
        }
    }

    /// 检查字段定义类型是否为 Pubkey (string field)
    fn is_field_definition_pubkey_type(type_str: &str) -> bool {
        matches!(type_str, "publicKey" | "pubkey" | "Pubkey")
    }
    
    /// 计算账户的 PACKED_LEN 大小
    fn calculate_account_packed_size(&self, account: &AnchorAccount) -> usize {
        let mut size = 8; // Anchor 账户总是有 8 字节 discriminator
        
        log::debug!("🧮 计算账户 {} 的大小，开始大小: {} (discriminator)", account.name, size);
        
        // 统一字段获取逻辑：优先使用直接字段，否则从字段分配获取
        if let Some(fields) = &account.fields {
            log::debug!("  🎯 使用直接字段 ({} 个)", fields.len());
            for field in fields {
                let field_size = Self::calculate_field_size(&field.field_type);
                log::debug!("  📐 字段 {} ({:?}): {} 字节", field.name, field.field_type, field_size);
                size += field_size;
            }
        } else {
            // 账户没有直接字段，尝试从字段分配获取
            log::debug!("  🔍 账户无直接字段，从字段分配获取");
            if let Some(allocated_fields) = self.idl.get_account_allocated_fields(&account.name) {
                log::debug!("  🎯 从字段分配获取 {} 个字段", allocated_fields.len());
                for field_def in allocated_fields {
                    let field_size = Self::calculate_field_definition_size(&field_def.field_type);
                    log::debug!("  📐 字段 {} ({}): {} 字节", field_def.name, field_def.field_type, field_size);
                    size += field_size;
                }
            } else {
                log::debug!("  ❌ 无法获取字段分配，只有 discriminator");
            }
        }
        
        log::debug!("🏁 账户 {} 总大小: {} 字节", account.name, size);
        size
    }
    
    /// 计算 FieldDefinition 字段的序列化大小（字段分配使用）
    fn calculate_field_definition_size(field_type: &str) -> usize {
        // 处理数组类型，如 "[u64; 16]"
        if field_type.starts_with('[') && field_type.ends_with(']') {
            if let Some(array_inner) = field_type.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some((inner_type, size_str)) = array_inner.split_once("; ") {
                    if let Ok(size) = size_str.parse::<usize>() {
                        let inner_size = Self::calculate_field_definition_size(inner_type.trim());
                        return inner_size * size;
                    }
                }
            }
        }
        
        // 基础类型大小映射
        match field_type {
            "bool" => 1,
            "u8" | "i8" => 1,
            "u16" | "i16" => 2,
            "u32" | "i32" | "f32" => 4,
            "u64" | "i64" | "f64" => 8,
            "u128" | "i128" => 16,
            "pubkey" | "Pubkey" | "publicKey" => 32,
            "string" => 4, // String 在 Borsh 中是长度前缀(4字节) + 内容
            _ => {
                // 自定义类型默认估算为8字节
                log::debug!("  🤔 未知类型 '{}' 默认为8字节", field_type);
                8
            }
        }
    }
    
    /// 计算单个字段的序列化大小
    fn calculate_field_size(field_type: &AnchorFieldType) -> usize {
        match field_type {
            AnchorFieldType::Basic(type_name) => {
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
            AnchorFieldType::PrimitiveOrPubkey(type_name) => {
                match type_name.as_str() {
                    "publicKey" | "pubkey" | "Pubkey" => 32,
                    _ => 8, // 其他基本类型默认
                }
            },
            AnchorFieldType::array(inner_type, size) => {
                Self::calculate_field_size(inner_type) * size
            },
            AnchorFieldType::option(_inner_type) => {
                1 + 0 // Option flag (1 byte) + inner type size (估算为0)
            },
            AnchorFieldType::vec(_inner_type) => {
                4 + 0 // Vec length prefix (4 bytes) + variable content (估算为0)
            },
            AnchorFieldType::defined(_type_name) => {
                8 // 自定义类型默认估算
            },
            AnchorFieldType::Complex { .. } => {
                8 // 复合类型默认估算
            },
            _ => 8, // 其他类型默认
        }
    }
}


impl<'a> TemplateGenerator for AnchorAccountsTemplate<'a> {
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
            //! Anchor accounts module
            //! Generated account definitions with 8-byte discriminator support
            //! Each account is defined in its own file
            
            #(#module_declarations)*
            
            // Re-export all account items
            #(#re_exports)*
        }
    }
}