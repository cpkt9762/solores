//! Anchor Types 模板
//!
//! 为 Anchor 合约生成 Types 相关代码，使用 8 字节 discriminator

use proc_macro2::TokenStream;
use quote::quote;
use convert_case::{Case, Casing};
use heck::ToShoutySnakeCase;
use sha2::{Digest, Sha256};

use crate::idl_format::anchor_idl::AnchorIdl;
use crate::Args;
use crate::templates::{TemplateGenerator, TypesTemplateGenerator};
use crate::templates::common::{doc_generator::DocGenerator};
use crate::templates::field_analyzer::{FieldAllocationAnalyzer, FieldAllocationMap};

/// Anchor Types 模板
pub struct AnchorTypesTemplate<'a> {
    pub idl: &'a AnchorIdl,
    pub args: &'a Args,
    pub field_allocation: FieldAllocationMap,
}

impl<'a> AnchorTypesTemplate<'a> {
    /// 创建新的 Anchor Types 模板
    pub fn new(idl: &'a AnchorIdl, args: &'a Args) -> Self {
        // 分析字段分配，排除被其他模块使用的类型
        log::debug!("🏭 创建Anchor Types模板，开始字段分配分析");
        let field_allocation = FieldAllocationAnalyzer::analyze_anchor_idl(idl);
        log::debug!("✅ Anchor Types模板字段分配分析完成");
        Self { idl, args, field_allocation }
    }

    /// 生成类型常量 - 只为剩余的（未被其他模块使用的）类型生成
    pub fn generate_type_constants(&self) -> TokenStream {
        let types = self.idl.types.as_deref().unwrap_or(&[]);
        if types.is_empty() {
            return quote! {};
        }

        // 为所有类型生成常量（types模块应该包含所有定义的类型）
        let constants = types.iter().filter_map(|r#type| {
            // Check if this is an enum that needs discriminator constants
            if let Some(type_kind) = &r#type.kind {
                if matches!(type_kind, crate::idl_format::anchor_idl::AnchorTypeKind::Enum(_)) {
                    let const_name = syn::Ident::new(
                        &format!("{}_TYPE_DISCM", r#type.name.to_shouty_snake_case()),
                        proc_macro2::Span::call_site(),
                    );
                    
                    // Generate SHA-256 based discriminator for Anchor types
                    let name_bytes = format!("type:{}", r#type.name).as_bytes().to_vec();
                    let hash = Sha256::digest(&name_bytes);
                    let first_8_bytes = &hash[..8];
                    let bytes = first_8_bytes.iter().map(|&b| b).collect::<Vec<_>>();
                    
                    Some(quote! {
                        pub const #const_name: [u8; 8] = [#(#bytes),*];
                    })
                } else {
                    None
                }
            } else {
                None
            }
        });

        quote! {
            #(#constants)*
        }
    }
}

impl<'a> TypesTemplateGenerator for AnchorTypesTemplate<'a> {
    fn generate_type_structs(&self) -> TokenStream {
        let types = self.idl.types.as_deref().unwrap_or(&[]);
        if types.is_empty() {
            return quote! {};
        }

        // 只为字段分配分析中剩余的类型生成结构体，排除已被其他模块实现的类型
        log::debug!("🏭 AnchorTypesTemplate: 开始生成类型结构体");
        log::debug!("   - 总类型数: {}", types.len());
        log::debug!("   - 剩余类型: {:?}", self.field_allocation.types_remaining_fields.keys().collect::<Vec<_>>());
        log::debug!("   - 被实现的类型: {:?}", self.field_allocation.implemented_types);
        
        let r#typeinitions = types.iter().filter_map(|r#type| {
            // 检查该类型是否应该保留在 types 模块中
            if self.field_allocation.types_remaining_fields.contains_key(&r#type.name) {
                log::debug!("   ✅ 生成类型: {}", r#type.name);
                Some(r#type)
            } else if self.field_allocation.implemented_types.contains(&r#type.name) {
                log::debug!("   🗑️ 跳过被实现的类型: {} (在events/accounts中有直接实现)", r#type.name);
                None
            } else {
                log::debug!("   ⚠️ 保留未分类的类型: {} (可能是枚举或别名)", r#type.name);
                Some(r#type)
            }
        }).map(|r#type| {
            let type_name = syn::Ident::new(&r#type.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let doc_comments = DocGenerator::generate_type_docs(r#type);
            
            if let Some(type_kind) = &r#type.kind {
                match type_kind {
                    crate::idl_format::anchor_idl::AnchorTypeKind::Struct(struct_def) => {
                        let struct_fields = struct_def.iter().map(|field| {
                            let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());
                            let field_type = self.convert_idl_type_to_rust(&field.field_type);
                            let field_docs = DocGenerator::generate_field_docs(&field.docs);
                            
                            quote! {
                                #field_docs
                                pub #field_name: #field_type,
                            }
                        });

                        quote! {
                            #doc_comments
                            #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, Eq, Default)]
                            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                            pub struct #type_name {
                                #(#struct_fields)*
                            }
                        }
                    },
                    crate::idl_format::anchor_idl::AnchorTypeKind::Enum(enum_def) => {
                        let enum_variants = enum_def.iter().map(|variant| {
                            let variant_name = syn::Ident::new(&variant.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
                            let variant_docs = quote! {}; // EnumVariant doesn't have docs field
                            
                            if let Some(fields) = &variant.fields {
                                // Variant with fields - 直接处理AnchorField列表
                                let variant_fields = fields.iter().map(|field| {
                                    let field_type = self.convert_idl_type_to_rust(&field.field_type);
                                    quote! { #field_type }
                                });
                                
                                quote! {
                                    #variant_docs
                                    #variant_name(#(#variant_fields),*),
                                }
                            } else {
                                // Simple variant
                                quote! {
                                    #variant_docs
                                    #variant_name,
                                }
                            }
                        });

                        // Generate Default implementation for enum (first variant)
                        let first_variant = enum_def.first().map(|v| {
                            let variant_name = syn::Ident::new(&v.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
                            quote! {
                                impl Default for #type_name {
                                    fn default() -> Self {
                                        Self::#variant_name
                                    }
                                }
                            }
                        }).unwrap_or_else(|| quote! {});

                        quote! {
                            #doc_comments
                            #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, Eq)]
                            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                            pub enum #type_name {
                                #(#enum_variants)*
                            }
                            #first_variant
                        }
                    },
                    crate::idl_format::anchor_idl::AnchorTypeKind::Alias(alias_def) => {
                        // For type aliases, just generate a type alias declaration
                        let alias_type = self.convert_idl_type_to_rust(alias_def);
                        
                        quote! {
                            #doc_comments
                            pub type #type_name = #alias_type;
                        }
                    },
                }
            } else {
                // Fallback for types without definition
                quote! {
                    #doc_comments
                    #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, Eq, Default)]
                    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                    pub struct #type_name;
                }
            }
        });

        quote! {
            #(#r#typeinitions)*
        }
    }

    fn generate_type_constants(&self) -> TokenStream {
        self.generate_type_constants()
    }
}

impl<'a> AnchorTypesTemplate<'a> {    
    /// 转换 IDL 类型为 Rust 类型
    fn convert_idl_type_to_rust(&self, idl_type: &crate::idl_format::anchor_idl::AnchorFieldType) -> TokenStream {
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
                    "publicKey" | "Pubkey" | "pubkey" => quote! { Pubkey },
                    "string" | "String" => quote! { String },
                    _ => {
                        let type_name = syn::Ident::new(s, proc_macro2::Span::call_site());
                        quote! { #type_name }
                    }
                }
            },
            crate::idl_format::anchor_idl::AnchorFieldType::defined(type_name) => {
                let type_name = syn::Ident::new(type_name, proc_macro2::Span::call_site());
                quote! { #type_name }
            },
            crate::idl_format::anchor_idl::AnchorFieldType::array(inner_type, size) => {
                let inner_type = self.convert_idl_type_to_rust(inner_type);
                let size_literal = proc_macro2::Literal::usize_unsuffixed(*size);
                quote! { [#inner_type; #size_literal] }
            },
            crate::idl_format::anchor_idl::AnchorFieldType::vec(inner_type) => {
                let inner_type = self.convert_idl_type_to_rust(inner_type);
                quote! { Vec<#inner_type> }
            },
            crate::idl_format::anchor_idl::AnchorFieldType::option(inner_type) => {
                let inner_type = self.convert_idl_type_to_rust(inner_type);
                quote! { Option<#inner_type> }
            },
            crate::idl_format::anchor_idl::AnchorFieldType::PrimitiveOrPubkey(s) => {
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
                    "publicKey" | "Pubkey" | "pubkey" => quote! { Pubkey },
                    "string" | "String" => quote! { String },
                    _ => {
                        let type_name = syn::Ident::new(s, proc_macro2::Span::call_site());
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

    /// 为单个type生成完整的文件内容
    pub fn generate_single_type_file(&self, type_def: &crate::idl_format::anchor_idl::AnchorType) -> TokenStream {
        let type_name = syn::Ident::new(&type_def.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
        let doc_comments = DocGenerator::generate_doc_comments(&type_def.docs);
        let type_name_str = &type_def.name;
        
        let type_definition = if let Some(type_kind) = &type_def.kind {
            match type_kind {
            crate::idl_format::anchor_idl::AnchorTypeKind::Struct(struct_def) => {
                let fields = struct_def.iter().map(|field| {
                    let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());
                    let field_type = self.convert_idl_type_to_rust(&field.field_type);
                    let field_docs = DocGenerator::generate_field_docs(&field.docs);
                    
                    quote! {
                        #field_docs
                        pub #field_name: #field_type,
                    }
                });

                // Generate Default implementation for struct
                let default_assignments = struct_def.iter().map(|field| {
                    let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());
                    let default_value = self.generate_default_value_for_field_type(&field.field_type);
                    quote! { #field_name: #default_value, }
                });

                quote! {
                    #doc_comments
                    #[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
                    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                    pub struct #type_name {
                        #(#fields)*
                    }
                    
                    impl Default for #type_name {
                        fn default() -> Self {
                            Self {
                                #(#default_assignments)*
                            }
                        }
                    }
                }
            },
            
            crate::idl_format::anchor_idl::AnchorTypeKind::Enum(enum_def) => {
                let variants = enum_def.iter().map(|variant| {
                    let variant_name = syn::Ident::new(&variant.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
                    let variant_docs = quote! {}; // EnumVariant doesn't have docs field
                    
                    if let Some(fields) = &variant.fields {
                        let variant_fields = fields.iter().map(|field| {
                            let field_type = self.convert_idl_type_to_rust(&field.field_type);
                            quote! { #field_type }
                        });
                        quote! {
                            #variant_docs
                            #variant_name(#(#variant_fields),*),
                        }
                    } else {
                        quote! {
                            #variant_docs
                            #variant_name,
                        }
                    }
                });

                // Generate Default implementation for enum (use first variant)
                let default_impl = if let Some(first_variant) = enum_def.first() {
                    let variant_name = syn::Ident::new(&first_variant.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
                    if first_variant.fields.is_some() {
                        // First variant has fields - generate default values for all fields
                        let variant_fields = first_variant.fields.as_ref().unwrap().iter().map(|_field| {
                            quote! { Default::default() }
                        });
                        quote! {
                            impl Default for #type_name {
                                fn default() -> Self {
                                    Self::#variant_name(#(#variant_fields),*)
                                }
                            }
                        }
                    } else {
                        // First variant is simple
                        quote! {
                            impl Default for #type_name {
                                fn default() -> Self {
                                    Self::#variant_name
                                }
                            }
                        }
                    }
                } else {
                    quote! {}
                };

                quote! {
                    #doc_comments
                    #[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
                    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                    pub enum #type_name {
                        #(#variants)*
                    }
                    #default_impl
                }
            },
            
            crate::idl_format::anchor_idl::AnchorTypeKind::Alias(field_type) => {
                let rust_type = self.convert_idl_type_to_rust(field_type);
                
                quote! {
                    #doc_comments
                    pub type #type_name = #rust_type;
                }
            }
            }
        } else {
            // Fallback for types without definition
            quote! {
                #doc_comments
                #[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                pub struct #type_name;
                
                impl Default for #type_name {
                    fn default() -> Self {
                        Self
                    }
                }
            }
        };

        quote! {
            #![doc = #type_name_str]
            #doc_comments
            
            use borsh::{BorshDeserialize, BorshSerialize};
            use solana_pubkey::Pubkey;
            use crate::*;
            
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

    /// 为字段类型生成合适的默认值
    fn generate_default_value_for_field_type(&self, field_type: &crate::idl_format::anchor_idl::AnchorFieldType) -> TokenStream {
        match field_type {
            crate::idl_format::anchor_idl::AnchorFieldType::array(inner_type, size) => {
                // 对于数组，需要检查大小来决定默认值生成方式
                if *size <= 32 {
                    // 小数组可以使用Default::default()
                    quote! { Default::default() }
                } else {
                    // 大数组需要手动初始化
                    match inner_type.as_ref() {
                        crate::idl_format::anchor_idl::AnchorFieldType::Basic(basic_type) => {
                            match basic_type.as_str() {
                                "u8" => {
                                    let size_lit = proc_macro2::Literal::usize_unsuffixed(*size);
                                    quote! { [0u8; #size_lit] }
                                },
                                "u64" => {
                                    let size_lit = proc_macro2::Literal::usize_unsuffixed(*size);
                                    quote! { [0u64; #size_lit] }
                                },
                                _ => quote! { Default::default() }
                            }
                        },
                        _ => quote! { Default::default() }
                    }
                }
            },
            _ => quote! { Default::default() }
        }
    }
}

impl<'a> TemplateGenerator for AnchorTypesTemplate<'a> {
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
        
        // 智能过滤：只为剩余的类型生成文件（排除被实现的类型，保留被引用的类型）
        let mut types_to_generate: Vec<&crate::idl_format::anchor_idl::AnchorType> = Vec::new();
        
        for type_def in types.iter() {
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
            types_to_generate.push(type_def);
        }
        
        if types_to_generate.is_empty() {
            return vec![("mod.rs".to_string(), quote! {
                //! Empty types module - all types implemented in other modules
            })];
        }
        
        let mut files = Vec::new();
        
        // 为过滤后的类型生成独立文件
        for type_def in types_to_generate.iter() {
            let file_name = format!("{}.rs", type_def.name.to_case(Case::Snake));
            let file_content = self.generate_single_type_file(type_def);
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
        let types_to_declare: Vec<&crate::idl_format::anchor_idl::AnchorType> = types.iter()
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
            //! Anchor types module
            //! Generated type definitions for the Anchor contract
            //! Each type is defined in its own file
            
            #(#module_declarations)*
            
            // Re-export all type items
            #(#re_exports)*
        }
    }
}