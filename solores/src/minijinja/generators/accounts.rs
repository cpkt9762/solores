//! 账户文件生成器
//! 
//! 负责生成账户相关的文件夹结构和单个账户文件

use crate::error::SoloresError;
use minijinja::{context, Environment, Value};
use std::fs;
use std::path::Path;
use convert_case::{Case, Casing};
use log;

/// 生成accounts文件夹和每个账户文件 - 简化版本
pub fn generate_accounts_folder(
    env: &mut Environment,
    src_dir: &Path,
    context: &Value,
    template_type: &str,
) -> std::result::Result<(), SoloresError> {
    // 获取accounts数据
    let accounts = context.get_attr("accounts").unwrap_or(Value::UNDEFINED);
    
    // 添加调试信息
    log::debug!("🔍 generate_accounts_folder: accounts = {:?}", accounts);
    log::debug!("🔍 generate_accounts_folder: accounts.len() = {:?}", accounts.len());
    log::debug!("🔍 generate_accounts_folder: accounts == Value::UNDEFINED = {}", accounts == Value::UNDEFINED);
    
    // 修复条件检查：正确检查Vec长度
    if accounts == Value::UNDEFINED {
        log::debug!("❌ accounts数据为UNDEFINED，跳过生成");
        return Ok(());
    }
    
    let accounts_len = accounts.len().unwrap_or(0);
    if accounts_len == 0 {
        log::debug!("❌ accounts数据为空，跳过生成");
        return Ok(());
    }
    
    log::debug!("✅ 找到 {} 个accounts，开始生成目录", accounts_len);
    
    // 创建accounts目录
    let accounts_dir = src_dir.join("accounts");
    fs::create_dir_all(&accounts_dir).map_err(|e| SoloresError::FileOperationError {
        operation: "create accounts directory".to_string(),
        path: accounts_dir.display().to_string(),
        current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
        resolved_path: None,
        source: e,
        suggestion: Some("检查目录权限".to_string()),
    })?;
    
    // 收集账户文件名用于mod.rs
    let mut account_names = Vec::new();
    
    // 为每个账户生成单独文件
    for i in 0..accounts.len().unwrap_or(0) {
        if let Ok(account) = accounts.get_item(&Value::from(i)) {
            if account != Value::UNDEFINED {
                if let Ok(name_value) = account.get_attr("name") {
                    if let Some(account_name) = name_value.as_str() {
                        let filename = struct_name_to_filename(account_name);
                        account_names.push(filename.trim_end_matches(".rs").to_string());
                        
                        // 创建单个账户上下文 - 使用官方API
                        let account_context = context! {
                            account => account.clone(),
                            crate_name => context.get_attr("crate_name").unwrap_or(Value::from("")),
                            has_serde => context.get_attr("has_serde").unwrap_or(Value::from(false))
                        };
                        
                        // 生成账户文件
                        generate_single_account_file(env, &accounts_dir, &account_context, template_type, &filename)?;
                    }
                }
            }
        }
    }
    
    // 生成accounts/mod.rs
    super::common::generate_folder_mod_file(env, &accounts_dir, &account_names, "accounts", template_type)?;
    
    Ok(())
}

/// 生成单个账户文件
pub fn generate_single_account_file(
    env: &mut Environment,
    folder: &Path,
    context: &Value,
    template_type: &str,
    filename: &str,
) -> std::result::Result<(), SoloresError> {
    let template_content = if template_type == "anchor" {
        include_str!("../templates/anchor/accounts/single_account.rs.jinja")
    } else {
        include_str!("../templates/non_anchor/accounts/single_account.rs.jinja")
    };
    
    let tmpl = env.template_from_str(template_content)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("accounts/single_account.rs.jinja".to_string()),
            message: format!("模板解析失败: {}", e),
            context: Some("解析单个账户模板".to_string()),
        })?;
    
    let rendered = tmpl.render(context)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("accounts/single_account.rs.jinja".to_string()),
            message: format!("模板渲染失败: {}", e),
            context: Some("渲染单个账户模板".to_string()),
        })?;
    
    let output_path = folder.join(filename);
    fs::write(&output_path, rendered)
        .map_err(|e| SoloresError::FileOperationError {
            operation: "write single account file".to_string(),
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