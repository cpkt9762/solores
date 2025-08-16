//! Askama 自定义过滤器
//!
//! 为 Askama 模板提供专用的文本处理过滤器

use askama::Result;

/// 自定义过滤器模块
/// 这些过滤器可以在 .askama 模板文件中使用
pub mod filters {
    use super::*;
    
    /// 转换为snake_case标识符
    /// 使用方式: {{ name|snake_case }}
    pub fn snake_case(s: &str) -> Result<String> {
        use convert_case::{Case, Casing};
        Ok(s.to_case(Case::Snake))
    }
    
    /// 转换为PascalCase标识符
    /// 使用方式: {{ name|pascal_case }}
    pub fn pascal_case(s: &str) -> Result<String> {
        use convert_case::{Case, Casing};
        Ok(s.to_case(Case::Pascal))
    }
    
    /// 转换为Title Case (Pascal Case的别名)
    /// 使用方式: {{ name|title }}
    pub fn title(s: &str) -> Result<String> {
        use convert_case::{Case, Casing};
        Ok(s.to_case(Case::Pascal))
    }
    
    /// 处理Rust关键字冲突
    /// 使用方式: {{ field_name|rust_keywords }}
    pub fn rust_keywords(s: &str) -> Result<String> {
        let result = match s {
            "type" => "r#type",
            "match" => "r#match", 
            "mod" => "r#mod",
            "pub" => "r#pub",
            "use" => "r#use",
            "fn" => "r#fn",
            "struct" => "r#struct",
            "enum" => "r#enum",
            "impl" => "r#impl",
            "trait" => "r#trait",
            "const" => "r#const",
            "static" => "r#static",
            "let" => "r#let",
            "mut" => "r#mut",
            "ref" => "r#ref",
            "move" => "r#move",
            "return" => "r#return",
            "if" => "r#if",
            "else" => "r#else",
            "while" => "r#while",
            "for" => "r#for",
            "loop" => "r#loop",
            "break" => "r#break",
            "continue" => "r#continue",
            _ => s,
        };
        Ok(result.to_string())
    }
    
    /// 转换为SCREAMING_SNAKE_CASE常量名
    /// 使用方式: {{ name|screaming_snake_case }}
    pub fn screaming_snake_case(s: &str) -> Result<String> {
        use convert_case::{Case, Casing};
        Ok(s.to_case(Case::ScreamingSnake))
    }
    
    /// 格式化discriminator数组
    /// 使用方式: {{ discriminator_bytes|format_discriminator }}
    pub fn format_discriminator(bytes: &[u8]) -> Result<String> {
        Ok(format!("{:?}", bytes))
    }
    
    /// 格式化单字节discriminator
    /// 使用方式: {{ discriminator_byte|format_single_discriminator }}
    pub fn format_single_discriminator(byte: u8) -> Result<String> {
        Ok(byte.to_string())
    }
    
    /// 检查是否为Pubkey类型
    /// 使用方式: {% if field.rust_type|is_pubkey_type %}
    pub fn is_pubkey_type(rust_type: &str) -> Result<bool> {
        Ok(matches!(rust_type, "Pubkey" | "solana_program::pubkey::Pubkey"))
    }
    
    /// 检查是否为Option类型
    /// 使用方式: {% if field.rust_type|is_option_type %}
    pub fn is_option_type(rust_type: &str) -> Result<bool> {
        Ok(rust_type.starts_with("Option<"))
    }
    
    /// 检查是否为Vec/Array类型
    /// 使用方式: {% if field.rust_type|is_array_type %}
    pub fn is_array_type(rust_type: &str) -> Result<bool> {
        Ok(rust_type.starts_with("Vec<") || rust_type.contains("[") && rust_type.contains("]"))
    }
}