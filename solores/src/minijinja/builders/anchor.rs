//! Anchor IDL 值构建器
//! 
//! 负责将 Anchor IDL 数据结构转换为模板可用的 Value 对象

use crate::idl_format::{IdlFormatEnum, anchor_idl::*};
use crate::minijinja::generators::pda::PdaInfo;
use convert_case::{Case, Casing};
use minijinja::{context, Value};
use log;
use super::super::utils;
use super::super::filters::extract_smallvec_inner;
use std::collections::HashMap;

/// 类型特征支持注册表
#[derive(Debug, Clone)]
pub struct TypeTraitRegistry {
    /// 记录哪些类型支持Copy trait
    pub copy_supported: HashMap<String, bool>,
    /// 记录哪些类型支持Eq trait  
    pub eq_supported: HashMap<String, bool>,
}

impl TypeTraitRegistry {
    pub fn new() -> Self {
        Self {
            copy_supported: HashMap::new(),
            eq_supported: HashMap::new(),
        }
    }
    
    /// 构建类型注册表，分析所有类型的trait支持情况
    pub fn build_from_idl(idl_enum: &IdlFormatEnum) -> Self {
        let mut registry = Self::new();
        
        match idl_enum {
            IdlFormatEnum::Anchor(anchor_idl) => {
                // 分析所有自定义类型
                if let Some(ref types) = anchor_idl.types {
                    for type_def in types {
                        let type_name = type_def.name.clone();
                        
                        let (can_copy, can_eq) = match &type_def.kind {
                            Some(AnchorTypeKind::Struct(fields)) => {
                                // 先进行基础分析（不考虑递归依赖）
                                let copy_compat = is_struct_copy_compatible_basic(fields);
                                let eq_compat = is_struct_eq_compatible_basic(fields);
                                (copy_compat, eq_compat)
                            },
                            Some(AnchorTypeKind::Enum(variants)) => {
                                let copy_compat = is_enum_copy_compatible_basic(variants);
                                let eq_compat = is_enum_eq_compatible_basic(variants);
                                (copy_compat, eq_compat)
                            },
                            _ => (true, true), // 其他类型默认支持
                        };
                        
                        registry.copy_supported.insert(type_name.clone(), can_copy);
                        registry.eq_supported.insert(type_name, can_eq);
                    }
                }
            },
            IdlFormatEnum::NonAnchor(_) => {
                // NonAnchor暂时使用原有逻辑
            },
        }
        
        // 进行递归依赖解析，多次迭代直到稳定
        registry.resolve_recursive_dependencies(idl_enum);
        registry
    }
    
    /// 递归解析依赖关系，多次迭代直到稳定
    fn resolve_recursive_dependencies(&mut self, idl_enum: &IdlFormatEnum) {
        let mut changed = true;
        let max_iterations = 10; // 防止无限循环
        let mut iteration = 0;
        
        while changed && iteration < max_iterations {
            changed = false;
            iteration += 1;
            
            if let IdlFormatEnum::Anchor(anchor_idl) = idl_enum {
                if let Some(ref types) = anchor_idl.types {
                    for type_def in types {
                        let type_name = &type_def.name;
                        
                        let (new_copy, new_eq) = match &type_def.kind {
                            Some(AnchorTypeKind::Struct(fields)) => {
                                let copy_compat = self.is_struct_copy_compatible_with_registry(fields);
                                let eq_compat = self.is_struct_eq_compatible_with_registry(fields);
                                (copy_compat, eq_compat)
                            },
                            Some(AnchorTypeKind::Enum(variants)) => {
                                let copy_compat = self.is_enum_copy_compatible_with_registry(variants);
                                let eq_compat = self.is_enum_eq_compatible_with_registry(variants);
                                (copy_compat, eq_compat)
                            },
                            _ => (true, true),
                        };
                        
                        // 检查是否有变化
                        let old_copy = *self.copy_supported.get(type_name).unwrap_or(&true);
                        let old_eq = *self.eq_supported.get(type_name).unwrap_or(&true);
                        
                        if old_copy != new_copy || old_eq != new_eq {
                            changed = true;
                            self.copy_supported.insert(type_name.clone(), new_copy);
                            self.eq_supported.insert(type_name.clone(), new_eq);
                        }
                    }
                }
            }
        }
        
        log::debug!("类型注册表构建完成，迭代了 {} 次", iteration);
    }
    
    /// 检查类型是否支持Copy trait（使用注册表）
    pub fn is_type_copy_compatible(&self, type_name: &str) -> bool {
        self.copy_supported.get(type_name).copied().unwrap_or(true)
    }
    
    /// 检查类型是否支持Eq trait（使用注册表）
    pub fn is_type_eq_compatible(&self, type_name: &str) -> bool {
        self.eq_supported.get(type_name).copied().unwrap_or(true)
    }
    
    /// 使用注册表检查字段类型是否支持Copy trait
    fn is_field_copy_compatible_with_registry(&self, field_type: &AnchorFieldType) -> bool {
        match field_type {
            AnchorFieldType::Basic(type_name) => {
                // 基础Copy兼容类型
                matches!(type_name.as_str(),
                    "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | 
                    "u64" | "i64" | "u128" | "i128" | "bool" | "f32" | "f64" |
                    "publicKey" | "pubkey"
                )
            },
            AnchorFieldType::PrimitiveOrPubkey(type_name) => {
                matches!(type_name.as_str(), "publicKey" | "pubkey")
            },
            AnchorFieldType::array(element_type, _size) => {
                self.is_field_copy_compatible_with_registry(element_type)
            },
            AnchorFieldType::option(inner_type) => {
                self.is_field_copy_compatible_with_registry(inner_type)
            },
            AnchorFieldType::vec(_) => false,
            AnchorFieldType::Complex { kind, params: _ } => {
                match kind.as_str() {
                    "vec" => false,
                    "option" => true, // 需要进一步分析内部类型
                    _ => false,
                }
            },
            AnchorFieldType::defined(name) => {
                // 首先检查是否为复杂类型（SmallVec、Vec、HashMap等）
                if name.starts_with("SmallVec<") || name.contains("String") || name.contains("Vec<") || name.contains("HashMap<") {
                    false
                } else if name.starts_with('[') && name.contains(';') && name.contains(']') {
                    // 数组类型检查：现代Rust支持所有大小数组的Copy trait
                    if let Some((_, size_str)) = extract_array_parts_from_string(name) {
                        if let Ok(_size) = size_str.parse::<usize>() {
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    // 对于纯自定义类型，使用注册表查询
                    self.is_type_copy_compatible(name)
                }
            }
        }
    }
    
    /// 使用注册表检查字段类型是否支持Eq trait
    fn is_field_eq_compatible_with_registry(&self, field_type: &AnchorFieldType) -> bool {
        match field_type {
            AnchorFieldType::Basic(type_name) => {
                // 基础Eq兼容类型（浮点数不支持Eq）
                !matches!(type_name.as_str(), "f32" | "f64")
            },
            AnchorFieldType::PrimitiveOrPubkey(_) => true,
            AnchorFieldType::array(element_type, _size) => {
                self.is_field_eq_compatible_with_registry(element_type)
            },
            AnchorFieldType::option(inner_type) => {
                self.is_field_eq_compatible_with_registry(inner_type)
            },
            AnchorFieldType::vec(element_type) => {
                self.is_field_eq_compatible_with_registry(element_type)
            },
            AnchorFieldType::Complex { kind: _, params: _ } => {
                true // 复杂类型需要进一步分析
            },
            AnchorFieldType::defined(name) => {
                // 首先检查是否包含浮点数类型或其他不支持Eq的类型
                if name.contains("f32") || name.contains("f64") {
                    false
                } else {
                    // 对于其他类型，使用注册表查询自定义类型的Eq支持情况
                    self.is_type_eq_compatible(name)
                }
            }
        }
    }
    
    /// 使用注册表检查struct是否支持Copy trait
    fn is_struct_copy_compatible_with_registry(&self, fields: &[AnchorField]) -> bool {
        fields.iter().all(|field| self.is_field_copy_compatible_with_registry(&field.field_type))
    }
    
    /// 使用注册表检查struct是否支持Eq trait
    fn is_struct_eq_compatible_with_registry(&self, fields: &[AnchorField]) -> bool {
        fields.iter().all(|field| self.is_field_eq_compatible_with_registry(&field.field_type))
    }
    
    /// 使用注册表检查enum是否支持Copy trait
    fn is_enum_copy_compatible_with_registry(&self, variants: &[AnchorEnumVariant]) -> bool {
        variants.iter().all(|variant| {
            if let Some(ref fields) = variant.fields {
                fields.iter().all(|field| self.is_field_copy_compatible_with_registry(&field.field_type))
            } else {
                true
            }
        })
    }
    
    /// 使用注册表检查enum是否支持Eq trait
    fn is_enum_eq_compatible_with_registry(&self, variants: &[AnchorEnumVariant]) -> bool {
        variants.iter().all(|variant| {
            if let Some(ref fields) = variant.fields {
                fields.iter().all(|field| self.is_field_eq_compatible_with_registry(&field.field_type))
            } else {
                true
            }
        })
    }
}

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
            let mut account_value = Value::from_serialize(acc);
            
            // 如果账户有PDA定义，添加PDA处理信息
            if let Some(pda_def) = &acc.pda {
                if let Ok(pda_info) = PdaInfo::from_pda_definition(&acc.name, pda_def) {
                    log::debug!("🔍 为指令账户 {} 添加PDA信息", acc.name);
                    
                    // 将PDA信息添加到账户value中
                    let pda_context = context! {
                        has_dynamic_params => pda_info.has_dynamic_params,
                        function_params => pda_info.function_params.iter().map(|p| {
                            context! {
                                name => p.name.clone(),
                                param_type => p.param_type.clone()
                            }
                        }).collect::<Vec<_>>(),
                        seeds_code => pda_info.seeds_code
                    };
                    
                    // 创建包含PDA信息的新上下文
                    if let Ok(mut acc_map) = serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(
                        serde_json::to_value(acc).unwrap_or_default()
                    ) {
                        acc_map.insert("pda".to_string(), serde_json::to_value(pda_context).unwrap_or_default());
                        account_value = Value::from_serialize(&acc_map);
                    }
                }
            }
            
            account_value
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
            
            // 检查struct是否支持Copy和Eq traits
            let can_copy = is_struct_copy_compatible(fields);
            let can_eq = is_struct_eq_compatible(fields);
            
            context! {
                name => type_def.name.to_case(Case::Pascal),
                fields => fields_values,
                kind => "struct",
                can_copy => can_copy,
                can_eq => can_eq,
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
            
            // 检查enum是否支持Copy和Eq traits
            let can_copy = is_enum_copy_compatible(variants);
            let can_eq = is_enum_eq_compatible(variants);
            
            context! {
                name => type_def.name.to_case(Case::Pascal),
                variants => variants_values,
                kind => "enum",
                can_copy => can_copy,
                can_eq => can_eq,
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

/// 使用类型注册表构建类型Value，确保trait支持的准确性
pub fn build_type_value_with_registry(type_def: &AnchorType, registry: &TypeTraitRegistry) -> Value {
    match &type_def.kind {
        Some(AnchorTypeKind::Struct(fields)) => {
            let fields_values: Vec<Value> = fields.iter().map(|field| {
                build_field_value(field)
            }).collect();
            
            // 使用注册表检查struct是否支持Copy和Eq traits
            let can_copy = registry.is_struct_copy_compatible_with_registry(fields);
            let can_eq = registry.is_struct_eq_compatible_with_registry(fields);
            
            context! {
                name => type_def.name.to_case(Case::Pascal),
                fields => fields_values,
                kind => "struct",
                can_copy => can_copy,
                can_eq => can_eq,
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
            
            // 使用注册表检查enum是否支持Copy和Eq traits
            let can_copy = registry.is_enum_copy_compatible_with_registry(variants);
            let can_eq = registry.is_enum_eq_compatible_with_registry(variants);
            
            context! {
                name => type_def.name.to_case(Case::Pascal),
                variants => variants_values,
                kind => "enum",
                can_copy => can_copy,
                can_eq => can_eq,
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
        name => field.name.to_case(Case::Snake),
        rust_type => rust_type,
        is_pubkey => is_anchor_field_pubkey(&field.field_type),
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
        AnchorFieldType::defined(name) => {
            log::debug!("🔍 Processing defined type: '{}'", name);
            
            // SmallVec特殊处理 - 转换为Vec
            if name.starts_with("SmallVec<") {
                log::debug!("🔧 SmallVec detected, converting to Vec: '{}'", name);
                if let Some(inner) = extract_smallvec_inner(name) {
                    let processed_inner = if inner == "Pubkey" {
                        "solana_pubkey::Pubkey".to_string()
                    } else if inner == "CompiledInstruction" || inner == "MessageAddressTableLookup" {
                        format!("crate::types::{}", inner)
                    } else if matches!(inner.as_str(), "u8" | "u16" | "u32" | "u64") {
                        inner
                    } else {
                        format!("crate::types::{}", inner)
                    };
                    let result = format!("Vec<{}>", processed_inner);
                    log::debug!("✅ SmallVec converted: '{}' -> '{}'", name, result);
                    return result;
                } else {
                    log::warn!("⚠️ SmallVec parsing failed for: '{}'", name);
                }
            }
            
            // 检查是否是基础数组类型语法，避免为内置数组类型添加前缀
            let is_array_syntax = name.starts_with('[') && name.ends_with(']');
            let is_std_type = name.starts_with("std::") || name.starts_with("solana_");
            let is_basic_type = matches!(name.as_str(), "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | 
                                      "u64" | "i64" | "u128" | "i128" | "bool" | 
                                      "f32" | "f64" | "string");
            let has_array_pattern = name.contains('[') && name.contains(';') && name.contains(']');
            
            log::debug!("  is_array_syntax: {}, is_std_type: {}, is_basic_type: {}, has_array_pattern: {}", 
                       is_array_syntax, is_std_type, is_basic_type, has_array_pattern);
            
            if is_array_syntax || is_std_type || is_basic_type || has_array_pattern {
                // 这是内置类型或数组，不需要添加前缀
                log::debug!("✅ Keeping as-is: '{}'", name);
                name.clone()
            } else {
                // 这是真正的用户定义类型，需要添加前缀
                log::debug!("🏷️ Adding crate::types prefix: '{}'", name);
                format!("crate::types::{}", name)
            }
        },
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

/// 检查Anchor字段是否为Pubkey类型
pub fn is_anchor_field_pubkey(field_type: &AnchorFieldType) -> bool {
    match field_type {
        AnchorFieldType::Basic(type_name) => {
            matches!(type_name.as_str(), "publicKey" | "pubkey")
        },
        AnchorFieldType::PrimitiveOrPubkey(type_name) => {
            matches!(type_name.as_str(), "publicKey" | "pubkey")
        },
        _ => false,
    }
}

/// 检查字段类型是否支持Copy trait
pub fn is_field_copy_compatible(field_type: &AnchorFieldType) -> bool {
    match field_type {
        AnchorFieldType::Basic(type_name) => {
            // 基础Copy兼容类型
            matches!(type_name.as_str(), 
                "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | 
                "u64" | "i64" | "u128" | "i128" | "bool" | 
                "f32" | "f64" | "publicKey" | "pubkey"
            )
        },
        AnchorFieldType::PrimitiveOrPubkey(type_name) => {
            matches!(type_name.as_str(), "publicKey" | "pubkey")
        },
        AnchorFieldType::array(element_type, _size) => {
            // 数组支持Copy当且仅当元素支持Copy：移除32字节限制（Rust 1.51+支持任意大小数组Copy）
            is_field_copy_compatible(element_type)
        },
        AnchorFieldType::option(inner_type) => {
            // Option支持Copy当且仅当内部类型支持Copy
            is_field_copy_compatible(inner_type)
        },
        AnchorFieldType::vec(_) => {
            // Vec永远不支持Copy
            false
        },
        AnchorFieldType::Complex { kind, params } => {
            // 分析复杂类型
            match kind.as_str() {
                "array" => {
                    // 数组类型：现代Rust支持所有大小数组的Copy trait
                    if let Some(params) = params {
                        if params.len() >= 2 {
                            if let (Some(_element_type), Some(size_val)) = (params.get(0), params.get(1)) {
                                if let Some(_size) = size_val.as_u64() {
                                    // 移除32字节限制：Rust 1.51+支持任意大小数组Copy
                                    return true;
                                }
                            }
                        }
                    }
                    false
                },
                "option" => {
                    // Option类型：需要内部类型支持Copy
                    if let Some(params) = params {
                        if let Some(_inner_type_val) = params.get(0) {
                            // 这里需要递归检查，暂时简化处理
                            return true;
                        }
                    }
                    false
                },
                "vec" => false, // Vec永不支持Copy
                _ => false, // 其他复杂类型暂时不支持Copy
            }
        },
        AnchorFieldType::defined(name) => {
            // 处理SmallVec和其他定义类型
            if name.starts_with("SmallVec<") || name.contains("String") || name.contains("Vec<") || name.contains("HashMap<") {
                false
            } else if name.starts_with('[') && name.contains(';') && name.contains(']') {
                // 数组类型检查：现代Rust支持所有大小数组的Copy trait
                if let Some((_, size_str)) = extract_array_parts_from_string(name) {
                    if let Ok(_size) = size_str.parse::<usize>() {
                        // 移除32字节限制：Rust 1.51+支持任意大小数组Copy
                        return true;
                    }
                }
                false
            } else {
                // 其他自定义类型默认需要进一步检查，暂时假设支持Copy
                true
            }
        }
    }
}

/// 检查字段类型是否支持Eq trait
pub fn is_field_eq_compatible(field_type: &AnchorFieldType) -> bool {
    match field_type {
        AnchorFieldType::Basic(type_name) => {
            // Eq兼容类型，排除浮点数
            matches!(type_name.as_str(), 
                "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | 
                "u64" | "i64" | "u128" | "i128" | "bool" | 
                "publicKey" | "pubkey"
                // 注意：f32, f64不支持Eq
            )
        },
        AnchorFieldType::PrimitiveOrPubkey(type_name) => {
            matches!(type_name.as_str(), "publicKey" | "pubkey")
        },
        AnchorFieldType::array(element_type, _) => {
            // 数组支持Eq当且仅当元素支持Eq
            is_field_eq_compatible(element_type)
        },
        AnchorFieldType::option(inner_type) => {
            // Option支持Eq当且仅当内部类型支持Eq
            is_field_eq_compatible(inner_type)
        },
        AnchorFieldType::vec(element_type) => {
            // Vec支持Eq当且仅当元素支持Eq
            is_field_eq_compatible(element_type)
        },
        AnchorFieldType::Complex { kind: _, params: _ } => {
            // 复杂类型需要进一步分析，暂时假设支持Eq
            true
        },
        AnchorFieldType::defined(name) => {
            // 检查是否包含浮点数
            if name.contains("f32") || name.contains("f64") {
                false
            } else {
                // 其他自定义类型默认支持Eq
                true
            }
        }
    }
}

/// 检查enum是否支持Copy trait（所有变体的所有字段都支持Copy）
pub fn is_enum_copy_compatible(variants: &[AnchorEnumVariant]) -> bool {
    variants.iter().all(|variant| {
        if let Some(ref fields) = variant.fields {
            fields.iter().all(|field| is_field_copy_compatible(&field.field_type))
        } else {
            // 无字段的变体总是支持Copy
            true
        }
    })
}

/// 检查enum是否支持Eq trait（所有变体的所有字段都支持Eq）
pub fn is_enum_eq_compatible(variants: &[AnchorEnumVariant]) -> bool {
    variants.iter().all(|variant| {
        if let Some(ref fields) = variant.fields {
            fields.iter().all(|field| is_field_eq_compatible(&field.field_type))
        } else {
            // 无字段的变体总是支持Eq
            true
        }
    })
}

/// 检查struct是否支持Copy trait（所有字段都支持Copy）
pub fn is_struct_copy_compatible(fields: &[AnchorField]) -> bool {
    fields.iter().all(|field| is_field_copy_compatible(&field.field_type))
}

/// 检查struct是否支持Eq trait（所有字段都支持Eq）
pub fn is_struct_eq_compatible(fields: &[AnchorField]) -> bool {
    fields.iter().all(|field| is_field_eq_compatible(&field.field_type))
}

/// 基础版本：检查字段类型是否支持Copy trait（不考虑递归依赖）
pub fn is_field_copy_compatible_basic(field_type: &AnchorFieldType) -> bool {
    match field_type {
        AnchorFieldType::Basic(type_name) => {
            // 基础Copy兼容类型
            matches!(type_name.as_str(),
                "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | 
                "u64" | "i64" | "u128" | "i128" | "bool" | "f32" | "f64" |
                "publicKey" | "pubkey"
            )
        },
        AnchorFieldType::PrimitiveOrPubkey(type_name) => {
            // Pubkey支持Copy
            matches!(type_name.as_str(), "publicKey" | "pubkey")
        },
        AnchorFieldType::array(element_type, _size) => {
            // 数组支持Copy当且仅当元素支持Copy
            is_field_copy_compatible_basic(element_type)
        },
        AnchorFieldType::option(inner_type) => {
            // Option支持Copy当且仅当内部类型支持Copy
            is_field_copy_compatible_basic(inner_type)
        },
        AnchorFieldType::vec(_) => {
            // Vec永远不支持Copy
            false
        },
        AnchorFieldType::Complex { kind, params: _ } => {
            // 复杂类型需要分析，暂时保守处理
            match kind.as_str() {
                "vec" => false,
                "option" => true, // 需要进一步分析内部类型
                _ => false,
            }
        },
        AnchorFieldType::defined(_) => {
            // 自定义类型在基础版本中假设不支持Copy（保守处理）
            false
        }
    }
}

/// 基础版本：检查字段类型是否支持Eq trait（不考虑递归依赖）
pub fn is_field_eq_compatible_basic(field_type: &AnchorFieldType) -> bool {
    match field_type {
        AnchorFieldType::Basic(type_name) => {
            // 基础Eq兼容类型（浮点数不支持Eq）
            !matches!(type_name.as_str(), "f32" | "f64")
        },
        AnchorFieldType::PrimitiveOrPubkey(_) => true,
        AnchorFieldType::array(element_type, _size) => {
            is_field_eq_compatible_basic(element_type)
        },
        AnchorFieldType::option(inner_type) => {
            is_field_eq_compatible_basic(inner_type)
        },
        AnchorFieldType::vec(element_type) => {
            is_field_eq_compatible_basic(element_type)
        },
        AnchorFieldType::Complex { kind: _, params: _ } => {
            true // 基础版本假设复杂类型支持Eq
        },
        AnchorFieldType::defined(_) => {
            // 自定义类型在基础版本中假设不支持Eq（保守处理）
            false
        }
    }
}

/// 基础版本：检查struct是否支持Copy trait（不考虑递归依赖）
pub fn is_struct_copy_compatible_basic(fields: &[AnchorField]) -> bool {
    fields.iter().all(|field| is_field_copy_compatible_basic(&field.field_type))
}

/// 基础版本：检查struct是否支持Eq trait（不考虑递归依赖）
pub fn is_struct_eq_compatible_basic(fields: &[AnchorField]) -> bool {
    fields.iter().all(|field| is_field_eq_compatible_basic(&field.field_type))
}

/// 基础版本：检查enum是否支持Copy trait（不考虑递归依赖）
pub fn is_enum_copy_compatible_basic(variants: &[AnchorEnumVariant]) -> bool {
    variants.iter().all(|variant| {
        if let Some(ref fields) = variant.fields {
            fields.iter().all(|field| is_field_copy_compatible_basic(&field.field_type))
        } else {
            true
        }
    })
}

/// 基础版本：检查enum是否支持Eq trait（不考虑递归依赖）
pub fn is_enum_eq_compatible_basic(variants: &[AnchorEnumVariant]) -> bool {
    variants.iter().all(|variant| {
        if let Some(ref fields) = variant.fields {
            fields.iter().all(|field| is_field_eq_compatible_basic(&field.field_type))
        } else {
            true
        }
    })
}

/// 辅助函数：从字符串中提取数组部分
fn extract_array_parts_from_string(value: &str) -> Option<(String, String)> {
    if !value.starts_with("[") || !value.ends_with("]") {
        return None;
    }
    
    let inner = &value[1..value.len()-1];
    
    if let Some(semicolon_pos) = inner.rfind(';') {
        let type_part = inner[..semicolon_pos].trim().to_string();
        let size_part = inner[semicolon_pos+1..].trim().to_string();
        
        if size_part.chars().all(|c| c.is_ascii_digit()) {
            Some((type_part, size_part))
        } else {
            None
        }
    } else {
        None
    }
}