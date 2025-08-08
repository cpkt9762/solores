//! 通用组件模块
//!
//! 提供模板系统中共享的通用功能，包括文档注释生成、导入管理、属性生成、错误生成、Cargo.toml生成等

pub mod attribute_generator;
pub mod cargo_generator;
pub mod doc_generator;
pub mod error_generator;
pub mod import_manager;
pub mod naming_converter;
pub mod test_utils;

// Re-export commonly used utilities
pub use attribute_generator::AttributeGenerator;
pub use cargo_generator::{CargoTomlGenerator, DependencyProfile};
pub use doc_generator::DocGenerator;
pub use error_generator::{ErrorGenerator, ErrorTemplateWrapper};
pub use import_manager::{ImportManager, ImportType, SolanaImport};
pub use naming_converter::NamingConverter;
pub use test_utils::TestUtils;
