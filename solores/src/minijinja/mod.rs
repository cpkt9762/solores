//! MiniJinja 模板系统
//! 
//! Solores 项目的现代模板生成系统，支持 Anchor 和 NonAnchor IDL 格式

pub mod generator;
pub mod builders;
pub mod generators;
pub mod filters;
pub mod context;
pub mod utils;

// 主要导出
pub use generator::MinijinjaTemplateGenerator;