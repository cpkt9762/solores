//! 指令文件生成器
//!
//! 负责生成指令相关的文件夹结构和单个指令文件

use crate::error::SoloresError;
use convert_case::{Case, Casing};
use minijinja::{context, Environment, Value};
use std::fs;
use std::path::Path;

/// 生成instructions文件夹和每个指令文件
pub fn generate_instructions_folder(
    env: &mut Environment,
    src_dir: &Path,
    context: &Value,
    template_type: &str,
) -> std::result::Result<(), SoloresError> {
    // 获取instructions数据
    let instructions = context.get_attr("instructions").unwrap_or(Value::UNDEFINED);

    // 检查是否有instructions数据
    if instructions == Value::UNDEFINED || instructions.len().unwrap_or(0) == 0 {
        return Ok(());
    }

    // 创建instructions目录
    let instructions_dir = src_dir.join("instructions");
    fs::create_dir_all(&instructions_dir).map_err(|e| SoloresError::FileOperationError {
        operation: "create instructions directory".to_string(),
        path: instructions_dir.display().to_string(),
        current_dir: std::env::current_dir()
            .ok()
            .map(|p| p.display().to_string()),
        resolved_path: None,
        source: e,
        suggestion: Some("检查目录权限".to_string()),
    })?;

    // 收集指令文件名用于mod.rs
    let mut instruction_names = Vec::new();

    // 为每个指令生成单独文件
    for i in 0..instructions.len().unwrap_or(0) {
        if let Ok(instruction) = instructions.get_item(&Value::from(i)) {
            if instruction != Value::UNDEFINED {
                if let Ok(name_value) = instruction.get_attr("name") {
                    if let Some(instruction_name) = name_value.as_str() {
                        let filename = struct_name_to_filename(instruction_name);
                        instruction_names.push(filename.trim_end_matches(".rs").to_string());

                        // 检测是否有remainingAccountsInfo参数
                        let has_remaining_accounts_info =
                            detect_remaining_accounts_info(&instruction);

                        // 调试日志：检查检测结果
                        if has_remaining_accounts_info {
                            log::debug!("🔍 检测到remainingAccountsInfo参数: {}", instruction_name);
                        }

                        // 创建单个指令上下文 - 使用官方API
                        let instruction_context = context! {
                            instruction => instruction.clone(),
                            crate_name => context.get_attr("crate_name").unwrap_or(Value::from("")),
                            has_serde => context.get_attr("has_serde").unwrap_or(Value::from(false)),
                            has_remaining_accounts_info => has_remaining_accounts_info
                        };

                        // 生成指令文件
                        generate_single_instruction_file(
                            env,
                            &instructions_dir,
                            &instruction_context,
                            template_type,
                            &filename,
                        )?;
                    }
                }
            }
        }
    }

    // 生成instructions/mod.rs
    super::common::generate_folder_mod_file(
        env,
        &instructions_dir,
        &instruction_names,
        "instructions",
        template_type,
    )?;

    Ok(())
}

/// 生成单个指令文件
pub fn generate_single_instruction_file(
    env: &mut Environment,
    folder: &Path,
    context: &Value,
    template_type: &str,
    filename: &str,
) -> std::result::Result<(), SoloresError> {
    let template_content = if template_type == "anchor" {
        include_str!("../templates/anchor/instructions/single_instruction.rs.jinja")
    } else {
        include_str!("../templates/non_anchor/instructions/single_instruction.rs.jinja")
    };

    let tmpl =
        env.template_from_str(template_content)
            .map_err(|e| SoloresError::TemplateError {
                template_name: Some("instructions/single_instruction.rs.jinja".to_string()),
                message: format!("模板解析失败: {}", e),
                context: Some("解析单个指令模板".to_string()),
            })?;

    let rendered = tmpl
        .render(context)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("instructions/single_instruction.rs.jinja".to_string()),
            message: format!("模板渲染失败: {}", e),
            context: Some("渲染单个指令模板".to_string()),
        })?;

    let output_path = folder.join(filename);
    fs::write(&output_path, rendered).map_err(|e| SoloresError::FileOperationError {
        operation: "write single instruction file".to_string(),
        path: output_path.display().to_string(),
        current_dir: std::env::current_dir()
            .ok()
            .map(|p| p.display().to_string()),
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

/// 检测指令是否包含remainingAccountsInfo参数
fn detect_remaining_accounts_info(instruction: &Value) -> bool {
    // 获取指令的args字段
    if let Ok(args) = instruction.get_attr("args") {
        // 遍历所有参数
        for i in 0..args.len().unwrap_or(0) {
            if let Ok(arg) = args.get_item(&Value::from(i)) {
                if let Ok(arg_name) = arg.get_attr("name") {
                    if let Some(name_str) = arg_name.as_str() {
                        // 检查参数名是否为remainingAccountsInfo或remaining_accounts_info
                        // 支持两种命名形式是因为IDL解析过程中可能进行了命名转换
                        if name_str == "remainingAccountsInfo"
                            || name_str == "remaining_accounts_info"
                        {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}
