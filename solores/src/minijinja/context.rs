//! 模板上下文构建模块
//! 
//! 负责从 IDL 数据创建模板渲染所需的上下文数据

use crate::error::SoloresError;
use crate::idl_format::IdlFormatEnum;
use convert_case::{Case, Casing};
use minijinja::{context, Value};
use log;

use super::builders::{anchor, non_anchor};
use super::builders::anchor::TypeTraitRegistry;

/// 创建模板上下文
pub fn create_template_context(
    idl_enum: &IdlFormatEnum,
    program_name: &str,
    serde_feature: bool,
    generate_parser: bool,
    no_empty_workspace: bool,
) -> std::result::Result<Value, SoloresError> {
    // 从IDL中提取实际数据
    let (accounts, instructions, events, types) = extract_idl_data(idl_enum)?;
    
    // 获取程序ID
    let program_id = match idl_enum {
        IdlFormatEnum::Anchor(anchor_idl) => &anchor_idl.address,
        IdlFormatEnum::NonAnchor(non_anchor_idl) => &non_anchor_idl.address,
    };
    
    // 获取当前时间戳
    let generation_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    
    // 使用官方context!宏构建上下文
    let context = context! {
        features => if serde_feature { vec!["serde".to_string()] } else { Vec::<String>::new() },
        has_serde => serde_feature,
        generate_parser => generate_parser,
        has_parsers => generate_parser,
        no_empty_workspace => no_empty_workspace,
        crate_name => program_name,
        program_name => program_name.to_case(Case::Pascal),
        program_id => program_id,
        generation_time => generation_time,
        accounts => accounts,
        instructions => instructions, 
        events => events,
        types => types,
        has_accounts => !accounts.is_empty(),
        has_instructions => !instructions.is_empty(),
        has_events => !events.is_empty(),
        has_types => !types.is_empty()
    };
    
    Ok(context)
}

/// 从IDL中提取数据 - 修复数据分类错误和字段丢失问题
pub fn extract_idl_data(
    idl_enum: &IdlFormatEnum,
) -> std::result::Result<(Vec<Value>, Vec<Value>, Vec<Value>, Vec<Value>), SoloresError> {
    match idl_enum {
        IdlFormatEnum::Anchor(anchor_idl) => {
            log::debug!("🔍 开始提取Anchor IDL数据 - 使用类型注册表修复版本");
            
            // 首先构建类型注册表，用于准确的trait支持检查
            let type_registry = TypeTraitRegistry::build_from_idl(idl_enum);
            log::debug!("📊 类型注册表构建完成，Copy支持: {} 个类型, Eq支持: {} 个类型", 
                       type_registry.copy_supported.len(), 
                       type_registry.eq_supported.len());
            
            // 直接从IDL构建各类数据，确保完整字段和正确分类
            let accounts: Vec<Value> = anchor_idl.accounts.as_ref().unwrap_or(&vec![])
                .iter()
                .map(|account| {
                    log::debug!("📋 处理Account: {}", account.name);
                    anchor::build_account_value(account, idl_enum)
                })
                .collect();
            
            let instructions: Vec<Value> = anchor_idl.instructions().unwrap_or(&vec![])
                .iter()
                .map(|instruction| {
                    log::debug!("📝 处理Instruction: {}", instruction.name);
                    anchor::build_instruction_value(instruction)
                })
                .collect();
            
            let events: Vec<Value> = anchor_idl.events.as_ref().unwrap_or(&vec![])
                .iter()
                .map(|event| {
                    log::debug!("🎯 处理Event: {}", event.name);
                    anchor::build_event_value(event, idl_enum)
                })
                .collect();
            
            // 收集所有被accounts/events/instructions使用的类型名称
            let mut used_type_names = std::collections::HashSet::new();
            
            // 收集accounts使用的类型
            for account in anchor_idl.accounts.as_ref().unwrap_or(&vec![]) {
                used_type_names.insert(account.name.clone());
            }
            
            // 收集events使用的类型
            for event in anchor_idl.events.as_ref().unwrap_or(&vec![]) {
                used_type_names.insert(event.name.clone());
            }
            
            // 只包含真正的types，排除已被accounts/events实现的类型
            let types: Vec<Value> = anchor_idl.types.as_ref().unwrap_or(&vec![])
                .iter()
                .filter(|type_def| {
                    let is_used = used_type_names.contains(&type_def.name);
                    if is_used {
                        log::debug!("🚫 排除已被实现的类型: {}", type_def.name);
                    } else {
                        log::debug!("✅ 保留纯类型: {}", type_def.name);
                    }
                    !is_used
                })
                .map(|type_def| {
                    log::debug!("🔧 处理Type: {} (使用注册表)", type_def.name);
                    anchor::build_type_value_with_registry(type_def, &type_registry)
                })
                .collect();
            
            log::debug!("📊 数据提取完成 - Accounts: {}, Instructions: {}, Events: {}, Types: {}", 
                       accounts.len(), instructions.len(), events.len(), types.len());
            
            Ok((accounts, instructions, events, types))
        },
        IdlFormatEnum::NonAnchor(non_anchor_idl) => {
            let accounts: Vec<Value> = non_anchor_idl.accounts.as_ref().unwrap_or(&vec![]).iter().map(|account| {
                non_anchor::build_non_anchor_account_value(account, idl_enum)
            }).collect();
            
            let instructions: Vec<Value> = non_anchor_idl.instructions().iter().enumerate().map(|(index, instruction)| {
                non_anchor::build_non_anchor_instruction_value(instruction, index)
            }).collect();
            
            let events: Vec<Value> = non_anchor_idl.events.as_ref().unwrap_or(&vec![]).iter().map(|event| {
                non_anchor::build_non_anchor_event_value(event)
            }).collect();
            
            let types: Vec<Value> = non_anchor_idl.types.as_ref().unwrap_or(&vec![]).iter().map(|type_def| {
                non_anchor::build_non_anchor_type_value(type_def)
            }).collect();
            
            Ok((accounts, instructions, events, types))
        }
    }
}