//! IDL Traits - 共享 trait 定义和派生宏库
//!
//! 为 IDL 生成的解析器提供统一的 trait 接口和自动派生宏。
//! 
//! # 核心功能
//! 
//! - **Parser Traits** - 统一的解析器接口定义
//! - **ProgramParser** - 扩展的程序解析器 (支持事件解析)
//! - **Derive Macros** - 零配置自动实现
//! 
//! # 使用示例
//! 
//! ```rust
//! use idl_traits::*;
//! 
//! // 指令解析器 (自动支持事件解析扩展)
//! #[derive(InstructionParser)]
//! pub struct MyInstructionParser;
//! 
//! // 账户解析器
//! #[derive(AccountParser)]
//! pub struct MyAccountParser;
//! ```

// 重新导出核心模块
pub mod traits;
pub mod types;

// 导出派生宏
pub use idl_traits_derive::{InstructionParser, AccountParser};

// 便利重导出
pub use traits::*;
pub use types::*;

// 删除了不需要的类型别名，直接使用 types 模块的导出