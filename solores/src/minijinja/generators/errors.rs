//! 错误文件生成器
//! 
//! 负责生成错误模块文件

use crate::error::SoloresError;
use minijinja::{Environment, Value};
use std::fs;
use std::path::Path;

/// 生成错误模块
pub fn generate_errors_single_file(
    env: &mut Environment,
    src_dir: &Path,
    context: &Value,
) -> std::result::Result<(), SoloresError> {
    // 使用include_str!直接包含模板内容
    let template_content = include_str!("../templates/common/errors.rs.jinja");
    
    let tmpl = env.template_from_str(template_content)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("common/errors.rs.jinja".to_string()),
            message: format!("模板解析失败: {}", e),
            context: Some("解析错误模板".to_string()),
        })?;
    
    let rendered = tmpl.render(context)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("common/errors.rs.jinja".to_string()),
            message: format!("模板渲染失败: {}", e),
            context: Some("渲染错误模板".to_string()),
        })?;
    
    let output_path = src_dir.join("errors.rs");
    fs::write(&output_path, rendered)
        .map_err(|e| SoloresError::FileOperationError {
            operation: "write errors file".to_string(),
            path: output_path.display().to_string(),
            current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
            resolved_path: None,
            source: e,
            suggestion: Some("检查文件权限".to_string()),
        })?;
    
    Ok(())
}

/// 生成Cargo.toml文件的后备方案
pub fn generate_errors_single_file_fallback(
    env: &mut Environment,
    src_dir: &Path,
    context: &Value,
) -> std::result::Result<(), SoloresError> {
    // 如果模板文件不存在的后备方案
    let template_content = {
            r#"//! {{ crate_name }} Errors
//! Error types for the interface

#[derive(thiserror::Error, Debug)]
pub enum InterfaceError {
    #[error("IDL parsing error: {0}")]
    IdlParsingError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
}
"#.to_string()
    };
    
    let tmpl = env.template_from_str(&template_content)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("common/errors.rs.jinja fallback".to_string()),
            message: format!("模板解析失败: {}", e),
            context: Some("解析错误模板".to_string()),
        })?;
    
    let rendered = tmpl.render(context)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("common/errors.rs.jinja fallback".to_string()),
            message: format!("模板渲染失败: {}", e),
            context: Some("渲染错误模板".to_string()),
        })?;
    
    let output_path = src_dir.join("errors.rs");
    fs::write(&output_path, rendered)
        .map_err(|e| SoloresError::FileOperationError {
            operation: "write errors file".to_string(),
            path: output_path.display().to_string(),
            current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
            resolved_path: None,
            source: e,
            suggestion: Some("检查文件权限".to_string()),
        })?;
    
    Ok(())
}