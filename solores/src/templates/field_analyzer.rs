//! 字段分配分析器
//!
//! 分析IDL中各模块的字段使用情况，实现全面的字段排除机制
//! 核心功能：确保Types中不包含Instructions/Events/Accounts使用的字段定义

use std::collections::{HashMap, HashSet};

/// 字段分配映射
#[derive(Debug, Clone)]
pub struct FieldAllocationMap {
    /// Instructions模块使用的字段类型 (instruction_name -> args字段类型列表)
    pub instructions_used_types: HashMap<String, HashSet<String>>,
    /// Events模块应该包含的字段 (event_name -> 字段列表)
    pub events_fields: HashMap<String, Vec<FieldDefinition>>,
    /// Events模块使用的字段类型
    pub events_used_types: HashSet<String>,
    /// Accounts模块应该包含的字段 (account_name -> 字段列表)  
    pub accounts_fields: HashMap<String, Vec<FieldDefinition>>,
    /// Accounts模块使用的字段类型
    pub accounts_used_types: HashSet<String>,
    /// Types模块剩余的字段 (type_name -> 字段列表)
    pub types_remaining_fields: HashMap<String, Vec<FieldDefinition>>,
    /// 所有被其他模块使用的类型名称集合
    pub all_used_types: HashSet<String>,
    /// 🆕 被引用的类型（应保留在types模块中）
    pub referenced_types: HashSet<String>,
    /// 🆕 被实现的类型（应从types模块中移除）
    pub implemented_types: HashSet<String>,
}

/// 字段定义
#[derive(Debug, Clone)]
pub struct FieldDefinition {
    pub name: String,
    pub field_type: String, // 简化的类型表示
    pub docs: Vec<String>,
}

/// 字段分配分析器
pub struct FieldAllocationAnalyzer;

impl FieldAllocationAnalyzer {
    /// 分析Anchor IDL的字段分配 - 智能引用vs实现检测版本（修复版）
    pub fn analyze_anchor_idl(idl: &crate::idl_format::anchor_idl::AnchorIdl) -> FieldAllocationMap {
        log::debug!("🔍 开始分析Anchor IDL字段分配（智能检测模式）");
        
        let mut allocation = FieldAllocationMap {
            instructions_used_types: HashMap::new(),
            events_fields: HashMap::new(),
            events_used_types: HashSet::new(),
            accounts_fields: HashMap::new(),
            accounts_used_types: HashSet::new(),
            types_remaining_fields: HashMap::new(),
            all_used_types: HashSet::new(),
            referenced_types: HashSet::new(),
            implemented_types: HashSet::new(),
        };

        // 1. 收集所有Types定义（初始状态）
        if let Some(types) = &idl.types {
            log::debug!("📚 发现 {} 个types定义", types.len());
            for type_def in types {
                log::debug!("  - Type: {}", type_def.name);
                if let Some(kind) = &type_def.kind {
                    match kind {
                        crate::idl_format::anchor_idl::AnchorTypeKind::Struct(fields) => {
                            log::debug!("    └─ Struct with {} fields", fields.len());
                            for field in fields {
                                log::debug!("      • {}: {:?}", field.name, field.field_type);
                            }
                            
                            let field_defs = fields.iter().map(|field| FieldDefinition {
                                name: field.name.clone(),
                                field_type: Self::format_anchor_type(&field.field_type),
                                docs: field.docs.clone().unwrap_or_default(),
                            }).collect();
                            
                            allocation.types_remaining_fields.insert(
                                type_def.name.clone(),
                                field_defs
                            );
                        },
                        _ => log::debug!("    └─ Non-struct type: {:?}", kind),
                    }
                } else {
                    log::debug!("    └─ ❌ No kind defined for type {}", type_def.name);
                }
            }
        } else {
            log::debug!("❌ IDL中没有types数组！");
        }

        // 2. 分析Events字段需求和使用的类型
        if let Some(events) = &idl.events {
            log::debug!("🎯 发现 {} 个events", events.len());
            for event in events {
                let event_name = &event.name;
                log::debug!("  - Event: {}", event_name);
                
                // 使用新的IDL判断方法检查事件是否有直接字段定义
                let has_direct_fields = idl.has_event_fields(event_name);
                log::debug!("    └─ 直接字段: {}", has_direct_fields);
                
                if has_direct_fields {
                    // 事件有直接字段定义 - 这是实现关系，应从types中移除
                    if let Some(event_fields) = idl.get_event_fields(event_name) {
                        log::debug!("    └─ 使用直接字段 ({} 个) - 实现关系", event_fields.len());
                        let field_defs = event_fields.iter().map(|field| FieldDefinition {
                            name: field.name.clone(),
                            field_type: Self::format_anchor_type(&field.field_type),
                            docs: field.docs.clone().unwrap_or_default(),
                        }).collect();
                        
                        allocation.events_fields.insert(event_name.clone(), field_defs);
                        allocation.events_used_types.insert(event_name.clone());
                        allocation.all_used_types.insert(event_name.clone());
                        // Event有直接字段定义，这是实现关系，应从types中移除
                        allocation.implemented_types.insert(event_name.clone());
                        
                        // 收集Events字段中引用的其他类型（这些类型应保留在types中）
                        for field in event_fields {
                            if let Some(type_name) = Self::extract_type_name_from_anchor_type(&field.field_type) {
                                if type_name != *event_name {  // 避免自引用
                                    allocation.referenced_types.insert(type_name.clone());
                                    allocation.all_used_types.insert(type_name.clone());
                                    log::debug!("      → 字段引用类型: {}", type_name);
                                }
                            }
                        }
                    }
                } else {
                    // 事件没有直接字段定义，尝试从Types获取 - 对于Anchor IDL这是重复实现关系
                    let has_type_fields = allocation.types_remaining_fields.contains_key(event_name);
                    log::debug!("    └─ Types中有字段: {}", has_type_fields);
                    
                    if has_type_fields {
                        log::debug!("    └─ ✅ 从Types获取字段定义 - Anchor重复实现关系");
                        allocation.events_used_types.insert(event_name.clone());
                        allocation.all_used_types.insert(event_name.clone());
                        // Event从types获取字段，对于Anchor IDL这是重复实现关系，应从types中移除
                        // 因为Events模块会生成带discriminator的完整结构体
                        allocation.implemented_types.insert(event_name.clone());
                        
                        if let Some(fields) = allocation.types_remaining_fields.get(event_name) {
                            log::debug!("    └─ 重复实现字段 {} 个", fields.len());
                            allocation.events_fields.insert(event_name.clone(), fields.clone());
                        }
                    } else {
                        // 无字段可用，创建空字段列表
                        log::debug!("    └─ ❌ 创建空字段列表");
                        allocation.events_fields.insert(event_name.clone(), Vec::new());
                    }
                }
            }
        } else {
            log::debug!("❌ IDL中没有events数组！");
        }

        // 4. 分析Accounts字段需求和使用的类型
        if let Some(accounts) = &idl.accounts {
            log::debug!("🏦 发现 {} 个accounts", accounts.len());
            for account in accounts {
                let account_name = &account.name;
                log::debug!("  - Account: {}", account_name);
                
                // 使用新的IDL判断方法检查账户是否有直接字段定义
                let has_direct_fields = idl.has_account_fields(account_name);
                log::debug!("    └─ 直接字段: {}", has_direct_fields);
                
                if has_direct_fields {
                    // 账户有直接字段定义 - 这是实现关系，应从types中移除
                    if let Some(account_fields) = idl.get_account_fields(account_name) {
                        log::debug!("    └─ 使用直接字段 ({} 个) - 实现关系", account_fields.len());
                        let field_defs = account_fields.iter().map(|field| FieldDefinition {
                            name: field.name.clone(),
                            field_type: Self::format_anchor_type(&field.field_type),
                            docs: field.docs.clone().unwrap_or_default(),
                        }).collect();
                        
                        allocation.accounts_fields.insert(account_name.clone(), field_defs);
                        allocation.accounts_used_types.insert(account_name.clone());
                        allocation.all_used_types.insert(account_name.clone());
                        // Account有直接字段定义，这是实现关系，应从types中移除
                        allocation.implemented_types.insert(account_name.clone());
                        
                        // 收集Accounts字段中引用的其他类型（这些类型应保留在types中）
                        for field in account_fields {
                            if let Some(type_name) = Self::extract_type_name_from_anchor_type(&field.field_type) {
                                if type_name != *account_name {  // 避免自引用
                                    allocation.referenced_types.insert(type_name.clone());
                                    allocation.all_used_types.insert(type_name.clone());
                                    log::debug!("      → 字段引用类型: {}", type_name);
                                }
                            }
                        }
                    }
                } else {
                    // 账户没有直接字段定义，尝试从Types获取 - 对于Anchor IDL这是重复实现关系
                    let has_type_fields = allocation.types_remaining_fields.contains_key(account_name);
                    log::debug!("    └─ Types中有字段: {}", has_type_fields);
                    
                    if has_type_fields {
                        log::debug!("    └─ ✅ 从Types获取字段定义 - Anchor重复实现关系");
                        allocation.accounts_used_types.insert(account_name.clone());
                        allocation.all_used_types.insert(account_name.clone());
                        // Account从types获取字段，对于Anchor IDL这是重复实现关系，应从types中移除
                        // 因为Accounts模块会生成带discriminator的完整结构体
                        allocation.implemented_types.insert(account_name.clone());
                        
                        if let Some(fields) = allocation.types_remaining_fields.get(account_name) {
                            log::debug!("    └─ 重复实现字段 {} 个", fields.len());
                            allocation.accounts_fields.insert(account_name.clone(), fields.clone());
                        }
                    } else {
                        // 无字段可用，创建空字段列表
                        log::debug!("    └─ ❌ 创建空字段列表");
                        allocation.accounts_fields.insert(account_name.clone(), Vec::new());
                    }
                }
            }
        } else {
            log::debug!("❌ IDL中没有accounts数组！");
        }

        // 4.5. 分析Instructions字段需求和使用的类型
        if let Some(instructions) = &idl.instructions {
            log::debug!("📝 发现 {} 个instructions", instructions.len());
            for instruction in instructions {
                let instruction_name = &instruction.name;
                log::debug!("  - Instruction: {}", instruction_name);
                
                // 分析指令参数中直接使用的类型
                if let Some(args) = &instruction.args {
                    log::debug!("    └─ 参数字段 {} 个", args.len());
                    for arg in args {
                        if let Some(type_name) = Self::extract_type_name_from_anchor_type(&arg.field_type) {
                            log::debug!("      → Instructions引用类型: {}", type_name);
                            allocation.all_used_types.insert(type_name.clone());
                            // Instructions的IxData结构体引用该类型，这是引用关系，应保留在types中
                            allocation.referenced_types.insert(type_name.clone());
                        }
                    }
                }
            }
        } else {
            log::debug!("❌ IDL中没有instructions数组！");
        }

        // 5. 智能移除类型定义 - 只移除被实现的类型，保留被引用的类型
        for implemented_type in &allocation.implemented_types {
            allocation.types_remaining_fields.remove(implemented_type);
            log::debug!("🗑️ 从types移除被实现的类型: {}", implemented_type);
        }
        
        for referenced_type in &allocation.referenced_types {
            log::debug!("🔗 保留在types中的被引用类型: {}", referenced_type);
        }

        // 6. 输出最终分配结果
        log::debug!("📊 Anchor IDL字段分配完成 - 智能分配结果:");
        log::debug!("  - Events字段: {:?}", allocation.events_fields.keys().collect::<Vec<_>>());
        log::debug!("  - Accounts字段: {:?}", allocation.accounts_fields.keys().collect::<Vec<_>>());
        log::debug!("  - 被引用的类型: {:?}", allocation.referenced_types);
        log::debug!("  - 被实现的类型: {:?}", allocation.implemented_types);
        log::debug!("  - Types模块保留: {:?}", allocation.types_remaining_fields.keys().collect::<Vec<_>>());
        
        // 详细输出每个分配的字段数量
        for (event_name, fields) in &allocation.events_fields {
            log::debug!("    • Event {} 有 {} 个字段", event_name, fields.len());
        }
        for (account_name, fields) in &allocation.accounts_fields {
            log::debug!("    • Account {} 有 {} 个字段", account_name, fields.len());
        }

        allocation
    }


    /// 格式化Anchor字段类型为字符串
    fn format_anchor_type(field_type: &crate::idl_format::anchor_idl::AnchorFieldType) -> String {
        let result = match field_type {
            crate::idl_format::anchor_idl::AnchorFieldType::Basic(s) => {
                log::debug!("🔄 格式化Basic类型: '{}'", s);
                s.clone()
            },
            crate::idl_format::anchor_idl::AnchorFieldType::defined(type_name) => {
                log::debug!("🔄 格式化defined类型: '{}'", type_name);
                type_name.clone()
            },
            crate::idl_format::anchor_idl::AnchorFieldType::array(inner_type, size) => {
                log::debug!("🔄 格式化array类型: size = {}", size);
                let inner_type_str = Self::format_anchor_type(inner_type);
                log::debug!("🔄 数组内部类型格式化结果: '{}'", inner_type_str);
                // 确保内部类型格式正确，避免生成无效的数组类型
                if inner_type_str.is_empty() {
                    log::warn!("⚠️  数组内部类型为空，使用u8作为fallback");
                    format!("[u8; {}]", size)
                } else {
                    let formatted = format!("[{}; {}]", inner_type_str, size);
                    log::debug!("✅ 数组类型格式化完成: '{}'", formatted);
                    formatted
                }
            },
            crate::idl_format::anchor_idl::AnchorFieldType::vec(inner_type) => {
                let inner_type_str = Self::format_anchor_type(inner_type);
                if inner_type_str.is_empty() {
                    log::warn!("⚠️  Vec内部类型为空，使用u8作为fallback");
                    "Vec<u8>".to_string()
                } else {
                    format!("Vec<{}>", inner_type_str)
                }
            },
            crate::idl_format::anchor_idl::AnchorFieldType::option(inner_type) => {
                let inner_type_str = Self::format_anchor_type(inner_type);
                if inner_type_str.is_empty() {
                    log::warn!("⚠️  Option内部类型为空，使用u8作为fallback");
                    "Option<u8>".to_string()
                } else {
                    format!("Option<{}>", inner_type_str)
                }
            },
            crate::idl_format::anchor_idl::AnchorFieldType::PrimitiveOrPubkey(s) => {
                // 标准化pubkey类型：pubkey/publicKey -> Pubkey
                match s.as_str() {
                    "pubkey" | "publicKey" => "Pubkey".to_string(),
                    _ => s.clone()
                }
            },
            crate::idl_format::anchor_idl::AnchorFieldType::Complex { kind, .. } => {
                if kind.is_empty() {
                    log::warn!("⚠️  复合类型kind为空，使用u8作为fallback");
                    "u8".to_string()
                } else {
                    kind.clone()
                }
            },
        };
        log::debug!("🎯 format_anchor_type最终结果: '{}'", result);
        result
    }

    /// 格式化NonAnchor字段类型为字符串
    fn format_non_anchor_type(field_type: &crate::idl_format::non_anchor_idl::NonAnchorFieldType) -> String {
        match field_type {
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Basic(s) => {
                // 标准化pubkey类型：pubkey/publicKey -> Pubkey
                match s.as_str() {
                    "pubkey" | "publicKey" => "Pubkey".to_string(),
                    _ => s.clone()
                }
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Defined { defined } => defined.clone(),
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Array { array } => {
                let (inner_type, size) = array;
                format!("[{}; {}]", Self::format_non_anchor_type(inner_type), size)
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Vec { vec } => {
                format!("Vec<{}>", Self::format_non_anchor_type(vec))
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Option { option } => {
                format!("Option<{}>", Self::format_non_anchor_type(option))
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::HashMap { key, value } => {
                format!("HashMap<{}, {}>", Self::format_non_anchor_type(key), Self::format_non_anchor_type(value))
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Complex { kind, .. } => kind.clone(),
        }
    }

    /// 从Anchor字段类型中提取类型名称（用于检测依赖）
    fn extract_type_name_from_anchor_type(field_type: &crate::idl_format::anchor_idl::AnchorFieldType) -> Option<String> {
        match field_type {
            crate::idl_format::anchor_idl::AnchorFieldType::defined(type_name) => Some(type_name.clone()),
            crate::idl_format::anchor_idl::AnchorFieldType::array(inner_type, _) => {
                Self::extract_type_name_from_anchor_type(inner_type)
            },
            crate::idl_format::anchor_idl::AnchorFieldType::vec(inner_type) => {
                Self::extract_type_name_from_anchor_type(inner_type)
            },
            crate::idl_format::anchor_idl::AnchorFieldType::option(inner_type) => {
                Self::extract_type_name_from_anchor_type(inner_type)
            },
            crate::idl_format::anchor_idl::AnchorFieldType::Complex { kind, .. } => {
                // 检查是否为自定义类型
                if !matches!(kind.as_str(), "Vec" | "Option" | "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "bool" | "String" | "Pubkey") {
                    Some(kind.clone())
                } else {
                    None
                }
            },
            _ => None, // 原生类型不需要排除
        }
    }

    /// 从NonAnchor字段类型中提取类型名称（用于检测依赖）
    fn extract_type_name_from_non_anchor_type(field_type: &crate::idl_format::non_anchor_idl::NonAnchorFieldType) -> Option<String> {
        match field_type {
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Defined { defined } => Some(defined.clone()),
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Array { array } => {
                let (inner_type, _) = array;
                Self::extract_type_name_from_non_anchor_type(inner_type)
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Vec { vec } => {
                Self::extract_type_name_from_non_anchor_type(vec)
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Option { option } => {
                Self::extract_type_name_from_non_anchor_type(option)
            },
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Complex { kind, .. } => {
                // 检查是否为自定义类型
                if !matches!(kind.as_str(), "Vec" | "Option" | "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "bool" | "String" | "Pubkey") {
                    Some(kind.clone())
                } else {
                    None
                }
            },
            _ => None, // 原生类型不需要排除
        }
    }

    /// 获取事件应该包含的字段
    pub fn get_event_fields(allocation: &FieldAllocationMap, event_name: &str) -> Vec<FieldDefinition> {
        allocation.events_fields
            .get(event_name)
            .cloned()
            .unwrap_or_default()
    }

    /// 获取账户应该包含的字段
    pub fn get_account_fields(allocation: &FieldAllocationMap, account_name: &str) -> Vec<FieldDefinition> {
        allocation.accounts_fields
            .get(account_name)
            .cloned()
            .unwrap_or_default()
    }

    /// 获取类型剩余的字段（排除已被其他模块使用的）
    pub fn get_type_remaining_fields(allocation: &FieldAllocationMap, type_name: &str) -> Vec<FieldDefinition> {
        allocation.types_remaining_fields
            .get(type_name)
            .cloned()
            .unwrap_or_default()
    }

    /// 检查类型是否被其他模块使用
    pub fn is_type_used_by_others(allocation: &FieldAllocationMap, type_name: &str) -> bool {
        allocation.all_used_types.contains(type_name)
    }

    /// 获取Instructions使用的类型
    pub fn get_instructions_used_types(allocation: &FieldAllocationMap, instruction_name: &str) -> HashSet<String> {
        allocation.instructions_used_types
            .get(instruction_name)
            .cloned()
            .unwrap_or_default()
    }

    /// 获取所有被使用的类型列表
    pub fn get_all_used_types(allocation: &FieldAllocationMap) -> &HashSet<String> {
        &allocation.all_used_types
    }

    /// 获取Types模块中剩余的（未被其他模块使用的）类型名称列表
    pub fn get_remaining_type_names(allocation: &FieldAllocationMap) -> Vec<String> {
        allocation.types_remaining_fields.keys().cloned().collect()
    }

    /// 分析NonAnchor IDL的字段分配 - 智能引用vs实现检测版本
    pub fn analyze_non_anchor_idl(idl: &crate::idl_format::non_anchor_idl::NonAnchorIdl) -> FieldAllocationMap {
        log::debug!("🔍 开始分析NonAnchor IDL字段分配（智能检测模式）");
        
        let mut allocation = FieldAllocationMap {
            instructions_used_types: HashMap::new(),
            events_fields: HashMap::new(),
            events_used_types: HashSet::new(),
            accounts_fields: HashMap::new(),
            accounts_used_types: HashSet::new(),
            types_remaining_fields: HashMap::new(),
            all_used_types: HashSet::new(),
            referenced_types: HashSet::new(),
            implemented_types: HashSet::new(),
        };

        // 1. 先收集所有Types中的字段定义到types_remaining_fields
        if let Some(types) = &idl.types {
            log::debug!("📋 发现 {} 个types", types.len());
            for type_def in types {
                let type_name = &type_def.name;
                log::debug!("  - Type: {}", type_name);
                
                // 根据NonAnchorTypeKind提取字段
                match &type_def.type_def {
                    crate::idl_format::non_anchor_idl::NonAnchorTypeKind::Struct { fields } => {
                        let field_defs: Vec<FieldDefinition> = fields.iter().map(|field| {
                            FieldDefinition {
                                name: field.name.clone(),
                                field_type: Self::format_non_anchor_type(&field.field_type),
                                docs: field.docs.clone().unwrap_or_default(),
                            }
                        }).collect();
                        
                        allocation.types_remaining_fields.insert(type_name.clone(), field_defs);
                        log::debug!("    └─ 收集 {} 个字段", fields.len());
                    },
                    _ => {
                        // 非Struct类型（Enum、Alias等）不收集字段
                        log::debug!("    └─ 非Struct类型，跳过字段收集");
                    }
                }
            }
        }

        // 2. 分析Instructions使用的类型 - 这些都是实现关系（IxData结构体直接使用类型）
        if let Some(instructions) = &idl.instructions {
            log::debug!("📝 发现 {} 个instructions", instructions.len());
            for instruction in instructions {
                let instruction_name = &instruction.name;
                log::debug!("  - Instruction: {}", instruction_name);
                
                let mut used_types = HashSet::new();
                if let Some(args) = &instruction.args {
                    log::debug!("    └─ 参数字段 {} 个", args.len());
                    for arg in args {
                        if let Some(type_name) = Self::extract_type_name_from_non_anchor_type(&arg.field_type) {
                            used_types.insert(type_name.clone());
                            allocation.all_used_types.insert(type_name.clone());
                            // Instructions的IxData结构体引用该类型，这是引用关系，应保留在types中
                            allocation.referenced_types.insert(type_name.clone());
                            log::debug!("      → Instructions引用类型: {}", type_name);
                        }
                    }
                }
                allocation.instructions_used_types.insert(instruction_name.clone(), used_types);
            }
        }

        // 3. 分析Events字段需求和使用的类型
        if let Some(events) = &idl.events {
            log::debug!("🎉 发现 {} 个events", events.len());
            for event in events {
                let event_name = &event.name;
                log::debug!("  - Event: {}", event_name);
                
                if let Some(event_fields) = &event.fields {
                    // 事件有直接字段定义 - 这是实现关系，应从types中移除
                    log::debug!("    └─ 使用直接字段 ({} 个) - 实现关系", event_fields.len());
                    let field_defs = event_fields.iter().map(|field| FieldDefinition {
                        name: field.name.clone(),
                        field_type: Self::format_non_anchor_type(&field.field_type),
                        docs: field.docs.clone().unwrap_or_default(),
                    }).collect();
                    
                    allocation.events_fields.insert(event_name.clone(), field_defs);
                    allocation.events_used_types.insert(event_name.clone());
                    allocation.all_used_types.insert(event_name.clone());
                    // Event有直接字段定义，这是实现关系，应从types中移除
                    allocation.implemented_types.insert(event_name.clone());
                    
                    // 收集Events字段中引用的其他类型（这些类型应保留在types中）
                    for field in event_fields {
                        if let Some(type_name) = Self::extract_type_name_from_non_anchor_type(&field.field_type) {
                            if type_name != *event_name {  // 避免自引用
                                allocation.referenced_types.insert(type_name.clone());
                                allocation.all_used_types.insert(type_name.clone());
                                log::debug!("      → 字段引用类型: {}", type_name);
                            }
                        }
                    }
                } else {
                    // 事件没有字段定义，尝试从Types获取 - 这是引用关系，应保留在types中
                    let has_type_fields = allocation.types_remaining_fields.contains_key(event_name);
                    log::debug!("    └─ Types中有字段: {}", has_type_fields);
                    
                    if has_type_fields {
                        log::debug!("    └─ ✅ 从Types获取字段定义 - 引用关系");
                        allocation.events_used_types.insert(event_name.clone());
                        allocation.all_used_types.insert(event_name.clone());
                        // Event从types获取字段，这是引用关系，应保留在types中
                        allocation.referenced_types.insert(event_name.clone());
                        
                        if let Some(fields) = allocation.types_remaining_fields.get(event_name) {
                            log::debug!("    └─ 引用字段 {} 个", fields.len());
                            allocation.events_fields.insert(event_name.clone(), fields.clone());
                        }
                    } else {
                        // 无字段可用，创建空字段列表
                        log::debug!("    └─ ❌ 创建空字段列表");
                        allocation.events_fields.insert(event_name.clone(), Vec::new());
                    }
                }
            }
        }

        // 4. 分析Accounts字段需求和使用的类型
        if let Some(accounts) = &idl.accounts {
            log::debug!("🏦 发现 {} 个accounts", accounts.len());
            for account in accounts {
                let account_name = &account.name;
                log::debug!("  - Account: {}", account_name);
                
                if let Some(account_fields) = &account.fields {
                    // 账户有直接字段定义 - 这是实现关系，应从types中移除
                    log::debug!("    └─ 使用直接字段 ({} 个) - 实现关系", account_fields.len());
                    let field_defs = account_fields.iter().map(|field| FieldDefinition {
                        name: field.name.clone(),
                        field_type: Self::format_non_anchor_type(&field.field_type),
                        docs: field.docs.clone().unwrap_or_default(),
                    }).collect();
                    
                    allocation.accounts_fields.insert(account_name.clone(), field_defs);
                    allocation.accounts_used_types.insert(account_name.clone());
                    allocation.all_used_types.insert(account_name.clone());
                    // Account有直接字段定义，这是实现关系，应从types中移除
                    allocation.implemented_types.insert(account_name.clone());
                    
                    // 收集Accounts字段中引用的其他类型（这些类型应保留在types中）
                    for field in account_fields {
                        if let Some(type_name) = Self::extract_type_name_from_non_anchor_type(&field.field_type) {
                            if type_name != *account_name {  // 避免自引用
                                allocation.referenced_types.insert(type_name.clone());
                                allocation.all_used_types.insert(type_name.clone());
                                log::debug!("      → 字段引用类型: {}", type_name);
                            }
                        }
                    }
                } else {
                    // 账户没有字段定义，尝试从Types获取 - 这是引用关系，应保留在types中
                    let has_type_fields = allocation.types_remaining_fields.contains_key(account_name);
                    log::debug!("    └─ Types中有字段: {}", has_type_fields);
                    
                    if has_type_fields {
                        log::debug!("    └─ ✅ 从Types获取字段定义 - 引用关系");
                        allocation.accounts_used_types.insert(account_name.clone());
                        allocation.all_used_types.insert(account_name.clone());
                        // Account从types获取字段，这是引用关系，应保留在types中
                        allocation.referenced_types.insert(account_name.clone());
                        
                        if let Some(fields) = allocation.types_remaining_fields.get(account_name) {
                            log::debug!("    └─ 引用字段 {} 个", fields.len());
                            allocation.accounts_fields.insert(account_name.clone(), fields.clone());
                        }
                    } else {
                        // 无字段定义，创建空字段列表
                        log::debug!("    └─ ❌ 创建空字段列表");
                        allocation.accounts_fields.insert(account_name.clone(), Vec::new());
                    }
                }
            }
        }

        // 5. 智能移除类型定义 - 只移除被实现的类型，保留被引用的类型
        for implemented_type in &allocation.implemented_types {
            allocation.types_remaining_fields.remove(implemented_type);
            log::debug!("🗑️ 从types移除被实现的类型: {}", implemented_type);
        }
        
        for referenced_type in &allocation.referenced_types {
            log::debug!("🔗 保留在types中的被引用类型: {}", referenced_type);
        }

        log::debug!("✅ NonAnchor IDL字段分配完成 - 智能分配结果:");
        log::debug!("  Events: {} 个", allocation.events_fields.len());
        log::debug!("  Accounts: {} 个", allocation.accounts_fields.len());
        log::debug!("  被引用的类型: {:?}", allocation.referenced_types);
        log::debug!("  被实现的类型: {:?}", allocation.implemented_types);
        log::debug!("  Types模块保留: {:?}", allocation.types_remaining_fields.keys().collect::<Vec<_>>());
        
        allocation
    }
}

/// 选择性字段分配策略
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldAllocationStrategy {
    /// Anchor策略：智能字段分配和排除
    Anchor,
    /// NonAnchor策略：直接IDL字段读取
    NonAnchor,
}