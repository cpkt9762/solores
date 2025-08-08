//! 非 Anchor Accounts 模板
//!
//! 为非 Anchor 合约生成 Accounts 相关代码，基于长度识别

use proc_macro2::TokenStream;
use quote::quote;
use convert_case::{Case, Casing};
use heck::ToShoutySnakeCase;

use crate::idl_format::non_anchor_idl::NonAnchorIdl;
use crate::idl_format::non_anchor_idl::NonAnchorFieldType;
use crate::Args;
use crate::templates::TemplateGenerator;
use crate::templates::common::{doc_generator::DocGenerator};

/// 非 Anchor Accounts 模板
pub struct NonAnchorAccountsTemplate<'a> {
    pub idl: &'a NonAnchorIdl,
    pub args: &'a Args,
}

impl<'a> NonAnchorAccountsTemplate<'a> {
    /// 创建新的非 Anchor Accounts 模板
    pub fn new(idl: &'a NonAnchorIdl, args: &'a Args) -> Self {
        Self { idl, args }
    }

    /// 生成基于长度的账户识别
    pub fn generate_length_based_unpack(&self) -> TokenStream {
        let accounts = self.idl.accounts.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if accounts.is_empty() {
            return quote! {};
        }

        let length_checks = accounts.iter().map(|account| {
            let struct_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let len_const = syn::Ident::new(
                &format!("{}_LEN", account.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );

            quote! {
                if data.len() == #len_const {
                    return Ok(ProgramAccount::#struct_name(
                        borsh::from_slice(data)?
                    ));
                }
            }
        });

        quote! {
            pub fn try_unpack_account(data: &[u8]) -> Result<ProgramAccount, std::io::Error> {
                #(#length_checks)*
                
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unknown account length: {}", data.len()),
                ))
            }
        }
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
                    let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());
                    let field_type = Self::convert_typedef_field_type_to_rust(&field.field_type);
                    let field_docs = DocGenerator::generate_field_docs(&field.docs);
                    
                    quote! {
                        #field_docs
                        pub #field_name: #field_type,
                    }
                });

                Some((doc_comments, quote! {
                    #(#struct_fields)*
                }))
            } else if let Some(allocated_fields) = self.idl.get_account_allocated_fields(&account.name) {
                // 优先级2：从字段分配获取字段
                let struct_fields = allocated_fields.iter().map(|field_def| {
                    let field_name = syn::Ident::new(&field_def.name, proc_macro2::Span::call_site());
                    let field_type = Self::convert_field_definition_type_to_rust(&field_def.field_type);
                    let field_docs = DocGenerator::generate_doc_comments(&Some(field_def.docs.clone()));
                    
                    quote! {
                        #field_docs
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
                    #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, Eq)]
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
                // 其他情况视为自定义类型或者结构体
                let type_ident = syn::Ident::new(type_str, proc_macro2::Span::call_site());
                quote! { #type_ident }
            },
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
                let type_name = syn::Ident::new(defined, proc_macro2::Span::call_site());
                quote! { #type_name }
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

    /// 生成长度常量
    pub fn generate_len_constants(&self) -> TokenStream {
        let accounts = self.idl.accounts.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if accounts.is_empty() {
            return quote! {};
        }

        let len_constants = accounts.iter().map(|account| {
            let struct_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let len_const = syn::Ident::new(
                &format!("{}_LEN", account.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );
            
            quote! {
                pub const #len_const: usize = std::mem::size_of::<#struct_name>();
                
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
            let len_const = syn::Ident::new(
                &format!("{}_LEN", account.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );
            
            quote! {
                impl #struct_name {
                    pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
                        if data.len() != #len_const {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!(
                                    "Account data length mismatch. Expected: {}, got: {}",
                                    #len_const, data.len()
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

        // 强制初始化字段分配缓存
        let _force_init = self.idl.get_field_allocation();
        
        let default_impls = accounts.iter().map(|account| {
            let struct_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            
            // 生成默认字段赋值
            let field_defaults = if let Some(account_fields) = &account.fields {
                // 优先级1：直接使用account.fields
                let default_values = account_fields.iter().map(|field| {
                    let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());
                    let default_value = Self::generate_field_default_from_typedef_field_type(&field.field_type);
                    quote! { #field_name: #default_value, }
                });
                quote! { #(#default_values)* }
            } else if let Some(allocated_fields) = self.idl.get_account_allocated_fields(&account.name) {
                // 优先级2：从字段分配获取字段
                let default_values = allocated_fields.iter().map(|field_def| {
                    let field_name = syn::Ident::new(&field_def.name, proc_macro2::Span::call_site());
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

    /// 生成账户枚举（用于统一解析）
    pub fn generate_account_enum(&self) -> TokenStream {
        let accounts = self.idl.accounts.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if accounts.is_empty() {
            return quote! {};
        }

        let enum_variants = accounts.iter().map(|account| {
            let variant_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let struct_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            
            quote! {
                #variant_name(#struct_name),
            }
        });

        quote! {
            /// Unified account enum for all program accounts
            #[derive(Clone, Debug, PartialEq)]
            pub enum ProgramAccount {
                #(#enum_variants)*
            }
        }
    }

    /// 为单个account生成完整的文件内容（NonAnchor使用长度识别）
    pub fn generate_single_account_file(&self, account: &crate::idl_format::non_anchor_idl::NonAnchorAccount) -> TokenStream {
        let struct_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
        let len_const = syn::Ident::new(
            &format!("{}_LEN", account.name.to_shouty_snake_case()),
            proc_macro2::Span::call_site(),
        );
        
        // 强制初始化字段分配缓存
        let _force_init = self.idl.get_field_allocation();
        
        let doc_comments = DocGenerator::generate_doc_comments(&account.docs);
        let account_name_str = &account.name;
        
        // NonAnchor字段生成优先级：direct fields → field allocation → empty structures
        let struct_fields = if let Some(account_fields) = &account.fields {
            // 优先级1：直接使用account.fields
            let field_tokens = account_fields.iter().map(|field| {
                let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());
                let field_type = Self::convert_typedef_field_type_to_rust(&field.field_type);
                let field_docs = DocGenerator::generate_field_docs(&field.docs);
                
                quote! {
                    #field_docs
                    pub #field_name: #field_type,
                }
            });
            Some(quote! { #(#field_tokens)* })
        } else if let Some(allocated_fields) = self.idl.get_account_allocated_fields(&account.name) {
            // 优先级2：从字段分配获取字段
            let field_tokens = allocated_fields.iter().map(|field_def| {
                let field_name = syn::Ident::new(&field_def.name, proc_macro2::Span::call_site());
                let field_type = Self::convert_field_definition_type_to_rust(&field_def.field_type);
                let field_docs = DocGenerator::generate_doc_comments(&Some(field_def.docs.clone()));
                
                quote! {
                    #field_docs
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
                let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());
                let default_value = Self::generate_field_default_from_typedef_field_type(&field.field_type);
                quote! { #field_name: #default_value, }
            });
            quote! { #(#default_values)* }
        } else if let Some(allocated_fields) = self.idl.get_account_allocated_fields(&account.name) {
            // 优先级2：从字段分配获取字段
            let default_values = allocated_fields.iter().map(|field_def| {
                let field_name = syn::Ident::new(&field_def.name, proc_macro2::Span::call_site());
                let default_value = Self::generate_field_default_from_field_definition_type(&field_def.field_type);
                quote! { #field_name: #default_value, }
            });
            quote! { #(#default_values)* }
        } else {
            quote! {} // 优先级3：无字段，空默认实现
        };

        let account_doc_comment = format!("Account: {} (NonAnchor)", account_name_str);
        
        // 检查IDL是否有types字段
        let has_types_module = self.idl.types.as_ref().map_or(false, |types| !types.is_empty());
        
        // 生成导入语句
        let imports = if has_types_module {
            quote! {
                use crate::types::*;
                use borsh::{BorshDeserialize, BorshSerialize};
                use solana_pubkey::Pubkey;
            }
        } else {
            quote! {
                use borsh::{BorshDeserialize, BorshSerialize};
                use solana_pubkey::Pubkey;
            }
        };
        
        quote! {
            #![doc = #account_doc_comment]
            #doc_comments
            
            #imports
            
            // Constants
            pub const #len_const: usize = std::mem::size_of::<#struct_name>();
            
            // Account Structure
            #doc_comments
            #[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
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
                pub const LEN: usize = std::mem::size_of::<Self>();
                
                pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
                    borsh::to_vec(self)
                }
                
                pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
                    if data.len() != #len_const {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "Account data length mismatch. Expected: {}, got: {}",
                                #len_const, data.len()
                            ),
                        ));
                    }
                    
                    borsh::from_slice(data).map_err(|e| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
                    })
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
        
        // 生成统一的账户枚举和解析函数
        let account_enum = self.generate_account_enum();
        let length_based_unpack = self.generate_length_based_unpack();
        
        quote! {
            //! Non-Anchor accounts module
            //! Generated account definitions with length-based identification
            //! Each account is defined in its own file
            
            use borsh::{BorshDeserialize, BorshSerialize};
            
            #(#module_declarations)*
            
            // Re-export all account items
            #(#re_exports)*
            
            // Unified account types
            #account_enum
            
            // Unified account parser
            #length_based_unpack
        }
    }
}