//! 类型文件生成器
//! 
//! 负责生成类型相关的文件夹结构和单个类型文件

use crate::error::SoloresError;
use minijinja::{context, Environment, Value};
use std::fs;
use std::path::Path;
use convert_case::{Case, Casing};

/// 生成types文件夹和每个类型文件
pub fn generate_types_folder(
    env: &mut Environment,
    src_dir: &Path,
    context: &Value,
    template_type: &str,
) -> std::result::Result<(), SoloresError> {
    // 获取types数据
    let types = context.get_attr("types").unwrap_or(Value::UNDEFINED);
    
    // 检查是否有types数据
    if types == Value::UNDEFINED || types.len().unwrap_or(0) == 0 {
        return Ok(());
    }
    
    // 创建types目录
    let types_dir = src_dir.join("types");
    fs::create_dir_all(&types_dir).map_err(|e| SoloresError::FileOperationError {
        operation: "create types directory".to_string(),
        path: types_dir.display().to_string(),
        current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
        resolved_path: None,
        source: e,
        suggestion: Some("检查目录权限".to_string()),
    })?;
    
    // 收集类型文件名用于mod.rs
    let mut type_names = Vec::new();
    
    // 为每个类型生成单独文件
    for i in 0..types.len().unwrap_or(0) {
        if let Ok(type_def) = types.get_item(&Value::from(i)) {
            if type_def != Value::UNDEFINED {
                if let Ok(name_value) = type_def.get_attr("name") {
                    if let Some(type_name) = name_value.as_str() {
                        let filename = struct_name_to_filename(type_name);
                        type_names.push(filename.trim_end_matches(".rs").to_string());
                        
                        // 创建单个类型上下文 - 使用官方API
                        let type_context = context! {
                            type_def => type_def.clone(),
                            crate_name => context.get_attr("crate_name").unwrap_or(Value::from("")),
                            has_serde => context.get_attr("has_serde").unwrap_or(Value::from(false))
                        };
                        
                        // 生成类型文件
                        generate_single_type_file(env, &types_dir, &type_context, template_type, &filename)?;
                    }
                }
            }
        }
    }
    
    // 生成types/mod.rs
    super::common::generate_folder_mod_file(env, &types_dir, &type_names, "types", template_type)?;
    
    Ok(())
}

/// 生成单个类型文件
pub fn generate_single_type_file(
    env: &mut Environment,
    folder: &Path,
    context: &Value,
    template_type: &str,
    filename: &str,
) -> std::result::Result<(), SoloresError> {
    let template_content = if template_type == "anchor" {
        include_str!("../templates/anchor/types/single_type.rs.jinja")
    } else {
        include_str!("../templates/non_anchor/types/single_type.rs.jinja")
    };
    
    let tmpl = env.template_from_str(template_content)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("types/single_type.rs.jinja".to_string()),
            message: format!("模板解析失败: {}", e),
            context: Some("解析单个类型模板".to_string()),
        })?;
    
    let rendered = tmpl.render(context)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("types/single_type.rs.jinja".to_string()),
            message: format!("模板渲染失败: {}", e),
            context: Some("渲染单个类型模板".to_string()),
        })?;
    
    let output_path = folder.join(filename);
    fs::write(&output_path, rendered)
        .map_err(|e| SoloresError::FileOperationError {
            operation: "write single type file".to_string(),
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