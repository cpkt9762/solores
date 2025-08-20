//! 解析器文件生成器
//! 
//! 负责生成解析器相关的文件夹结构和解析器文件

use crate::error::SoloresError;
use minijinja::{Environment, Value};
use std::fs;
use std::path::Path;

/// 生成parsers文件夹结构
pub fn generate_parsers_folder(
    env: &mut Environment,
    src_dir: &Path,
    context: &Value,
    template_type: &str,
) -> std::result::Result<(), SoloresError> {
    // 创建parsers目录
    let parsers_dir = src_dir.join("parsers");
    fs::create_dir_all(&parsers_dir).map_err(|e| SoloresError::FileOperationError {
        operation: "create parsers directory".to_string(),
        path: parsers_dir.display().to_string(),
        current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
        resolved_path: None,
        source: e,
        suggestion: Some("检查目录权限".to_string()),
    })?;

    // 准备解析器模块列表 - 总是包含events模块
    let parser_modules = vec!["accounts".to_string(), "instructions".to_string(), "events".to_string()];
    
    // 生成parsers/mod.rs
    super::common::generate_folder_mod_file(env, &parsers_dir, &parser_modules, "parsers", template_type)?;
    
    // 生成parsers/accounts.rs文件
    generate_parsers_accounts_file(env, &parsers_dir, context, template_type)?;
    
    // 生成parsers/instructions.rs文件
    generate_parsers_instructions_file(env, &parsers_dir, context, template_type)?;
    
    // 生成parsers/events.rs文件（总是生成，包括空events情况）
    generate_parsers_events_file(env, &parsers_dir, context, template_type)?;
    
    Ok(())
}

/// 生成parsers/mod.rs文件
pub fn generate_parsers_mod_file(
    env: &mut Environment,
    parsers_dir: &Path,
    context: &Value,
    template_type: &str,
) -> std::result::Result<(), SoloresError> {
    let template_content = if template_type == "anchor" {
        include_str!("../templates/anchor/parsers/mod.rs.jinja")
    } else {
        include_str!("../templates/non_anchor/parsers/mod.rs.jinja")
    };
    
    let tmpl = env.template_from_str(template_content)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("parsers/mod.rs.jinja".to_string()),
            message: format!("模板解析失败: {}", e),
            context: Some("解析parsers mod模板".to_string()),
        })?;
    
    let rendered = tmpl.render(context)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("parsers/mod.rs.jinja".to_string()),
            message: format!("模板渲染失败: {}", e),
            context: Some("渲染parsers mod模板".to_string()),
        })?;
    
    let output_path = parsers_dir.join("mod.rs");
    fs::write(&output_path, rendered)
        .map_err(|e| SoloresError::FileOperationError {
            operation: "write parsers mod file".to_string(),
            path: output_path.display().to_string(),
            current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
            resolved_path: None,
            source: e,
            suggestion: Some("检查文件权限".to_string()),
        })?;
    
    Ok(())
}

/// 生成parsers/instructions.rs文件
pub fn generate_parsers_instructions_file(
    env: &mut Environment,
    parsers_dir: &Path,
    context: &Value,
    template_type: &str,
) -> std::result::Result<(), SoloresError> {
    let template_content = if template_type == "anchor" {
        include_str!("../templates/anchor/parsers/instructions.rs.jinja")
    } else {
        include_str!("../templates/non_anchor/parsers/instructions.rs.jinja")
    };
    
    let tmpl = env.template_from_str(template_content)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("parsers/instructions.rs.jinja".to_string()),
            message: format!("模板解析失败: {}", e),
            context: Some("解析parsers instructions模板".to_string()),
        })?;
    
    let rendered = tmpl.render(context)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("parsers/instructions.rs.jinja".to_string()),
            message: format!("模板渲染失败: {}", e),
            context: Some("渲染parsers instructions模板".to_string()),
        })?;
    
    let output_path = parsers_dir.join("instructions.rs");
    fs::write(&output_path, rendered)
        .map_err(|e| SoloresError::FileOperationError {
            operation: "write parsers instructions file".to_string(),
            path: output_path.display().to_string(),
            current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
            resolved_path: None,
            source: e,
            suggestion: Some("检查文件权限".to_string()),
        })?;
    
    Ok(())
}

/// 生成parsers/accounts.rs文件
pub fn generate_parsers_accounts_file(
    env: &mut Environment,
    parsers_dir: &Path,
    context: &Value,
    template_type: &str,
) -> std::result::Result<(), SoloresError> {
    let template_content = if template_type == "anchor" {
        include_str!("../templates/anchor/parsers/accounts.rs.jinja")
    } else {
        include_str!("../templates/non_anchor/parsers/accounts.rs.jinja")
    };
    
    let tmpl = env.template_from_str(template_content)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("parsers/accounts.rs.jinja".to_string()),
            message: format!("模板解析失败: {}", e),
            context: Some("解析parsers accounts模板".to_string()),
        })?;
    
    let rendered = tmpl.render(context)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("parsers/accounts.rs.jinja".to_string()),
            message: format!("模板渲染失败: {}", e),
            context: Some("渲染parsers accounts模板".to_string()),
        })?;
    
    let output_path = parsers_dir.join("accounts.rs");
    fs::write(&output_path, rendered)
        .map_err(|e| SoloresError::FileOperationError {
            operation: "write parsers accounts file".to_string(),
            path: output_path.display().to_string(),
            current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
            resolved_path: None,
            source: e,
            suggestion: Some("检查文件权限".to_string()),
        })?;
    
    Ok(())
}

/// 生成parsers/events.rs文件
pub fn generate_parsers_events_file(
    env: &mut Environment,
    parsers_dir: &Path,
    context: &Value,
    template_type: &str,
) -> std::result::Result<(), SoloresError> {
    let template_content = if template_type == "anchor" {
        include_str!("../templates/anchor/parsers/events.rs.jinja")
    } else {
        include_str!("../templates/non_anchor/parsers/events.rs.jinja")
    };
    
    let tmpl = env.template_from_str(template_content)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("parsers/events.rs.jinja".to_string()),
            message: format!("模板解析失败: {}", e),
            context: Some("解析parsers events模板".to_string()),
        })?;
    
    let rendered = tmpl.render(context)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("parsers/events.rs.jinja".to_string()),
            message: format!("模板渲染失败: {}", e),
            context: Some("渲染parsers events模板".to_string()),
        })?;
    
    let output_path = parsers_dir.join("events.rs");
    fs::write(&output_path, rendered)
        .map_err(|e| SoloresError::FileOperationError {
            operation: "write parsers events file".to_string(),
            path: output_path.display().to_string(),
            current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
            resolved_path: None,
            source: e,
            suggestion: Some("检查文件权限".to_string()),
        })?;
    
    Ok(())
}