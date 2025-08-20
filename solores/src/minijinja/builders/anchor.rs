//! Anchor IDL 值构建器
//! 
//! 负责将 Anchor IDL 数据结构转换为模板可用的 Value 对象

use crate::idl_format::{IdlFormatEnum, anchor_idl::*};
use convert_case::{Case, Casing};
use minijinja::{context, Value};
use log;
use super::super::utils;

/// 构建账户Value，确保字段信息完整并修复命名问题
pub fn build_account_value(account: &AnchorAccount, idl_enum: &IdlFormatEnum) -> Value {
    // 首先尝试使用账户自己的字段
    let mut fields: Vec<Value> = if let Some(ref fields_vec) = account.fields {
        log::debug!("  └─ Account {} 有 {} 个直接字段", account.name, fields_vec.len());
        fields_vec.iter().map(|field| {
            build_field_value(field)
        }).collect()
    } else {
        Vec::new()
    };

    // 如果账户没有字段，尝试从当前IDL的types中查找同名类型的字段
    if fields.is_empty() {
        if let Some(matching_fields) = utils::find_fields_from_types(&account.name, idl_enum) {
            log::debug!("  └─ Account {} 从types获取 {} 个字段", account.name, matching_fields.len());
            fields = matching_fields;
        } else {
            log::debug!("  └─ Account {} 无可用字段", account.name);
        }
    }

    // 计算packed_size
    let packed_size = utils::calculate_anchor_account_packed_size(account, idl_enum);
    log::debug!("🎯 Account {} 计算得到 PACKED_LEN: {} 字节", account.name, packed_size);

    context! {
        name => account.name.to_case(Case::Pascal),  // 确保PascalCase
        discriminator => account.discriminator,
        fields => fields,
        packed_size => packed_size,
        docs => account.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
    }
}

/// 构建指令Value，修复命名和字段问题
pub fn build_instruction_value(instruction: &AnchorInstruction) -> Value {
    let args: Vec<Value> = if let Some(ref args_vec) = instruction.args {
        log::debug!("  └─ Instruction {} 有 {} 个参数", instruction.name, args_vec.len());
        args_vec.iter().map(|field| {
            build_field_value(field)
        }).collect()
    } else {
        log::debug!("  └─ Instruction {} 无参数", instruction.name);
        Vec::new()
    };

    let accounts: Vec<Value> = if let Some(ref accounts_vec) = instruction.accounts {
        accounts_vec.iter().map(|acc| {
            Value::from_serialize(acc)
        }).collect()
    } else {
        Vec::new()
    };

    context! {
        name => instruction.name.to_case(Case::Pascal),  // 修复PascalCase命名
        discriminator => instruction.discriminator,
        args => args.clone(),
        fields => args,  // 模板中使用fields，确保字段数据传递
        accounts => accounts,
        docs => instruction.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
    }
}

/// 构建事件Value，确保字段完整
pub fn build_event_value(event: &AnchorEvent, idl_enum: &IdlFormatEnum) -> Value {
    let mut fields: Vec<Value> = if let Some(ref fields_vec) = event.fields {
        log::debug!("  └─ Event {} 有 {} 个直接字段", event.name, fields_vec.len());
        fields_vec.iter().map(|field| {
            build_field_value(field)
        }).collect()
    } else {
        log::debug!("  └─ Event {} 无直接字段，尝试从types查找", event.name);
        Vec::new()
    };
    
    // 如果事件没有直接字段，尝试从types中查找同名类型的字段
    if fields.is_empty() {
        if let Some(matching_fields) = utils::find_fields_from_types(&event.name, idl_enum) {
            log::debug!("  └─ Event {} 从types获取 {} 个字段", event.name, matching_fields.len());
            fields = matching_fields;
        } else {
            log::debug!("  └─ Event {} 无可用字段", event.name);
        }
    }

    context! {
        name => event.name.to_case(Case::Pascal),  // 确保PascalCase
        discriminator => event.discriminator,
        fields => fields,
        docs => event.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
    }
}

/// 手动构建类型Value
pub fn build_type_value(type_def: &AnchorType) -> Value {
    match &type_def.kind {
        Some(AnchorTypeKind::Struct(fields)) => {
            let fields_values: Vec<Value> = fields.iter().map(|field| {
                build_field_value(field)
            }).collect();
            
            context! {
                name => type_def.name.to_case(Case::Pascal),
                fields => fields_values,
                kind => "struct",
                docs => type_def.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
            }
        },
        Some(AnchorTypeKind::Enum(variants)) => {
            let variants_values: Vec<Value> = variants.iter().map(|variant| {
                let fields: Vec<Value> = if let Some(ref fields_vec) = variant.fields {
                    fields_vec.iter().map(|f| build_field_value(f)).collect()
                } else {
                    Vec::new()
                };
                context! {
                    name => variant.name.clone(),
                    fields => fields,
                    docs => variant.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
                }
            }).collect();
            
            context! {
                name => type_def.name.to_case(Case::Pascal),
                variants => variants_values,  // 枚举使用variants字段
                kind => "enum",
                docs => type_def.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
            }
        },
        Some(AnchorTypeKind::Alias(_)) => {
            context! {
                name => type_def.name.to_case(Case::Pascal),
                kind => "alias",
                docs => type_def.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
            }
        },
        None => {
            context! {
                name => type_def.name.to_case(Case::Pascal),
                kind => "unknown",
                docs => type_def.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
            }
        }
    }
}

/// 手动构建字段Value，包含完整的字段信息
pub fn build_field_value(field: &AnchorField) -> Value {
    // 转换字段类型为Rust类型字符串
    let rust_type = convert_field_type_to_rust(&field.field_type);
    
    context! {
        name => field.name.clone(),
        rust_type => rust_type,
        is_big_array => is_big_array(&field.field_type),
        docs => field.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
    }
}

/// 将AnchorFieldType转换为Rust类型字符串，使用完整路径引用
pub fn convert_field_type_to_rust(field_type: &AnchorFieldType) -> String {
    match field_type {
        AnchorFieldType::Basic(type_name) => {
            // 基础类型转换，使用完整路径
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
        AnchorFieldType::PrimitiveOrPubkey(type_name) => {
            match type_name.as_str() {
                "publicKey" => "solana_pubkey::Pubkey".to_string(),
                "pubkey" => "solana_pubkey::Pubkey".to_string(),
                _ => type_name.clone(),
            }
        },
        AnchorFieldType::Complex { kind, params: _ } => {
            // 复合类型处理，使用完整路径
            match kind.as_str() {
                "option" => "std::option::Option<T>".to_string(),
                "vec" => "std::vec::Vec<T>".to_string(),
                "array" => "[T; N]".to_string(),
                _ => kind.clone(),
            }
        },
        AnchorFieldType::defined(name) => name.clone(),
        AnchorFieldType::array(element_type, size) => {
            let element_rust_type = convert_field_type_to_rust(element_type);
            format!("[{}; {}]", element_rust_type, size)
        },
        AnchorFieldType::vec(element_type) => {
            let element_rust_type = convert_field_type_to_rust(element_type);
            format!("std::vec::Vec<{}>", element_rust_type)
        },
        AnchorFieldType::option(inner_type) => {
            let inner_rust_type = convert_field_type_to_rust(inner_type);
            format!("std::option::Option<{}>", inner_rust_type)
        },
    }
}

/// 检查是否是大数组类型（需要serde_big_array处理）
pub fn is_big_array(field_type: &AnchorFieldType) -> bool {
    match field_type {
        AnchorFieldType::array(_, size) => *size > 32,  // Rust serde默认只支持到32的数组
        _ => false,
    }
}