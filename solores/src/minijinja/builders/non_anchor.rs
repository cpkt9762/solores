//! NonAnchor IDL 值构建器
//! 
//! 负责将 NonAnchor IDL 数据结构转换为模板可用的 Value 对象

use crate::idl_format::non_anchor_idl::*;
use convert_case::{Case, Casing};
use minijinja::{context, Value};
use log;
use super::super::utils;

/// NonAnchor账户构建方法 - 完整实现
pub fn build_non_anchor_account_value(account: &NonAnchorAccount) -> Value {
    let fields: Vec<Value> = if let Some(ref fields_vec) = account.fields {
        fields_vec.iter().map(|field| {
            build_non_anchor_field_value(field)
        }).collect()
    } else {
        Vec::new()
    };

    // 计算packed_size
    let packed_size = utils::calculate_non_anchor_account_packed_size(account);
    log::debug!("🎯 NonAnchor Account {} 计算得到 PACKED_LEN: {} 字节", account.name, packed_size);

    context! {
        name => account.name.to_case(Case::Pascal),
        fields => fields,
        discriminator => account.discriminator.as_ref().unwrap_or(&Vec::new()),
        packed_size => packed_size,
        docs => account.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
    }
}

/// NonAnchor指令构建方法 - 完整实现
pub fn build_non_anchor_instruction_value(instruction: &NonAnchorInstruction, index: usize) -> Value {
    let args: Vec<Value> = instruction.args.as_ref().unwrap_or(&Vec::new()).iter().map(|field| {
        build_non_anchor_field_value(field)
    }).collect();

    let accounts: Vec<Value> = instruction.accounts.as_ref().unwrap_or(&Vec::new()).iter().map(|acc| {
        Value::from_serialize(acc)
    }).collect();

    // 确保discriminator存在，如果没有则用索引
    let discriminator_value = if let Some(ref disc) = instruction.discriminator {
        if !disc.is_empty() {
            disc.clone()
        } else {
            vec![index as u8]  // 使用索引作为discriminator
        }
    } else {
        vec![index as u8]  // 使用索引作为discriminator
    };

    context! {
        name => instruction.name.to_case(Case::Pascal),
        discriminator => discriminator_value,
        args => args.clone(),
        fields => args,
        accounts => accounts,
        docs => instruction.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
    }
}

/// NonAnchor事件构建方法 - 完整实现
pub fn build_non_anchor_event_value(event: &NonAnchorEvent) -> Value {
    let fields: Vec<Value> = if let Some(ref fields_vec) = event.fields {
        fields_vec.iter().map(|field| {
            build_non_anchor_field_value(field)
        }).collect()
    } else {
        Vec::new()
    };

    context! {
        name => event.name.to_case(Case::Pascal),
        discriminator => event.discriminator.as_ref().unwrap_or(&Vec::new()),
        fields => fields,
        docs => event.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
    }
}

/// NonAnchor类型构建方法 - 完整实现
pub fn build_non_anchor_type_value(type_def: &NonAnchorType) -> Value {
    match &type_def.type_def {
        NonAnchorTypeKind::Struct { fields } => {
            let fields_values: Vec<Value> = fields.iter().map(|field| {
                build_non_anchor_field_value(field)
            }).collect();
            
            context! {
                name => type_def.name.to_case(Case::Pascal),
                fields => fields_values,
                kind => "struct",
                docs => type_def.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
            }
        },
        NonAnchorTypeKind::Enum { variants } => {
            let variants_values: Vec<Value> = variants.iter().map(|variant| {
                let fields: Vec<Value> = variant.fields.as_ref().unwrap_or(&Vec::new()).iter().map(|f| {
                    build_non_anchor_field_value(f)
                }).collect();
                context! {
                    name => variant.name.clone(),
                    fields => fields,
                    docs => variant.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
                }
            }).collect();
            
            context! {
                name => type_def.name.to_case(Case::Pascal),
                variants => variants_values,
                kind => "enum",
                docs => type_def.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
            }
        },
        NonAnchorTypeKind::Alias { value: _ } => {
            context! {
                name => type_def.name.to_case(Case::Pascal),
                kind => "alias",
                docs => type_def.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
            }
        }
    }
}

/// 构建NonAnchor字段Value
pub fn build_non_anchor_field_value(field: &NonAnchorField) -> Value {
    let rust_type = convert_non_anchor_field_type_to_rust(&field.field_type);
    
    context! {
        name => field.name.to_case(Case::Snake),
        rust_type => rust_type,
        is_pubkey => is_non_anchor_field_pubkey(&field.field_type),
        is_big_array => is_non_anchor_big_array(&field.field_type),
        docs => field.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
    }
}

/// 将NonAnchor字段类型转换为Rust类型字符串
pub fn convert_non_anchor_field_type_to_rust(field_type: &NonAnchorFieldType) -> String {
    match field_type {
        NonAnchorFieldType::Basic(type_name) => {
            match type_name.as_str() {
                "u8" => "u8".to_string(),
                "i8" => "i8".to_string(),
                "u16" => "u16".to_string(),
                "i16" => "i16".to_string(),
                "u32" => "u32".to_string(),
                "i32" => "i32".to_string(),
                "u64" => "u64".to_string(),
                "i64" => "i64".to_string(),
                "u128" => "u128".to_string(),
                "i128" => "i128".to_string(),
                "bool" => "bool".to_string(),
                "f32" => "f32".to_string(),
                "f64" => "f64".to_string(),
                "string" => "std::string::String".to_string(),
                "publicKey" => "solana_pubkey::Pubkey".to_string(),
                "pubkey" => "solana_pubkey::Pubkey".to_string(),
                "bytes" => "std::vec::Vec<u8>".to_string(),
                _ => type_name.clone(),
            }
        },
        NonAnchorFieldType::Option { option } => {
            let inner_rust_type = convert_non_anchor_field_type_to_rust(option);
            format!("std::option::Option<{}>", inner_rust_type)
        },
        NonAnchorFieldType::Vec { vec } => {
            let element_rust_type = convert_non_anchor_field_type_to_rust(vec);
            format!("std::vec::Vec<{}>", element_rust_type)
        },
        NonAnchorFieldType::Array { array } => {
            let (element_type, size) = array;
            let element_rust_type = convert_non_anchor_field_type_to_rust(element_type);
            format!("[{}; {}]", element_rust_type, size)
        },
        NonAnchorFieldType::Defined { defined } => {
            // 检查是否是基础数组类型语法，避免为内置数组类型添加前缀
            if (defined.starts_with('[') && defined.ends_with(']')) ||
               defined.starts_with("std::") ||
               defined.starts_with("solana_") ||
               // 检查是否是基础类型
               matches!(defined.as_str(), "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | 
                                          "u64" | "i64" | "u128" | "i128" | "bool" | 
                                          "f32" | "f64" | "string") ||
               // 检查是否包含数组语法模式，如 "[u64; 8]"
               (defined.contains('[') && defined.contains(';') && defined.contains(']')) {
                // 这是内置类型或数组，不需要添加前缀
                defined.clone()
            } else {
                // 这是真正的用户定义类型，需要添加前缀
                format!("crate::types::{}", defined)
            }
        },
        NonAnchorFieldType::HashMap { key, value } => {
            let key_rust_type = convert_non_anchor_field_type_to_rust(key);
            let value_rust_type = convert_non_anchor_field_type_to_rust(value);
            format!("std::collections::HashMap<{}, {}>", key_rust_type, value_rust_type)
        },
        NonAnchorFieldType::Complex { kind, params: _ } => {
            match kind.as_str() {
                "option" => "std::option::Option<T>".to_string(),
                "vec" => "std::vec::Vec<T>".to_string(),
                "array" => "[T; N]".to_string(),
                _ => kind.clone(),
            }
        },
    }
}

/// 检查NonAnchor字段是否为Pubkey类型
pub fn is_non_anchor_field_pubkey(field_type: &NonAnchorFieldType) -> bool {
    match field_type {
        NonAnchorFieldType::Basic(type_name) => {
            matches!(type_name.as_str(), "publicKey" | "pubkey")
        },
        _ => false,
    }
}

/// 检查NonAnchor字段是否为大数组类型
pub fn is_non_anchor_big_array(field_type: &NonAnchorFieldType) -> bool {
    match field_type {
        NonAnchorFieldType::Array { array } => {
            let (_, size) = array;
            *size > 32
        },
        _ => false,
    }
}