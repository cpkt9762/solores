//! 事件文件生成器
//! 
//! 负责生成事件相关的文件夹结构和单个事件文件

use crate::error::SoloresError;
use minijinja::{context, Environment, Value};
use std::fs;
use std::path::Path;
use convert_case::{Case, Casing};
use log;

/// 生成events文件夹和每个事件文件
pub fn generate_events_folder(
    env: &mut Environment,
    src_dir: &Path,
    context: &Value,
    template_type: &str,
) -> std::result::Result<(), SoloresError> {
    // 获取events数据
    let events = context.get_attr("events").unwrap_or(Value::UNDEFINED);
    
    // 添加调试信息
    log::debug!("🔍 generate_events_folder: events = {:?}", events);
    log::debug!("🔍 generate_events_folder: events.len() = {:?}", events.len());
    log::debug!("🔍 generate_events_folder: events == Value::UNDEFINED = {}", events == Value::UNDEFINED);
    
    // 修复条件检查：总是生成events目录，包括空情况
    let events_len = if events == Value::UNDEFINED {
        log::debug!("🔍 events数据为UNDEFINED，将生成空的events目录");
        0
    } else {
        events.len().unwrap_or(0)
    };
    
    if events_len == 0 {
        log::debug!("✅ events数据为空，将生成空的events目录和空的mod.rs");
    } else {
        log::debug!("✅ 找到 {} 个events，开始生成目录", events_len);
    }
    
    // 创建events目录
    let events_dir = src_dir.join("events");
    fs::create_dir_all(&events_dir).map_err(|e| SoloresError::FileOperationError {
        operation: "create events directory".to_string(),
        path: events_dir.display().to_string(),
        current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
        resolved_path: None,
        source: e,
        suggestion: Some("检查目录权限".to_string()),
    })?;
    
    // 收集事件文件名用于mod.rs
    let mut event_names = Vec::new();
    
    // 为每个事件生成单独文件（仅当events_len > 0时）
    if events_len > 0 {
        for i in 0..events_len {
            if let Ok(event) = events.get_item(&Value::from(i)) {
                if event != Value::UNDEFINED {
                    if let Ok(name_value) = event.get_attr("name") {
                        if let Some(event_name) = name_value.as_str() {
                            let filename = struct_name_to_filename(event_name);
                            event_names.push(filename.trim_end_matches(".rs").to_string());
                            
                            // 创建单个事件上下文 - 使用官方API
                            let event_context = context! {
                                event => event.clone(),
                                crate_name => context.get_attr("crate_name").unwrap_or(Value::from("")),
                                has_serde => context.get_attr("has_serde").unwrap_or(Value::from(false))
                            };
                            
                            // 生成事件文件
                            generate_single_event_file(env, &events_dir, &event_context, template_type, &filename)?;
                        }
                    }
                }
            }
        }
    }
    
    // 生成events/mod.rs
    super::common::generate_folder_mod_file(env, &events_dir, &event_names, "events", template_type)?;
    
    Ok(())
}

/// 生成单个事件文件
pub fn generate_single_event_file(
    env: &mut Environment,
    folder: &Path,
    context: &Value,
    template_type: &str,
    filename: &str,
) -> std::result::Result<(), SoloresError> {
    let template_content = if template_type == "anchor" {
        include_str!("../templates/anchor/events/single_event.rs.jinja")
    } else {
        include_str!("../templates/non_anchor/events/single_event.rs.jinja")
    };
    
    let tmpl = env.template_from_str(template_content)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("events/single_event.rs.jinja".to_string()),
            message: format!("模板解析失败: {}", e),
            context: Some("解析单个事件模板".to_string()),
        })?;
    
    let rendered = tmpl.render(context)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("events/single_event.rs.jinja".to_string()),
            message: format!("模板渲染失败: {}", e),
            context: Some("渲染单个事件模板".to_string()),
        })?;
    
    let output_path = folder.join(filename);
    fs::write(&output_path, rendered)
        .map_err(|e| SoloresError::FileOperationError {
            operation: "write single event file".to_string(),
            path: output_path.display().to_string(),
            current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
            resolved_path: None,
            source: e,
            suggestion: Some("检查文件权限".to_string()),
        })?;
    
    Ok(())
}

/// 结构体名到文件名转换
fn struct_name_to_filename(name: &str) -> String {
    name.to_case(Case::Snake) + ".rs"
}