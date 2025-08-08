//! 通用错误生成器
//!
//! 统一处理所有IDL格式的错误代码生成，消除重复代码

use proc_macro2::TokenStream;
use quote::quote;
use convert_case::{Case, Casing};

use crate::idl_format::anchor_idl::AnchorError;
use crate::templates::{ContractMode, TemplateGenerator};
use crate::templates::common::doc_generator::DocGenerator;
use crate::templates::common::import_manager::ImportManager;

/// 通用错误生成器
/// 
/// 为不同类型的合约（Anchor/NonAnchor）统一生成错误相关代码
pub struct ErrorGenerator<'a> {
    /// 程序名称
    pub program_name: &'a str,
    /// 错误变体列表
    pub errors: &'a [AnchorError],
    /// 合约模式
    pub contract_mode: ContractMode,
}

impl<'a> ErrorGenerator<'a> {
    /// 创建新的错误生成器
    pub fn new(
        program_name: &'a str, 
        errors: &'a [AnchorError],
        contract_mode: ContractMode,
    ) -> Self {
        Self {
            program_name,
            errors,
            contract_mode,
        }
    }

    /// 生成错误枚举定义
    pub fn generate_error_enum(&self) -> TokenStream {
        if self.errors.is_empty() {
            return quote! {};
        }

        let error_enum_name = syn::Ident::new(
            &format!("{}Error", self.program_name.to_case(Case::Pascal)),
            proc_macro2::Span::call_site(),
        );

        let error_variants = self.errors.iter().map(|error| {
            let variant_name = syn::Ident::new(
                &error.name.to_case(Case::Pascal), 
                proc_macro2::Span::call_site()
            );
            let error_code = error.code;
            
            // 处理错误消息和文档注释
            let (error_message, doc_comments) = self.generate_error_variant_docs(error);

            // 将 u32 转换为 isize，这是 Rust 枚举 discriminant 的标准类型
            let error_code_isize = error_code as isize;
            
            quote! {
                #doc_comments
                #[error(#error_message)]
                #variant_name = #error_code_isize,
            }
        });

        let enum_doc = self.generate_enum_docs();
        let enum_attributes = self.generate_enum_attributes();

        quote! {
            #enum_doc
            #enum_attributes
            pub enum #error_enum_name {
                #(#error_variants)*
            }
        }
    }

    /// 生成到 ProgramError 的转换实现
    pub fn generate_program_error_conversion(&self) -> TokenStream {
        // 如果有自定义错误，添加转换实现
        if !self.errors.is_empty() {
            let error_enum_name = syn::Ident::new(
                &format!("{}Error", self.program_name.to_case(Case::Pascal)),
                proc_macro2::Span::call_site(),
            );
            
            quote! {
                impl From<#error_enum_name> for solana_program_error::ProgramError {
                    fn from(e: #error_enum_name) -> Self {
                        solana_program_error::ProgramError::Custom(e as u32)
                    }
                }
            }
        } else {
            quote! {}
        }
    }

    /// 生成错误相关的导入语句
    pub fn generate_error_imports(&self) -> TokenStream {
        ImportManager::generate_error_imports()
    }

    /// 生成完整的错误模块代码
    pub fn generate_error_module(&self) -> TokenStream {
        let imports = self.generate_error_imports();
        let error_enum = self.generate_error_enum();
        let conversion = self.generate_program_error_conversion();
        
        let module_doc = self.generate_module_docs();

        quote! {
            #module_doc
            
            #imports
            
            #error_enum
            
            #conversion
        }
    }

    /// 生成错误变体的文档注释
    fn generate_error_variant_docs(&self, error: &AnchorError) -> (String, TokenStream) {
        let error_message = error.msg.clone().unwrap_or_else(|| format!("Error code {}", error.code));

        let doc_comments = if error.docs.is_some() {
            DocGenerator::generate_doc_comments(&error.docs)
        } else {
            let doc_str = format!("{} - {}", error.code, &error_message);
            quote! { #[doc = #doc_str] }
        };

        (error_message, doc_comments)
    }

    /// 生成枚举的文档注释
    fn generate_enum_docs(&self) -> TokenStream {
        let enum_doc = format!("Custom errors for the {} program", self.program_name);
        quote! { #[doc = #enum_doc] }
    }

    /// 生成枚举的属性
    fn generate_enum_attributes(&self) -> TokenStream {
        match self.contract_mode {
            ContractMode::Anchor => {
                quote! {
                    #[derive(Clone, Copy, Debug, Eq, thiserror::Error, num_derive::FromPrimitive, PartialEq)]
                }
            },
            ContractMode::NonAnchor => {
                quote! {
                    #[derive(Clone, Copy, Debug, Eq, thiserror::Error, num_derive::FromPrimitive, PartialEq)]
                }
            }
        }
    }

    /// 生成模块级别的文档注释
    fn generate_module_docs(&self) -> TokenStream {
        let contract_type = match self.contract_mode {
            ContractMode::Anchor => "Anchor",
            ContractMode::NonAnchor => "Non-Anchor",
        };
        
        quote! {
            #![doc = concat!(#contract_type, " error definitions module")]
            #![doc = "Generated error enums and conversion implementations"]
        }
    }
}

/// 错误模板包装器
/// 
/// 实现 TemplateGenerator trait，使错误生成器能够在模板系统中使用
pub struct ErrorTemplateWrapper<'a> {
    generator: ErrorGenerator<'a>,
}

impl<'a> ErrorTemplateWrapper<'a> {
    /// 创建新的错误模板包装器
    pub fn new(
        program_name: &'a str,
        errors: &'a [AnchorError],
        contract_mode: ContractMode,
    ) -> Self {
        Self {
            generator: ErrorGenerator::new(program_name, errors, contract_mode),
        }
    }
}

impl<'a> TemplateGenerator for ErrorTemplateWrapper<'a> {
    fn get_standard_module_name(&self) -> &'static str {
        "errors"
    }

    fn gen_files(&self) -> Vec<(String, TokenStream)> {
        vec![
            ("errors.rs".to_string(), self.generator.generate_error_module())
        ]
    }

    fn gen_mod_file(&self) -> TokenStream {
        // errors.rs 作为根目录单文件，不需要mod.rs
        quote! {}
    }

    fn is_single_root_file(&self) -> bool {
        // errors.rs 直接生成到src/根目录下
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_errors_generation() {
        let generator = ErrorGenerator::new("test_program", &[], ContractMode::Anchor);
        let result = generator.generate_error_enum();
        
        // 空的错误列表应该生成空的 TokenStream
        assert!(result.is_empty());
    }

    #[test]
    fn test_error_enum_name_generation() {
        let errors = vec![];
        let generator = ErrorGenerator::new("my_program", &errors, ContractMode::Anchor);
        
        // 测试程序名称转换为合适的枚举名称
        let result = generator.generate_error_enum();
        let result_str = result.to_string();
        
        // 由于是空数组，应该生成空内容
        assert!(result_str.is_empty());
    }
}