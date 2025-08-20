//! 值构建器模块
//! 
//! 处理 Anchor 和 NonAnchor IDL 数据到 MiniJinja 值的转换

pub mod anchor;
pub mod non_anchor;

// 重新导出主要功能
pub use anchor::*;
pub use non_anchor::*;