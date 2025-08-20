//! 通用生成器函数
//! 
//! 提供在多个生成器中使用的通用功能

use crate::error::SoloresError;
use minijinja::{context, Environment, Value};
use std::fs;
use std::path::Path;

/// 生成文件夹的mod.rs
pub fn generate_folder_mod_file(
    env: &mut Environment,
    folder: &Path,
    items: &[String],
    module_name: &str,
    template_type: &str,
) -> std::result::Result<(), SoloresError> {
    let template_content = match (template_type, module_name) {
        ("anchor", "accounts") => include_str!("../templates/anchor/accounts/mod.rs.jinja"),
        ("anchor", "instructions") => include_str!("../templates/anchor/instructions/mod.rs.jinja"),
        ("anchor", "events") => include_str!("../templates/anchor/events/mod.rs.jinja"),
        ("anchor", "types") => include_str!("../templates/anchor/types/mod.rs.jinja"),
        ("anchor", "parsers") => include_str!("../templates/anchor/parsers/mod.rs.jinja"),
        ("non_anchor", "accounts") => include_str!("../templates/non_anchor/accounts/mod.rs.jinja"),
        ("non_anchor", "instructions") => include_str!("../templates/non_anchor/instructions/mod.rs.jinja"),
        ("non_anchor", "events") => include_str!("../templates/non_anchor/events/mod.rs.jinja"),
        ("non_anchor", "types") => include_str!("../templates/non_anchor/types/mod.rs.jinja"),
        ("non_anchor", "parsers") => include_str!("../templates/non_anchor/parsers/mod.rs.jinja"),
        _ => include_str!("../templates/anchor/accounts/mod.rs.jinja"), // 默认
    };
    
    // 创建mod.rs上下文 - 使用官方API
    let mod_context = context! {
        module_name => module_name,
        items => items,
        crate_name => ""
    };
    
    let tmpl = env.template_from_str(template_content)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some(format!("{}/mod.rs.jinja", module_name)),
            message: format!("模板解析失败: {}", e),
            context: Some("解析folder mod模板".to_string()),
        })?;
    
    let rendered = tmpl.render(&mod_context)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some(format!("{}/mod.rs.jinja", module_name)),
            message: format!("模板渲染失败: {}", e),
            context: Some("渲染folder mod模板".to_string()),
        })?;
    
    let output_path = folder.join("mod.rs");
    fs::write(&output_path, rendered)
        .map_err(|e| SoloresError::FileOperationError {
            operation: "write folder mod file".to_string(),
            path: output_path.display().to_string(),
            current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
            resolved_path: None,
            source: e,
            suggestion: Some("检查文件权限".to_string()),
        })?;
    
    Ok(())
}

/// 生成多文件夹模式的lib.rs
pub fn generate_lib_multi_folder(
    env: &mut Environment,
    src_dir: &Path,
    context: &Value,
    template_type: &str,
) -> std::result::Result<(), SoloresError> {
    let template_content = if template_type == "anchor" {
        include_str!("../templates/anchor/lib.rs.jinja")
    } else {
        include_str!("../templates/non_anchor/lib.rs.jinja")
    };
    
    let tmpl = env.template_from_str(template_content)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("lib.rs.jinja".to_string()),
            message: format!("模板解析失败: {}", e),
            context: Some("解析lib模板".to_string()),
        })?;
    
    let rendered = tmpl.render(context)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("lib.rs.jinja".to_string()),
            message: format!("模板渲染失败: {}", e),
            context: Some("渲染lib模板".to_string()),
        })?;
    
    let output_path = src_dir.join("lib.rs");
    fs::write(&output_path, rendered)
        .map_err(|e| SoloresError::FileOperationError {
            operation: "write lib file".to_string(),
            path: output_path.display().to_string(),
            current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
            resolved_path: None,
            source: e,
            suggestion: Some("检查文件权限".to_string()),
        })?;
    
    Ok(())
}