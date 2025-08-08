//! 文档注释生成器
//!
//! 处理从 IDL docs 字段到 Rust 文档注释的转换

use proc_macro2::TokenStream;
use quote::quote;

/// 文档注释生成器
pub struct DocGenerator;

impl DocGenerator {
    /// 从 IDL docs 数组生成 Rust 文档注释的 TokenStream
    ///
    /// # Arguments
    /// * `docs` - IDL 中的文档字符串数组
    ///
    /// # Returns
    /// * 生成的文档注释 TokenStream，如果 docs 为空则返回空 TokenStream
    pub fn generate_doc_comments(docs: &Option<Vec<String>>) -> TokenStream {
        if let Some(docs) = docs {
            let doc_tokens: Vec<TokenStream> = docs
                .iter()
                .filter(|doc| !doc.trim().is_empty())
                .map(|doc| {
                    let doc_str = doc.trim();
                    quote! { #[doc = #doc_str] }
                })
                .collect();
            quote! { #(#doc_tokens)* }
        } else {
            quote! {}
        }
    }
    
    /// 生成指令级别的文档注释
    ///
    /// 用于 Instructions 模块中的指令文档
    pub fn generate_instruction_docs(docs: &Option<Vec<String>>) -> TokenStream {
        log::debug!("📚 开始生成指令文档，docs: {:?}", docs);
        let result = Self::generate_doc_comments(docs);
        log::debug!("📚 生成的指令文档TokenStream: {}", result);
        result
    }
    
    /// 生成账户字段级别的文档注释
    ///
    /// 用于 Instructions 模块中的账户字段文档
    pub fn generate_account_field_docs(docs: &Option<Vec<String>>) -> TokenStream {
        Self::generate_doc_comments(docs)
    }
    
    /// 生成类型级别的文档注释（基础版本）
    ///
    /// 用于 Types 模块中的类型结构体文档
    pub fn generate_type_docs_basic(docs: &Option<Vec<String>>) -> TokenStream {
        Self::generate_doc_comments(docs)
    }
    
    /// 生成事件级别的文档注释
    ///
    /// 用于 Events 模块中的事件结构体文档
    /// 优先使用 types 数组中的文档，回退到 events 数组中的文档
    pub fn generate_event_docs(
        types_docs: &Option<Vec<String>>, 
        events_docs: &Option<Vec<String>>
    ) -> TokenStream {
        // 优先使用 types 数组中的 docs
        if types_docs.is_some() {
            Self::generate_doc_comments(types_docs)
        } else {
            Self::generate_doc_comments(events_docs)
        }
    }
    
    /// 生成字段级别的文档注释
    ///
    /// 通用的字段文档生成方法，用于结构体字段、枚举变体等
    pub fn generate_field_docs(docs: &Option<Vec<String>>) -> TokenStream {
        Self::generate_doc_comments(docs)
    }
    
    /// 生成类型文档（适配AnchorType）
    ///
    /// 从AnchorType结构体生成文档注释
    pub fn generate_type_docs(named_type: &crate::idl_format::anchor_idl::AnchorType) -> TokenStream {
        Self::generate_doc_comments(&named_type.docs)
    }
    
    /// 生成账户级别的文档注释（适配AnchorAccount）
    ///
    /// 从AnchorAccount结构体生成文档注释
    pub fn generate_account_docs(named_account: &crate::idl_format::anchor_idl::AnchorAccount) -> TokenStream {
        Self::generate_doc_comments(&named_account.docs)
    }

    /// 生成NonAnchor类型级别的文档注释
    ///
    /// 从NonAnchorType结构体生成文档注释
    pub fn generate_non_anchor_type_docs(named_type: &crate::idl_format::non_anchor_idl::NonAnchorType) -> TokenStream {
        Self::generate_doc_comments(&named_type.docs)
    }
    
    /// 生成NonAnchor账户级别的文档注释
    ///
    /// 从NonAnchorAccount结构体生成文档注释
    pub fn generate_non_anchor_account_docs(named_account: &crate::idl_format::non_anchor_idl::NonAnchorAccount) -> TokenStream {
        Self::generate_doc_comments(&named_account.docs)
    }

    /// 生成非Anchor事件的文档（适配NonAnchorEvent）
    pub fn generate_non_anchor_event_docs(named_event: &crate::idl_format::non_anchor_idl::NonAnchorEvent) -> TokenStream {
        Self::generate_doc_comments(&named_event.docs)
    }
    
    /// 生成指令账户字段的文档注释（适配IxAccount）
    ///
    /// 从IxAccount结构体生成文档注释
    pub fn generate_instruction_account_docs(ix_account: &crate::idl_format::anchor_idl::IxAccount) -> TokenStream {
        Self::generate_doc_comments(&ix_account.docs)
    }
    
    /// 生成错误级别的文档注释
    ///
    /// 用于 Errors 模块中的错误变体文档
    pub fn generate_error_docs(docs: &[String]) -> TokenStream {
        if docs.is_empty() {
            return quote! {};
        }
        
        let doc_tokens: Vec<TokenStream> = docs
            .iter()
            .filter(|doc| !doc.trim().is_empty())
            .map(|doc| {
                let doc_str = doc.trim();
                quote! { #[doc = #doc_str] }
            })
            .collect();
        quote! { #(#doc_tokens)* }
    }
}