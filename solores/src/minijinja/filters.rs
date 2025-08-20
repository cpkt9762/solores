//! MiniJinja 过滤器函数
//! 
//! 提供模板系统中使用的各种过滤器，包括命名转换、类型处理、文档格式化等

use convert_case::{Case, Casing};

/// 处理蛇形命名的过滤器
pub fn to_snake_case_filter(value: String) -> String {
    value.to_case(Case::Snake)
}

/// 处理帕斯卡命名的过滤器
pub fn to_pascal_case_filter(value: String) -> String {
    value.to_case(Case::Pascal)
}

/// 处理类型路径的过滤器
pub fn type_path_filter(value: String) -> String {
    // 添加调试输出
    if value.starts_with("SmallVec") {
        eprintln!("🔍 type_path_filter 检测到 SmallVec: {}", value);
    }
    
    // 基础类型和已包含命名空间的类型直接返回
    match value.as_str() {
        // Rust基础类型
        "u8" | "u16" | "u32" | "u64" | "u128" |
        "i8" | "i16" | "i32" | "i64" | "i128" |
        "f32" | "f64" | "bool" | "String" => value,
        _ => {
            // 首先尝试递归处理复杂类型
            if let Some(result) = process_complex_type(&value) {
                result
            } else if value.contains("::") {
                // 已经包含命名空间的简单类型直接返回
                value
            } else {
                // 其他都视为自定义类型，添加crate::types::前缀
                format!("crate::types::{}", value)
            }
        }
    }
}

/// 递归处理复杂类型（Option, Vec, 数组等）
pub fn process_complex_type(value: &str) -> Option<String> {
    // SmallVec类型处理 - 转换为Vec
    if value.starts_with("SmallVec<") {
        if let Some(inner) = extract_smallvec_inner(value) {
            let processed_inner = type_path_filter(inner);
            return Some(format!("Vec<{}>", processed_inner));
        }
    }
    
    // Vec类型处理
    if value.starts_with("Vec<") {
        if let Some(inner) = extract_generic_inner(value, "Vec") {
            let processed_inner = type_path_filter(inner);
            return Some(format!("Vec<{}>", processed_inner));
        }
    }
    if value.starts_with("std::vec::Vec<") {
        if let Some(inner) = extract_generic_inner(value, "std::vec::Vec") {
            let processed_inner = type_path_filter(inner);
            return Some(format!("std::vec::Vec<{}>", processed_inner));
        }
    }
    
    // Option类型处理
    if value.starts_with("Option<") {
        if let Some(inner) = extract_generic_inner(value, "Option") {
            let processed_inner = type_path_filter(inner);
            return Some(format!("Option<{}>", processed_inner));
        }
    }
    if value.starts_with("std::option::Option<") {
        if let Some(inner) = extract_generic_inner(value, "std::option::Option") {
            let processed_inner = type_path_filter(inner);
            return Some(format!("std::option::Option<{}>", processed_inner));
        }
    }
    
    // 数组类型处理
    if value.starts_with("[") && value.contains(";") {
        if let Some((inner_type, size)) = extract_array_parts(value) {
            let processed_inner = type_path_filter(inner_type);
            return Some(format!("[{}; {}]", processed_inner, size));
        }
    }
    
    None
}

/// 提取泛型类型的内部类型
pub fn extract_generic_inner(value: &str, prefix: &str) -> Option<String> {
    if !value.starts_with(prefix) {
        return None;
    }
    
    let start = prefix.len() + 1; // +1 for '<'
    if start >= value.len() {
        return None;
    }
    
    let end = value.rfind('>')?;
    if end <= start {
        return None;
    }
    
    Some(value[start..end].to_string())
}

/// 提取SmallVec的内部类型（忽略尺寸参数，只返回实际类型）
/// SmallVec<u8, Pubkey> -> Pubkey
/// SmallVec<16, CompiledInstruction> -> CompiledInstruction
pub fn extract_smallvec_inner(value: &str) -> Option<String> {
    if !value.starts_with("SmallVec<") {
        return None;
    }
    
    let start = "SmallVec<".len();
    if start >= value.len() {
        return None;
    }
    
    let end = value.rfind('>')?;
    if end <= start {
        return None;
    }
    
    let inner = &value[start..end];
    
    // 查找第一个逗号，之后的部分是实际类型
    if let Some(comma_pos) = inner.find(',') {
        let type_part = inner[comma_pos + 1..].trim();
        Some(type_part.to_string())
    } else {
        // 如果没有逗号，说明格式不对，返回None
        None
    }
}

/// 提取数组类型的组成部分，正确处理嵌套数组
pub fn extract_array_parts(value: &str) -> Option<(String, String)> {
    if !value.starts_with("[") || !value.ends_with("]") {
        return None;
    }
    
    let inner = &value[1..value.len()-1];
    
    // 找到最后一个分号，这样可以正确处理嵌套数组
    // 对于 "[[u64; 8]; 12]"，inner = "[u64; 8]; 12"
    // 我们需要找到最后的分号来分离 "[u64; 8]" 和 "12"
    if let Some(semicolon_pos) = inner.rfind(';') {
        let type_part = inner[..semicolon_pos].trim().to_string();
        let size_part = inner[semicolon_pos+1..].trim().to_string();
        
        // 验证 size_part 是否为纯数字
        if size_part.chars().all(|c| c.is_ascii_digit()) {
            Some((type_part, size_part))
        } else {
            // 如果不是纯数字，说明这不是一个有效的数组尺寸
            None
        }
    } else {
        None
    }
}

/// 处理 Rust 关键字字段名
pub fn rust_field_filter(value: String) -> String {
    match value.as_str() {
        "type" => "r#type".to_string(),
        "async" => "r#async".to_string(),
        "await" => "r#await".to_string(),
        "match" => "r#match".to_string(),
        "impl" => "r#impl".to_string(),
        "trait" => "r#trait".to_string(),
        "struct" => "r#struct".to_string(),
        "enum" => "r#enum".to_string(),
        "fn" => "r#fn".to_string(),
        "let" => "r#let".to_string(),
        "mut" => "r#mut".to_string(),
        "ref" => "r#ref".to_string(),
        "if" => "r#if".to_string(),
        "else" => "r#else".to_string(),
        "loop" => "r#loop".to_string(),
        "while" => "r#while".to_string(),
        "for" => "r#for".to_string(),
        "in" => "r#in".to_string(),
        "break" => "r#break".to_string(),
        "continue" => "r#continue".to_string(),
        "return" => "r#return".to_string(),
        "const" => "r#const".to_string(),
        "static" => "r#static".to_string(),
        "pub" => "r#pub".to_string(),
        "mod" => "r#mod".to_string(),
        "use" => "r#use".to_string(),
        "crate" => "r#crate".to_string(),
        "super" => "r#super".to_string(),
        "self" => "r#self".to_string(),
        "Self" => "r#Self".to_string(),
        "where" => "r#where".to_string(),
        "extern" => "r#extern".to_string(),
        "unsafe" => "r#unsafe".to_string(),
        _ => value,
    }
}

/// 检查字符串是否以指定前缀开始
pub fn starts_with_filter(value: String, prefix: String) -> bool {
    value.starts_with(&prefix)
}

/// 处理多行文档字符串，为每行添加///前缀
pub fn multiline_docs_filter(value: String) -> String {
    if value.is_empty() {
        return String::new();
    }
    
    value
        .lines()
        .map(|line| {
            if line.trim().is_empty() {
                "///".to_string()
            } else {
                format!("/// {}", line.trim())
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

/// 正则表达式替换过滤器 - 简化版本，直接从字段类型中提取数字
pub fn regex_replace_filter(value: String) -> String {
    // 针对我们的特定用例：从 "[u64; 16]" 中提取 "16"
    if let Some(start) = value.find('[') {
        if let Some(end) = value.find(']') {
            if let Some(semicolon) = value[start..end].find(';') {
                let size_part = &value[start + semicolon + 1..end];
                return size_part.trim().to_string();
            }
        }
    }
    value
}