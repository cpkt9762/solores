//! 配置文件生成器
//! 
//! 负责生成 Cargo.toml 和 README.md 文件

use crate::error::SoloresError;
use minijinja::{Environment, Value};
use std::fs;
use std::path::Path;

/// 生成Cargo.toml文件
pub fn generate_cargo_toml(
    env: &mut Environment,
    output_dir: &Path,
    context: &Value,
) -> std::result::Result<(), SoloresError> {
    // 使用include_str!直接包含Cargo.toml模板
    let template_content = include_str!("../templates/common/Cargo.toml.jinja");
    
    let tmpl = env.template_from_str(template_content)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("common/Cargo.toml.jinja".to_string()),
            message: format!("模板解析失败: {}", e),
            context: Some("解析Cargo.toml模板".to_string()),
        })?;
    
    let rendered = tmpl.render(context)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("common/Cargo.toml.jinja".to_string()),
            message: format!("模板渲染失败: {}", e),
            context: Some("渲染Cargo.toml模板".to_string()),
        })?;
    
    let output_path = output_dir.join("Cargo.toml");
    fs::write(&output_path, rendered)
        .map_err(|e| SoloresError::FileOperationError {
            operation: "write Cargo.toml".to_string(),
            path: output_path.display().to_string(),
            current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
            resolved_path: None,
            source: e,
            suggestion: Some("检查文件权限".to_string()),
        })?;
    
    Ok(())
}

/// Cargo.toml生成的后备方案
pub fn generate_cargo_toml_fallback(
    env: &mut Environment,
    output_dir: &Path,
    context: &Value,
    program_name: &str,
) -> std::result::Result<(), SoloresError> {
    // 如果模板不存在的后备方案
    let template_content = {
            format!(r#"[package]
name = "sol_{}_interface"
version = "0.2.0"
edition = "2021"
description = "{{{{ crate_name }}}} 程序接口库，由 Solores 生成"
license = "MIT OR Apache-2.0"

[workspace]
# 空 workspace 表，防止被父目录 workspace 控制

[dependencies.borsh]
version = "^1.5"

[dependencies.num-derive]
version = "0.4.2"

[dependencies.num-traits]
version = "^0.2"

[dependencies.serde]
features = ["derive"]
optional = true
version = "^1.0"

[dependencies.serde-big-array]
optional = true
version = "^0.5"

[dependencies.serde_json]
optional = true
version = "^1.0"

[dependencies.serde_with]
optional = true
version = "^3.0"

[dependencies.solana-account-info]
optional = true
version = "2.3.0"

[dependencies.solana-cpi]
optional = true
version = "2.2.1"

[dependencies.solana-instruction]
version = "2.3.0"

[dependencies.solana-program-entrypoint]
optional = true
version = "2.3.0"

[dependencies.solana-program-error]
version = "2.2.2"

[dependencies.solana-pubkey]
features = ["borsh", "curve25519", "serde"]
version = "2.4.0"

[dependencies.thiserror]
version = "^1.0"

[features]
account-info = ["dep:solana-account-info"]
cpi = ["dep:solana-cpi"]
full-solana = ["account-info", "program-entrypoint", "cpi"]
program-entrypoint = ["dep:solana-program-entrypoint"]
serde = ["dep:serde", "dep:serde_with", "dep:serde-big-array", "dep:serde_json"]
"#, program_name)
    };
    
    let tmpl = env.template_from_str(&template_content)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("common/Cargo.toml.jinja fallback".to_string()),
            message: format!("模板解析失败: {}", e),
            context: Some("解析Cargo.toml模板".to_string()),
        })?;
    
    let rendered = tmpl.render(context)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("common/Cargo.toml.jinja fallback".to_string()),
            message: format!("模板渲染失败: {}", e),
            context: Some("渲染Cargo.toml模板".to_string()),
        })?;
    
    let output_path = output_dir.join("Cargo.toml");
    fs::write(&output_path, rendered)
        .map_err(|e| SoloresError::FileOperationError {
            operation: "write Cargo.toml".to_string(),
            path: output_path.display().to_string(),
            current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
            resolved_path: None,
            source: e,
            suggestion: Some("检查文件权限".to_string()),
        })?;
    
    Ok(())
}

/// 生成README.md文件
pub fn generate_readme(
    env: &mut Environment,
    output_dir: &Path,
    context: &Value,
) -> std::result::Result<(), SoloresError> {
    // 使用include_str!直接包含README模板
    let template_content = include_str!("../templates/common/readme.md.jinja");
    
    let tmpl = env.template_from_str(template_content)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("common/readme.md.jinja".to_string()),
            message: format!("模板解析失败: {}", e),
            context: Some("解析README模板".to_string()),
        })?;
    
    let rendered = tmpl.render(context)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("common/readme.md.jinja".to_string()),
            message: format!("模板渲染失败: {}", e),
            context: Some("渲染README模板".to_string()),
        })?;
    
    let output_path = output_dir.join("README.md");
    fs::write(&output_path, rendered)
        .map_err(|e| SoloresError::FileOperationError {
            operation: "write README.md".to_string(),
            path: output_path.display().to_string(),
            current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
            resolved_path: None,
            source: e,
            suggestion: Some("检查文件权限".to_string()),
        })?;
    
    Ok(())
}

/// README生成的后备方案
pub fn generate_readme_fallback(
    env: &mut Environment,
    output_dir: &Path,
    context: &Value,
    program_name: &str,
) -> std::result::Result<(), SoloresError> {
    // 如果模板不存在的后备方案
    let template_content = {
            format!(r#"# {{{{ crate_name }}}} Interface

Auto-generated Solana program interface library created by [Solores](https://github.com/your-repo/solores).

## Usage

```rust
use sol_{}_interface::{{id, accounts::*, instructions::*, types::*}};

// Get program ID
let program_id = id();

// Use generated types and instructions
```

## Features

- `serde`: Enable JSON serialization support
- `account-info`: Include Solana account info dependencies
- `cpi`: Include Cross-Program Invocation support
- `full-solana`: Enable all Solana-related features

## Generated with

This interface was generated using Solores IDL-to-Rust interface generator.
"#, program_name)
    };
    
    let tmpl = env.template_from_str(&template_content)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("common/readme.md.jinja fallback".to_string()),
            message: format!("模板解析失败: {}", e),
            context: Some("解析README模板".to_string()),
        })?;
    
    let rendered = tmpl.render(context)
        .map_err(|e| SoloresError::TemplateError {
            template_name: Some("common/readme.md.jinja fallback".to_string()),
            message: format!("模板渲染失败: {}", e),
            context: Some("渲染README模板".to_string()),
        })?;
    
    let output_path = output_dir.join("README.md");
    fs::write(&output_path, rendered)
        .map_err(|e| SoloresError::FileOperationError {
            operation: "write README.md".to_string(),
            path: output_path.display().to_string(),
            current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
            resolved_path: None,
            source: e,
            suggestion: Some("检查文件权限".to_string()),
        })?;
    
    Ok(())
}