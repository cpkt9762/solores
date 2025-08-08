//! Non-Anchor Errors 模板
//!
//! 为非 Anchor 合约生成 Errors 相关代码

use proc_macro2::TokenStream;
use quote::quote;

use crate::idl_format::non_anchor_idl::NonAnchorError;
use crate::templates::{TemplateGenerator, ContractMode};
use crate::templates::common::ErrorGenerator;

/// Non-Anchor Errors 模板
pub struct NonAnchorErrorsTemplate<'a> {
    error_generator: ErrorGenerator<'a>,
}

impl<'a> NonAnchorErrorsTemplate<'a> {
    /// 创建新的 Non-Anchor Errors 模板
    pub fn new(program_name: &'a str, errors: &'a [NonAnchorError]) -> Self {
        // 转换NonAnchorError为AnchorError以兼容ErrorGenerator
        let anchor_errors: Vec<crate::idl_format::anchor_idl::AnchorError> = errors.iter().map(|e| {
            crate::idl_format::anchor_idl::AnchorError {
                name: e.name.clone(),
                code: e.code,
                msg: Some(e.msg.clone()),
                docs: e.docs.clone(),
            }
        }).collect();
        
        // 使用Box::leak创建静态引用
        let anchor_errors_ref: &'static [crate::idl_format::anchor_idl::AnchorError] = 
            Box::leak(anchor_errors.into_boxed_slice());
        
        Self {
            error_generator: ErrorGenerator::new(program_name, anchor_errors_ref, ContractMode::NonAnchor),
        }
    }
}

impl<'a> TemplateGenerator for NonAnchorErrorsTemplate<'a> {
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
        quote! {}
    }

    fn is_single_root_file(&self) -> bool {
        // errors.rs 直接生成到src/根目录下
        true
    }
}