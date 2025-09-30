//! 工具函数模块
//! 
//! 提供字段查找、大小计算等实用工具函数

use crate::idl_format::{IdlFormatEnum, anchor_idl::*, non_anchor_idl::*};
use minijinja::Value;
use log;
use super::builders::anchor;
use std::collections::HashMap;

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
    let mut cache: HashMap<String, usize> = HashMap::new();
    
    log::debug!("🧮 计算账户 {} 的大小，开始大小: {} (discriminator)", account.name, size);
    
    // 统一字段获取逻辑：优先使用直接字段，否则从字段分配获取
    if let Some(fields) = &account.fields {
        log::debug!("  🎯 使用直接字段 ({} 个)", fields.len());
        for field in fields {
            let field_size = calculate_anchor_field_size_recursive(&field.field_type, idl_enum, &mut cache);
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
            size += calculate_fields_size_from_types(&account.name, idl_enum, &mut cache);
        } else {
            log::debug!("  ❌ 无法获取字段分配，只有 discriminator");
        }
    }
    
    log::debug!("🏁 账户 {} 总大小: {} 字节", account.name, size);
    size
}

/// 从types中计算字段大小（递归版本）
pub fn calculate_fields_size_from_types(
    type_name: &str, 
    idl_enum: &IdlFormatEnum, 
    cache: &mut HashMap<String, usize>
) -> usize {
    // 直接使用递归函数
    calculate_defined_type_size_recursive(type_name, idl_enum, cache)
}


/// 计算自定义类型大小（递归查找IDL中的类型定义）
pub fn calculate_defined_type_size_recursive(
    type_name: &str, 
    idl_enum: &crate::idl_format::IdlFormatEnum,
    cache: &mut HashMap<String, usize>
) -> usize {
    // 检查缓存避免重复计算
    if let Some(&cached_size) = cache.get(type_name) {
        log::debug!("  📋 从缓存获取类型 '{}' 大小: {} 字节", type_name, cached_size);
        return cached_size;
    }

    // 特殊类型的已知大小（保留向后兼容）
    let size = match type_name {
        "VestingSchedule" => {
            // VestingSchedule特殊类型的估算大小
            40
        },
        _ => {
            // 尝试从IDL中查找类型定义
            if let Some(calculated_size) = lookup_type_size_from_idl(type_name, idl_enum, cache) {
                calculated_size
            } else {
                // 如果找不到定义，使用默认估算
                log::debug!("  🤔 未知自定义类型 '{}' 默认为32字节", type_name);
                32
            }
        }
    };

    // 缓存结果
    cache.insert(type_name.to_string(), size);
    log::debug!("  ✅ 计算并缓存类型 '{}' 大小: {} 字节", type_name, size);
    size
}

/// 从IDL中查找类型定义并计算大小
pub fn lookup_type_size_from_idl(
    type_name: &str,
    idl_enum: &crate::idl_format::IdlFormatEnum,
    cache: &mut HashMap<String, usize>
) -> Option<usize> {
    match idl_enum {
        crate::idl_format::IdlFormatEnum::Anchor(anchor_idl) => {
            if let Some(types) = &anchor_idl.types {
                if let Some(type_def) = types.iter().find(|t| t.name == type_name) {
                    if let Some(kind) = &type_def.kind {
                        return calculate_anchor_kind_size(kind, idl_enum, cache);
                    }
                }
            }
        },
        crate::idl_format::IdlFormatEnum::NonAnchor(non_anchor_idl) => {
            if let Some(types) = &non_anchor_idl.types {
                if let Some(type_def) = types.iter().find(|t| t.name == type_name) {
                    return calculate_non_anchor_kind_size(&type_def.type_def, idl_enum, cache);
                }
            }
        }
    }
    None
}

/// 计算Anchor类型大小
pub fn calculate_anchor_kind_size(
    anchor_kind: &AnchorTypeKind,
    idl_enum: &crate::idl_format::IdlFormatEnum,
    cache: &mut HashMap<String, usize>
) -> Option<usize> {
    match anchor_kind {
        AnchorTypeKind::Struct(fields) => {
            let mut total_size = 0;
            for field in fields {
                total_size += calculate_anchor_field_size_recursive(&field.field_type, idl_enum, cache);
            }
            Some(total_size)
        },
        AnchorTypeKind::Enum(_variants) => {
            // 枚举类型：1字节discriminator + 最大variant大小
            // 简化处理，使用默认大小
            Some(32)
        },
        AnchorTypeKind::Alias(field_type) => {
            // 类型别名：计算实际类型大小
            Some(calculate_anchor_field_size_recursive(field_type, idl_enum, cache))
        }
    }
}

/// 计算NonAnchor类型大小
pub fn calculate_non_anchor_kind_size(
    non_anchor_kind: &NonAnchorTypeKind,
    idl_enum: &crate::idl_format::IdlFormatEnum,
    cache: &mut HashMap<String, usize>
) -> Option<usize> {
    match non_anchor_kind {
        NonAnchorTypeKind::Struct { fields } => {
            let mut total_size = 0;
            for field in fields {
                total_size += calculate_non_anchor_field_size_recursive(&field.field_type, idl_enum, cache);
            }
            Some(total_size)
        },
        NonAnchorTypeKind::Enum { variants: _ } => {
            // 枚举类型：使用默认大小
            Some(32)
        },
        NonAnchorTypeKind::Alias { value } => {
            // 类型别名：计算实际类型大小
            Some(calculate_non_anchor_field_size_recursive(value, idl_enum, cache))
        }
    }
}

/// 递归计算Anchor字段大小
pub fn calculate_anchor_field_size_recursive(
    field_type: &AnchorFieldType,
    idl_enum: &crate::idl_format::IdlFormatEnum,
    cache: &mut HashMap<String, usize>
) -> usize {
    match field_type {
        AnchorFieldType::Basic(type_name) => {
            calculate_basic_type_size(type_name)
        },
        AnchorFieldType::array(inner_type, size) => {
            let inner_size = calculate_anchor_field_size_recursive(inner_type, idl_enum, cache);
            inner_size * size
        },
        AnchorFieldType::vec(_inner_type) => {
            4 // Vec长度前缀
        },
        AnchorFieldType::option(inner_type) => {
            let inner_size = calculate_anchor_field_size_recursive(inner_type, idl_enum, cache);
            1 + inner_size
        },
        AnchorFieldType::defined(type_name) => {
            calculate_defined_type_size_recursive(type_name, idl_enum, cache)
        },
        AnchorFieldType::PrimitiveOrPubkey(type_name) => {
            calculate_basic_type_size(type_name)
        },
        AnchorFieldType::Complex { kind, params } => {
            // 处理复合类型，正确解析params参数
            calculate_complex_type_size(kind, params.as_ref(), idl_enum, cache)
        },
    }
}

/// 递归计算NonAnchor字段大小
pub fn calculate_non_anchor_field_size_recursive(
    field_type: &NonAnchorFieldType,
    idl_enum: &crate::idl_format::IdlFormatEnum,
    cache: &mut HashMap<String, usize>
) -> usize {
    match field_type {
        NonAnchorFieldType::Basic(type_name) => {
            calculate_basic_type_size(type_name)
        },
        NonAnchorFieldType::Array { array } => {
            let (inner_type, size) = array;
            let inner_size = calculate_non_anchor_field_size_recursive(inner_type, idl_enum, cache);
            inner_size * size
        },
        NonAnchorFieldType::Vec { vec: _inner_type } => {
            4 // Vec长度前缀
        },
        NonAnchorFieldType::Option { option } => {
            let inner_size = calculate_non_anchor_field_size_recursive(option, idl_enum, cache);
            1 + inner_size
        },
        NonAnchorFieldType::Defined { defined } => {
            calculate_defined_type_size_recursive(defined, idl_enum, cache)
        },
        NonAnchorFieldType::HashMap { key: _, value: _ } => {
            64 // HashMap估算大小
        },
        NonAnchorFieldType::Complex { kind, params } => {
            // 处理复合类型，正确解析params参数
            calculate_complex_type_size(kind, params.as_ref(), idl_enum, cache)
        },
    }
}

/// 计算复合类型大小（统一处理Anchor和NonAnchor）
pub fn calculate_complex_type_size(
    kind: &str,
    params: Option<&Vec<serde_json::Value>>,
    idl_enum: &crate::idl_format::IdlFormatEnum,
    cache: &mut HashMap<String, usize>
) -> usize {
    match kind {
        "array" => {
            // 数组类型：[元素类型, 大小]
            if let Some(params_vec) = params {
                if params_vec.len() >= 2 {
                    // 解析数组元素类型和大小
                    let element_type_json = &params_vec[0];
                    let array_size_json = &params_vec[1];
                    
                    // 解析数组大小
                    let array_size = if let Some(size_num) = array_size_json.as_u64() {
                        size_num as usize
                    } else {
                        log::debug!("  🤔 无法解析数组大小，使用默认值1");
                        1
                    };
                    
                    // 解析元素类型并计算大小
                    let element_size = parse_and_calculate_element_type(element_type_json, idl_enum, cache);
                    let total_size = element_size * array_size;
                    
                    log::debug!("  🔍 复合array: 元素大小{}字节 × 数组大小{} = {}字节", element_size, array_size, total_size);
                    total_size
                } else {
                    log::debug!("  🤔 array参数不足，默认为64字节");
                    64
                }
            } else {
                log::debug!("  🤔 array缺少params，默认为64字节");
                64
            }
        },
        "vec" => {
            // Vec类型：长度前缀 + 可变元素
            4
        },
        "option" => {
            // Option类型：标识字节 + 可能的值
            if let Some(params_vec) = params {
                if !params_vec.is_empty() {
                    let element_size = parse_and_calculate_element_type(&params_vec[0], idl_enum, cache);
                    1 + element_size
                } else {
                    9 // 默认：1字节标识 + 8字节默认值
                }
            } else {
                9
            }
        },
        _ => {
            log::debug!("  🤔 未知复合类型 '{}' 默认为16字节", kind);
            16
        }
    }
}

/// 解析并计算元素类型大小
pub fn parse_and_calculate_element_type(
    element_json: &serde_json::Value,
    idl_enum: &crate::idl_format::IdlFormatEnum,
    cache: &mut HashMap<String, usize>
) -> usize {
    match element_json {
        serde_json::Value::String(type_str) => {
            // 简单类型字符串
            calculate_basic_type_size(type_str)
        },
        serde_json::Value::Object(obj) => {
            // 复杂类型对象，如 {"defined": "TypeName"} 或 {"array": [...]}
            if let Some(defined_name) = obj.get("defined").and_then(|v| v.as_str()) {
                calculate_defined_type_size_recursive(defined_name, idl_enum, cache)
            } else if let Some(array_data) = obj.get("array") {
                if let Some(array_vec) = array_data.as_array() {
                    calculate_complex_type_size("array", Some(array_vec), idl_enum, cache)
                } else {
                    log::debug!("  🤔 array数据格式错误，默认为32字节");
                    32
                }
            } else {
                log::debug!("  🤔 未知对象类型，默认为32字节");
                32
            }
        },
        _ => {
            log::debug!("  🤔 未知JSON类型，默认为32字节");
            32
        }
    }
}

/// 计算基础类型大小的通用函数（统一处理所有IDL类型）
pub fn calculate_basic_type_size(type_name: &str) -> usize {
    match type_name {
        // 布尔和8位类型
        "bool" => 1,
        "u8" | "i8" => 1,
        
        // 16位类型
        "u16" | "i16" => 2,
        
        // 32位类型
        "u32" | "i32" | "f32" => 4,
        
        // 64位类型
        "u64" | "i64" | "f64" => 8,
        
        // 128位类型
        "u128" | "i128" => 16,
        
        // 公钥类型（各种变体）
        "pubkey" | "Pubkey" | "publicKey" => 32,
        
        // 字符串类型
        "string" => 4, // Borsh序列化：长度前缀(4字节) + 内容(变长)
        
        // 其他未知类型
        _ => {
            log::debug!("  🤔 未知基础类型 '{}' 使用保守估算32字节", type_name);
            32
        }
    }
}

/// 计算自定义类型大小（兼容函数，保持API）
pub fn calculate_defined_type_size(type_name: &str) -> usize {
    // 对于无IDL上下文的调用，使用默认值
    match type_name {
        "VestingSchedule" => 40,
        _ => {
            log::debug!("  🤔 未知自定义类型 '{}' 默认为32字节（无IDL上下文）", type_name);
            32
        }
    }
}


/// 计算NonAnchor账户的PACKED_LEN大小  
pub fn calculate_non_anchor_account_packed_size(account: &NonAnchorAccount, idl_enum: &IdlFormatEnum) -> usize {
    let mut size = 0; // NonAnchor 没有 discriminator
    let mut cache: HashMap<String, usize> = HashMap::new();
    
    log::debug!("🧮 计算NonAnchor账户 {} 的大小，开始大小: {}", account.name, size);
    
    if let Some(fields) = &account.fields {
        log::debug!("  🎯 使用直接字段 ({} 个)", fields.len());
        for field in fields {
            let field_size = calculate_non_anchor_field_size_recursive(&field.field_type, idl_enum, &mut cache);
            log::debug!("  📐 字段 {} ({:?}): {} 字节", field.name, field.field_type, field_size);
            size += field_size;
        }
    } else {
        log::debug!("  🔍 账户无直接字段");
    }
    
    log::debug!("🏁 NonAnchor账户 {} 总大小: {} 字节", account.name, size);
    size
}