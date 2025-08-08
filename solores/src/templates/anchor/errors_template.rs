//! Anchor Errors 模板
//!
//! 为 Anchor 合约生成 Errors 相关代码

use proc_macro2::TokenStream;

use crate::idl_format::anchor_idl::AnchorError;
use crate::templates::{TemplateGenerator, ContractMode};
use crate::templates::common::ErrorGenerator;

/// Anchor Errors 模板
pub struct AnchorErrorsTemplate<'a> {
    error_generator: ErrorGenerator<'a>,
}

impl<'a> AnchorErrorsTemplate<'a> {
    /// 创建新的 Anchor Errors 模板
    pub fn new(program_name: &'a str, errors: &'a [AnchorError]) -> Self {
        Self {
            error_generator: ErrorGenerator::new(program_name, errors, ContractMode::Anchor),
        }
    }
}

impl<'a> TemplateGenerator for AnchorErrorsTemplate<'a> {
    fn get_standard_module_name(&self) -> &'static str {
        "errors"
    }

    fn gen_files(&self) -> Vec<(String, TokenStream)> {
        vec![
            ("errors.rs".to_string(), self.error_generator.generate_error_module())
        ]
    }

    fn gen_mod_file(&self) -> TokenStream {
        // errors.rs 作为根目录单文件，不需要mod.rs
        quote::quote! {}
    }

    fn is_single_root_file(&self) -> bool {
        // errors.rs 直接生成到src/根目录下
        true
    }
}