//! 工具函数模块
//! 
//! 提供字段查找、大小计算等实用工具函数

use crate::idl_format::{IdlFormatEnum, anchor_idl::*, non_anchor_idl::*};
use minijinja::Value;
use log;
use super::builders::anchor;

/// 从types中查找同名类型的字段（解决IDL中账户定义缺少字段的问题）
pub fn find_fields_from_types(account_name: &str, idl_enum: &IdlFormatEnum) -> Option<Vec<Value>> {
    // 获取当前IDL的types数据
    match idl_enum {
        IdlFormatEnum::Anchor(anchor_idl) => {
            if let Some(ref types) = anchor_idl.types {
                for type_def in types {
                    if type_def.name == account_name {
                        // 找到同名类型，提取字段
                        if let Some(AnchorTypeKind::Struct(fields)) = &type_def.kind {
                            return Some(fields.iter().map(|field| {
                                anchor::build_field_value(field)
                            }).collect());
                        }
                    }
                }
            }
        },
        IdlFormatEnum::NonAnchor(_non_anchor_idl) => {
            // NonAnchor也可能有类似问题，暂时返回空
            // TODO: 如果需要，可以添加NonAnchor的处理逻辑
        }
    }
    None
}

/// 计算Anchor账户的PACKED_LEN大小
pub fn calculate_anchor_account_packed_size(account: &AnchorAccount, idl_enum: &IdlFormatEnum) -> usize {
    let mut size = 8; // Anchor 账户总是有 8 字节 discriminator
    
    log::debug!("🧮 计算账户 {} 的大小，开始大小: {} (discriminator)", account.name, size);
    
    // 统一字段获取逻辑：优先使用直接字段，否则从字段分配获取
    if let Some(fields) = &account.fields {
        log::debug!("  🎯 使用直接字段 ({} 个)", fields.len());
        for field in fields {
            let field_size = calculate_anchor_field_size(&field.field_type);
            log::debug!("  📐 字段 {} ({:?}): {} 字节", field.name, field.field_type, field_size);
            size += field_size;
        }
    } else {
        log::debug!("  🔍 账户无直接字段，尝试从types中查找");
        // 尝试从当前IDL的types中查找同名类型的字段
        if find_fields_from_types(&account.name, idl_enum).is_some() {
            log::debug!("  🎯 从types获取字段");
            // 这里需要计算实际的字段大小，而不是Value类型
            // 我们需要访问IDL数据来获取实际的字段类型信息
            size += calculate_fields_size_from_types(&account.name, idl_enum);
        } else {
            log::debug!("  ❌ 无法获取字段分配，只有 discriminator");
        }
    }
    
    log::debug!("🏁 账户 {} 总大小: {} 字节", account.name, size);
    size
}

/// 从types中计算字段大小
pub fn calculate_fields_size_from_types(type_name: &str, idl_enum: &IdlFormatEnum) -> usize {
    if let IdlFormatEnum::Anchor(anchor_idl) = idl_enum {
        if let Some(types) = &anchor_idl.types {
            if let Some(type_def) = types.iter().find(|t| t.name == type_name) {
                if let Some(kind) = &type_def.kind {
                    match kind {
                        AnchorTypeKind::Struct(fields) => {
                            let mut total_size = 0;
                            for field in fields {
                                let field_size = calculate_anchor_field_size(&field.field_type);
                                log::debug!("    📐 类型字段 {} ({:?}): {} 字节", field.name, field.field_type, field_size);
                                total_size += field_size;
                            }
                            log::debug!("  🧮 类型 {} 总字段大小: {} 字节", type_name, total_size);
                            return total_size;
                        },
                        _ => {
                            log::debug!("  🤔 类型 {} 不是结构体，估算为32字节", type_name);
                            return 32;
                        }
                    }
                }
            }
        }
    }
    log::debug!("  🤔 未找到类型 {} 定义，估算为32字节", type_name);
    32
}

/// 计算Anchor字段大小
pub fn calculate_anchor_field_size(field_type: &AnchorFieldType) -> usize {
    match field_type {
        AnchorFieldType::Basic(type_name) => {
            match type_name.as_str() {
                "bool" => 1,
                "u8" | "i8" => 1,
                "u16" | "i16" => 2,
                "u32" | "i32" | "f32" => 4,
                "u64" | "i64" | "f64" => 8,
                "u128" | "i128" => 16,
                "pubkey" | "Pubkey" | "publicKey" => 32,
                "string" => 4, // String 在 Borsh 中是长度前缀(4字节) + 内容（变长）
                _ => {
                    log::debug!("  🤔 未知基础类型 '{}' 默认为8字节", type_name);
                    8
                }
            }
        },
        AnchorFieldType::array(inner_type, size) => {
            let inner_size = calculate_anchor_field_size(inner_type);
            inner_size * size
        },
        AnchorFieldType::vec(_inner_type) => {
            // Vec 在 Borsh 中是长度前缀(4字节) + 元素（变长）
            // 对于PACKED_LEN，我们假设空vec，只计算长度前缀
            4
        },
        AnchorFieldType::option(inner_type) => {
            // Option 在 Borsh 中是 1字节标识 + 可能的值
            let inner_size = calculate_anchor_field_size(inner_type);
            1 + inner_size
        },
        AnchorFieldType::defined(type_name) => {
            // 自定义类型，尝试查找定义或使用默认估算
            calculate_defined_type_size(type_name)
        },
        AnchorFieldType::PrimitiveOrPubkey(type_name) => {
            // 处理 PrimitiveOrPubkey 类型
            match type_name.as_str() {
                "bool" => 1,
                "u8" | "i8" => 1,
                "u16" | "i16" => 2,
                "u32" | "i32" | "f32" => 4,
                "u64" | "i64" | "f64" => 8,
                "u128" | "i128" => 16,
                "pubkey" | "Pubkey" | "publicKey" => 32,
                "string" => 4,
                _ => {
                    log::debug!("  🤔 未知PrimitiveOrPubkey类型 '{}' 默认为8字节", type_name);
                    8
                }
            }
        },
        AnchorFieldType::Complex { kind, params: _ } => {
            // 处理复合类型
            match kind.as_str() {
                "array" => {
                    // 对于复合数组，估算为64字节
                    64
                },
                "vec" => 4, // Vec长度前缀
                "option" => 9, // 1字节标识 + 8字节默认值
                _ => {
                    log::debug!("  🤔 未知复合类型 '{}' 默认为16字节", kind);
                    16
                }
            }
        }
    }
}

/// 计算自定义类型大小
pub fn calculate_defined_type_size(type_name: &str) -> usize {
    // 特殊类型的已知大小
    match type_name {
        "VestingSchedule" => {
            // VestingSchedule特殊类型的估算大小
            // 假设包含：start_time(8) + cliff_time(8) + end_time(8) + total_amount(8) + released_amount(8) = 40字节
            40
        },
        _ => {
            // 其他自定义类型默认估算为32字节
            log::debug!("  🤔 未知自定义类型 '{}' 默认为32字节", type_name);
            32
        }
    }
}

/// 计算NonAnchor账户的PACKED_LEN大小  
pub fn calculate_non_anchor_account_packed_size(account: &NonAnchorAccount) -> usize {
    let mut size = 0; // NonAnchor 没有 discriminator
    
    log::debug!("🧮 计算NonAnchor账户 {} 的大小，开始大小: {}", account.name, size);
    
    if let Some(fields) = &account.fields {
        log::debug!("  🎯 使用直接字段 ({} 个)", fields.len());
        for field in fields {
            let field_size = calculate_non_anchor_field_size(&field.field_type);
            log::debug!("  📐 字段 {} ({:?}): {} 字节", field.name, field.field_type, field_size);
            size += field_size;
        }
    } else {
        log::debug!("  🔍 账户无直接字段");
    }
    
    log::debug!("🏁 NonAnchor账户 {} 总大小: {} 字节", account.name, size);
    size
}

/// 计算NonAnchor字段大小
pub fn calculate_non_anchor_field_size(field_type: &NonAnchorFieldType) -> usize {
    match field_type {
        NonAnchorFieldType::Basic(type_name) => {
            match type_name.as_str() {
                "bool" => 1,
                "u8" | "i8" => 1,
                "u16" | "i16" => 2,
                "u32" | "i32" | "f32" => 4,
                "u64" | "i64" | "f64" => 8,
                "u128" | "i128" => 16,
                "pubkey" | "Pubkey" | "publicKey" => 32,
                "string" => 4, // String 长度前缀
                _ => {
                    log::debug!("  🤔 未知NonAnchor基础类型 '{}' 默认为8字节", type_name);
                    8
                }
            }
        },
        NonAnchorFieldType::Array { array } => {
            let (inner_type, size) = array;
            let inner_size = calculate_non_anchor_field_size(inner_type);
            inner_size * size
        },
        NonAnchorFieldType::Vec { vec: _ } => {
            // Vec 长度前缀
            4
        },
        NonAnchorFieldType::Option { option } => {
            // Option 标识 + 值
            let inner_size = calculate_non_anchor_field_size(option);
            1 + inner_size
        },
        NonAnchorFieldType::Defined { defined } => {
            // 自定义类型
            calculate_defined_type_size(defined)
        },
        NonAnchorFieldType::HashMap { key: _, value: _ } => {
            // HashMap: 长度前缀 + key/value对 (变长，估算)
            8 // 保守估算
        },
        NonAnchorFieldType::Complex { kind, params: _ } => {
            // 处理复合类型
            match kind.as_str() {
                "array" => 64, // 估算数组大小
                "vec" => 4,    // Vec长度前缀
                "option" => 9, // 1字节标识 + 8字节默认值
                _ => {
                    log::debug!("  🤔 未知NonAnchor复合类型 '{}' 默认为16字节", kind);
                    16
                }
            }
        },
    }
}