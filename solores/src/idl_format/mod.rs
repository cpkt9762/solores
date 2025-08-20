//! IDL格式处理
//!
//! 极简二元架构：只支持 Anchor 和 NonAnchor 两种IDL格式
//! 
//! - AnchorIdl：8字节discriminator的Anchor合约格式
//! - NonAnchorIdl：1字节discriminator或其他识别方式的合约格式

use proc_macro2::TokenStream;
use toml::{map::Map, Value};

pub mod anchor_idl;
pub mod non_anchor_idl;

// Re-export for convenient access
pub use anchor_idl::AnchorIdl;
pub use non_anchor_idl::NonAnchorIdl;

/// Legacy系统兼容的代码生成模块接口
/// TODO: 逐步迁移到新模板系统后可以移除
pub trait IdlCodegenModule {
    /// 模块文件名，例如 "errors"
    fn name(&self) -> &str;

    /// 生成模块文件的头部内容（通常是import语句）
    fn gen_head(&self) -> TokenStream;

    /// 生成模块文件的主体内容
    fn gen_body(&self) -> TokenStream;

    /// 检查此模块是否生成多个文件
    fn has_multiple_files(&self) -> bool {
        false
    }

    /// 生成多个文件（文件名，内容）对
    /// 仅在has_multiple_files()返回true时调用
    fn gen_files(&self) -> Vec<(String, TokenStream)> {
        vec![]
    }

    /// 为多文件模块生成mod.rs文件内容
    fn gen_mod_file(&self) -> TokenStream {
        TokenStream::new()
    }
    
    /// 支持类型转换，用于访问具体实现的特殊方法
    fn as_any(&self) -> &dyn std::any::Any;
}

/// IDL格式的统一接口
pub trait IdlFormat {
    /// 获取程序名称
    fn program_name(&self) -> &str;

    /// 获取程序版本
    fn program_version(&self) -> &str;

    /// 获取程序地址
    fn program_address(&self) -> Option<&str>;

    /// 检查是否为正确的IDL格式
    fn is_correct_idl_format(&self) -> bool;

    /// 获取依赖项配置
    fn dependencies(&self, args: &crate::Args) -> Map<String, Value>;

    /// 生成代码模块
    fn modules<'me>(&'me self, args: &'me crate::Args) -> Vec<Box<dyn IdlCodegenModule + 'me>>;

    /// 检查是否为Anchor合约
    fn is_anchor_contract(&self) -> bool;
    
    /// 检查是否有错误定义
    fn has_errors(&self) -> bool;
}

/// 自动检测IDL格式并解析
pub fn parse_idl_json(json_str: &str) -> Result<IdlFormatEnum, serde_json::Error> {
    // 首先尝试解析为通用JSON值进行格式检测
    let json_value: serde_json::Value = serde_json::from_str(json_str)?;
    
    // 检测是否为Anchor格式
    let is_anchor = is_anchor_format(&json_value);
    log::debug!("IDL格式检测结果: is_anchor = {}", is_anchor);
    
    if is_anchor {
        log::debug!("尝试解析为Anchor格式");
        let anchor_idl = AnchorIdl::parse_json(json_str)?;
        log::debug!("✓ 成功解析为Anchor IDL: {}", anchor_idl.program_name());
        Ok(IdlFormatEnum::Anchor(anchor_idl))
    } else {
        log::debug!("尝试解析为NonAnchor格式");
        let non_anchor_idl = NonAnchorIdl::parse_json(json_str)?;
        log::debug!("✓ 成功解析为NonAnchor IDL: {}", non_anchor_idl.program_name());
        Ok(IdlFormatEnum::NonAnchor(non_anchor_idl))
    }
}

/// 检测是否为Anchor格式的IDL
fn is_anchor_format(json: &serde_json::Value) -> bool {
    // 检查Anchor特有的字段和结构
    if let Some(obj) = json.as_object() {
        // 优先检查metadata.spec是否明确标识为非Anchor格式
        if let Some(metadata) = obj.get("metadata") {
            if let Some(spec) = metadata.get("spec").and_then(|s| s.as_str()) {
                match spec {
                    "anchor" => return true,
                    "shank" => return false,  // shank生成的是NonAnchor格式
                    _ => {} // 继续其他检测
                }
            }
            // 检查origin字段
            if let Some(origin) = metadata.get("origin").and_then(|s| s.as_str()) {
                match origin {
                    "anchor" => return true,
                    "shank" => return false,  // shank生成的是NonAnchor格式
                    _ => {} // 继续其他检测
                }
            }
        }
        
        // 检查顶级是否有8字节discriminator数组（真正的Anchor特征）
        if let Some(discriminator) = obj.get("discriminator") {
            if let Some(arr) = discriminator.as_array() {
                if arr.len() == 8 {
                    return true;
                }
            }
        }
        
        // 检查指令是否有8字节discriminator数组（注意是discriminator不是discriminant）
        if let Some(instructions) = obj.get("instructions").and_then(|i| i.as_array()) {
            for instruction in instructions {
                if let Some(discriminator) = instruction.get("discriminator") {
                    // Anchor discriminator是8字节数组
                    if let Some(arr) = discriminator.as_array() {
                        if arr.len() == 8 {
                            return true;
                        }
                    }
                }
            }
        }
        
        // 检查账户是否有8字节discriminator数组
        if let Some(accounts) = obj.get("accounts").and_then(|a| a.as_array()) {
            for account in accounts {
                if let Some(discriminator) = account.get("discriminator") {
                    if let Some(arr) = discriminator.as_array() {
                        if arr.len() == 8 {
                            return true;
                        }
                    }
                }
            }
        }
    }
    
    false
}

/// IDL格式枚举，用于统一处理
#[derive(Debug, Clone)]
pub enum IdlFormatEnum {
    /// Anchor格式IDL
    Anchor(AnchorIdl),
    /// 非Anchor格式IDL
    NonAnchor(NonAnchorIdl),
}

impl IdlFormat for IdlFormatEnum {
    fn program_name(&self) -> &str {
        match self {
            IdlFormatEnum::Anchor(idl) => idl.program_name(),
            IdlFormatEnum::NonAnchor(idl) => idl.program_name(),
        }
    }

    fn program_version(&self) -> &str {
        match self {
            IdlFormatEnum::Anchor(idl) => idl.program_version(),
            IdlFormatEnum::NonAnchor(idl) => idl.program_version(),
        }
    }

    fn program_address(&self) -> Option<&str> {
        match self {
            IdlFormatEnum::Anchor(idl) => Some(&idl.address),
            IdlFormatEnum::NonAnchor(idl) => Some(&idl.address),
        }
    }

    fn is_correct_idl_format(&self) -> bool {
        // 二元架构中，任何成功解析的格式都是正确的
        true
    }

    fn dependencies(&self, _args: &crate::Args) -> Map<String, Value> {
        // 传统模板系统的依赖方法已移除，现在使用 cargo 模块处理依赖
        Map::new()
    }

    fn modules<'me>(&'me self, _args: &'me crate::Args) -> Vec<Box<dyn IdlCodegenModule + 'me>> {
        // 传统模板系统的模块方法已移除，现在使用 MiniJinja 系统
        // 返回空向量，write_readme.rs 会通过扫描生成的文件来确定模块结构
        Vec::new()
    }

    fn is_anchor_contract(&self) -> bool {
        match self {
            IdlFormatEnum::Anchor(_) => true,
            IdlFormatEnum::NonAnchor(_) => false,
        }
    }
    
    fn has_errors(&self) -> bool {
        match self {
            IdlFormatEnum::Anchor(idl) => idl.errors.is_some(),
            IdlFormatEnum::NonAnchor(idl) => idl.errors.is_some(),
        }
    }
}