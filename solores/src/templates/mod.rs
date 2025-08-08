//! Solores 模板系统
//!
//! 为 Anchor 和非 Anchor 合约提供统一的代码生成模板系统
//! 支持 Instructions、Accounts、Events、Types、Parsers 及其测试的模板化生成

use proc_macro2::TokenStream;

pub mod common;
pub mod anchor;
pub mod non_anchor;
pub mod factory;
pub mod template_adapter;
pub mod boxed_template_adapter;
pub mod unified_interface;
pub mod field_analyzer;
// pub mod data_adapters; // 已删除 - 使用极简二元架构

// Re-export factory for convenient access
pub use factory::TemplateFactory;
pub use unified_interface::{UnifiedCodegen, UnifiedCodegenFactory, CodegenContext, ModuleType, GenerationStrategy};
pub use field_analyzer::{FieldAllocationAnalyzer, FieldAllocationMap, FieldDefinition};
// 数据适配器已移除 - 现在使用极简二元架构
pub use boxed_template_adapter::BoxedTemplateAdapter;

/// Contract mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractMode {
    /// Anchor contracts with 8-byte discriminators
    Anchor,
    /// Non-Anchor contracts with 1-byte discriminators or length-based identification
    NonAnchor,
}

/// Core trait for contract mode specific template behavior
pub trait ContractModeTemplate {
    /// The discriminator type for this contract mode
    type DiscriminatorType;
    
    /// Get the size of discriminator for this contract mode
    fn discriminator_size() -> usize;
    
    /// Generate discriminator parsing code
    fn parse_discriminator_code() -> TokenStream;
    
    /// Generate discriminator constants
    fn generate_constants(&self) -> TokenStream;
    
    /// Generate tests for this contract mode
    fn generate_tests(&self) -> TokenStream;
}

/// Base template generator trait - Pure Multi-file Architecture
pub trait TemplateGenerator {
    /// Get the standard module name for this template
    /// This determines the directory name or file name for generated code
    fn get_standard_module_name(&self) -> &'static str;
    
    /// Generate multiple files for this template
    /// Returns Vec<(file_name, content)> where file_name is relative to module directory
    fn gen_files(&self) -> Vec<(String, TokenStream)>;
    
    /// Generate the mod.rs file content for this template
    /// This file declares and re-exports all generated items
    fn gen_mod_file(&self) -> TokenStream;
    
    /// Check if this template generates files directly in src/ root directory
    /// Returns true for special cases like errors.rs
    fn is_single_root_file(&self) -> bool {
        false
    }
}

/// Types template generator trait - extends TemplateGenerator
pub trait TypesTemplateGenerator: TemplateGenerator {
    /// Generate type structure definitions
    fn generate_type_structs(&self) -> TokenStream;
    
    /// Generate type-related constants
    fn generate_type_constants(&self) -> TokenStream;
}

/// Parsers template generator trait - extends TemplateGenerator
pub trait ParsersTemplateGenerator: TemplateGenerator {
    /// Generate instructions parser
    fn generate_instructions_parser(&self) -> TokenStream;
    
    /// Generate accounts parser
    fn generate_accounts_parser(&self) -> TokenStream;
}

/// Instructions parser test generator trait (Anchor)
pub trait InstructionsParserTestGenerator {
    /// Generate consistency tests for instructions parser
    fn generate_instructions_consistency_tests(&self, instructions: &[crate::idl_format::anchor_idl::AnchorInstruction], program_name: &str) -> TokenStream;
}

/// Instructions parser test generator trait (NonAnchor)
pub trait NonAnchorInstructionsParserTestGenerator {
    /// Generate consistency tests for instructions parser (NonAnchor)
    fn generate_instructions_consistency_tests(&self, instructions: &[crate::idl_format::non_anchor_idl::NonAnchorInstruction], program_name: &str) -> TokenStream;
}

/// Accounts parser test generator trait (Anchor)
pub trait AccountsParserTestGenerator {
    /// Generate consistency tests for accounts parser
    fn generate_accounts_consistency_tests(&self, accounts: &[crate::idl_format::anchor_idl::AnchorAccount], program_name: &str) -> TokenStream;
}

/// Accounts parser test generator trait (NonAnchor)
pub trait NonAnchorAccountsParserTestGenerator {
    /// Generate consistency tests for accounts parser (NonAnchor)
    fn generate_accounts_consistency_tests(&self, accounts: &[crate::idl_format::non_anchor_idl::NonAnchorAccount], program_name: &str) -> TokenStream;
}

/// Events template generator trait
pub trait EventsTemplateGenerator {
    /// Generate event structure definitions
    fn generate_event_structures(&self) -> TokenStream;
    
    /// Generate event wrapper code
    fn generate_event_wrappers(&self) -> TokenStream;
    
    /// Generate event-related constants
    fn generate_event_constants(&self) -> TokenStream;
}