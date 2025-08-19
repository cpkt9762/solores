use crate::error::SoloresError;
use crate::idl_format::IdlFormatEnum;
use convert_case::{Case, Casing};
use log::{debug, info};
use minijinja::{context, Environment, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// MiniJinja模板生成器
pub struct MinijinjaTemplateGenerator {
    idl_enum: IdlFormatEnum,
    env: Environment<'static>,
}

impl MinijinjaTemplateGenerator {
    /// 创建新的MiniJinja生成器实例
    pub fn new(idl_enum: IdlFormatEnum) -> std::result::Result<Self, SoloresError> {
        let mut env = Environment::new();
        
        // 添加自定义过滤器
        env.add_filter("snake_case", to_snake_case_filter);
        env.add_filter("upper", |value: String| -> String { value.to_uppercase() });
        env.add_filter("lower", |value: String| -> String { value.to_lowercase() });
        env.add_filter("type_path", type_path_filter);
        env.add_filter("rust_field", rust_field_filter);
        env.add_filter("starts_with", starts_with_filter);
        env.add_filter("multiline_docs", multiline_docs_filter);
        
        Ok(Self { idl_enum, env })
    }
    
    /// 生成多文件夹架构的完整Rust代码
    pub fn generate_multi_file_structure(
        &mut self,
        output_dir: &Path,
        serde_feature: bool,
        generate_parser: bool,
    ) -> std::result::Result<(), SoloresError> {
        info!("开始使用MiniJinja生成多文件夹架构");
        
        // 创建源代码目录结构
        let src_dir = output_dir.join("src");
        fs::create_dir_all(&src_dir).map_err(|e| SoloresError::FileOperationError {
            operation: "create src directory".to_string(),
            path: src_dir.display().to_string(),
            current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
            resolved_path: None,
            source: e,
            suggestion: Some("检查目录权限".to_string()),
        })?;
        
        // 创建模板上下文
        let context = self.create_template_context(&self.idl_enum, serde_feature, generate_parser)?;
        
        // 确定使用的模板类型
        let template_type = if self.is_anchor_idl() { "anchor" } else { "non_anchor" };
        
        // 生成多文件夹模块
        self.generate_accounts_folder(&src_dir, &context, template_type)?;
        self.generate_instructions_folder(&src_dir, &context, template_type)?;
        self.generate_events_folder(&src_dir, &context, template_type)?;
        self.generate_types_folder(&src_dir, &context, template_type)?;
        if generate_parser {
            self.generate_parsers_folder(&src_dir, &context, template_type)?;
        }
        self.generate_errors_single_file(&src_dir, &context)?;
        self.generate_lib_multi_folder(&src_dir, &context, template_type)?;
        
        // 生成配置文件
        self.generate_cargo_toml(output_dir, &context)?;
        self.generate_readme(output_dir, &context)?;
        
        info!("MiniJinja多文件夹架构生成完成");
        Ok(())
    }
    
    /// 检查是否是Anchor IDL
    fn is_anchor_idl(&self) -> bool {
        matches!(self.idl_enum, IdlFormatEnum::Anchor(_))
    }

    /// 获取程序名称
    fn get_program_name(&self) -> &str {
        match &self.idl_enum {
            IdlFormatEnum::Anchor(anchor_idl) => anchor_idl.program_name(),
            IdlFormatEnum::NonAnchor(non_anchor_idl) => non_anchor_idl.program_name(),
        }
    }

    /// 结构体名到文件名转换
    fn struct_name_to_filename(name: &str) -> String {
        name.to_case(Case::Snake) + ".rs"
    }

    /// 创建模板上下文
    fn create_template_context(
        &self,
        idl_enum: &IdlFormatEnum,
        serde_feature: bool,
        generate_parser: bool,
    ) -> std::result::Result<Value, SoloresError> {
        // 从IDL中提取实际数据
        let (accounts, instructions, events, types) = self.extract_idl_data(idl_enum)?;
        
        // 获取程序ID
        let program_id = match idl_enum {
            IdlFormatEnum::Anchor(anchor_idl) => &anchor_idl.address,
            IdlFormatEnum::NonAnchor(non_anchor_idl) => &non_anchor_idl.address,
        };
        
        // 使用官方context!宏构建上下文
        let context = context! {
            features => if serde_feature { vec!["serde".to_string()] } else { Vec::<String>::new() },
            has_serde => serde_feature,
            generate_parser => generate_parser,
            has_parsers => generate_parser,
            crate_name => self.get_program_name(),
            program_name => self.get_program_name().to_case(Case::Pascal),
            program_id => program_id,
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
    fn extract_idl_data(
        &self,
        idl_enum: &IdlFormatEnum,
    ) -> std::result::Result<(Vec<Value>, Vec<Value>, Vec<Value>, Vec<Value>), SoloresError> {
        match idl_enum {
            IdlFormatEnum::Anchor(anchor_idl) => {
                log::debug!("🔍 开始提取Anchor IDL数据 - 修复版本");
                
                // 直接从IDL构建各类数据，确保完整字段和正确分类
                let accounts: Vec<Value> = anchor_idl.accounts.as_ref().unwrap_or(&vec![])
                    .iter()
                    .map(|account| {
                        log::debug!("📋 处理Account: {}", account.name);
                        self.build_account_value(account)
                    })
                    .collect();
                
                let instructions: Vec<Value> = anchor_idl.instructions().unwrap_or(&vec![])
                    .iter()
                    .map(|instruction| {
                        log::debug!("📝 处理Instruction: {}", instruction.name);
                        self.build_instruction_value(instruction)
                    })
                    .collect();
                
                let events: Vec<Value> = anchor_idl.events.as_ref().unwrap_or(&vec![])
                    .iter()
                    .map(|event| {
                        log::debug!("🎯 处理Event: {}", event.name);
                        self.build_event_value(event)
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
                        log::debug!("🔧 处理Type: {}", type_def.name);
                        self.build_type_value(type_def)
                    })
                    .collect();
                
                log::debug!("📊 数据提取完成 - Accounts: {}, Instructions: {}, Events: {}, Types: {}", 
                           accounts.len(), instructions.len(), events.len(), types.len());
                
                Ok((accounts, instructions, events, types))
            },
            IdlFormatEnum::NonAnchor(non_anchor_idl) => {
                let accounts: Vec<Value> = non_anchor_idl.accounts.as_ref().unwrap_or(&vec![]).iter().map(|account| {
                    self.build_non_anchor_account_value(account)
                }).collect();
                
                let instructions: Vec<Value> = non_anchor_idl.instructions().iter().map(|instruction| {
                    self.build_non_anchor_instruction_value(instruction)
                }).collect();
                
                let events: Vec<Value> = non_anchor_idl.events.as_ref().unwrap_or(&vec![]).iter().map(|event| {
                    self.build_non_anchor_event_value(event)
                }).collect();
                
                let types: Vec<Value> = non_anchor_idl.types.as_ref().unwrap_or(&vec![]).iter().map(|type_def| {
                    self.build_non_anchor_type_value(type_def)
                }).collect();
                
                Ok((accounts, instructions, events, types))
            }
        }
    }




    /// 生成parsers文件夹结构
    fn generate_parsers_folder(
        &mut self,
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

        // 生成parsers/mod.rs
        self.generate_folder_mod_file(&parsers_dir, &["accounts".to_string(), "instructions".to_string()], "parsers", template_type)?;
        
        // 生成parsers/accounts.rs文件
        self.generate_parsers_accounts_file(&parsers_dir, context, template_type)?;
        
        // 生成parsers/instructions.rs文件
        self.generate_parsers_instructions_file(&parsers_dir, context, template_type)?;
        
        Ok(())
    }

    /// 生成parsers/mod.rs文件
    fn generate_parsers_mod_file(
        &mut self,
        parsers_dir: &Path,
        context: &Value,
        template_type: &str,
    ) -> std::result::Result<(), SoloresError> {
        let template_content = if template_type == "anchor" {
            include_str!("../minijinatemplates/anchor/parsers/mod.rs.jinja")
        } else {
            include_str!("../minijinatemplates/non_anchor/parsers/mod.rs.jinja")
        };
        
        let tmpl = self.env.template_from_str(template_content)
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
    fn generate_parsers_instructions_file(
        &mut self,
        parsers_dir: &Path,
        context: &Value,
        template_type: &str,
    ) -> std::result::Result<(), SoloresError> {
        let template_content = if template_type == "anchor" {
            include_str!("../minijinatemplates/anchor/parsers/instructions.rs.jinja")
        } else {
            include_str!("../minijinatemplates/non_anchor/parsers/instructions.rs.jinja")
        };
        
        let tmpl = self.env.template_from_str(template_content)
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
    fn generate_parsers_accounts_file(
        &mut self,
        parsers_dir: &Path,
        context: &Value,
        template_type: &str,
    ) -> std::result::Result<(), SoloresError> {
        let template_content = if template_type == "anchor" {
            include_str!("../minijinatemplates/anchor/parsers/accounts.rs.jinja")
        } else {
            include_str!("../minijinatemplates/non_anchor/parsers/accounts.rs.jinja")
        };
        
        let tmpl = self.env.template_from_str(template_content)
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

    /// 生成错误模块
    fn generate_errors_single_file(
        &mut self,
        src_dir: &Path,
        context: &Value,
    ) -> std::result::Result<(), SoloresError> {
        // 使用include_str!直接包含模板内容
        let template_content = include_str!("../minijinatemplates/common/errors.rs.jinja");
        
        let tmpl = self.env.template_from_str(template_content)
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
    fn generate_errors_single_file_fallback(
        &mut self,
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
        
        let tmpl = self.env.template_from_str(&template_content)
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


    /// 生成Cargo.toml文件
    fn generate_cargo_toml(
        &mut self,
        output_dir: &Path,
        context: &Value,
    ) -> std::result::Result<(), SoloresError> {
        // 使用include_str!直接包含Cargo.toml模板
        let template_content = include_str!("../minijinatemplates/common/Cargo.toml.jinja");
        
        let tmpl = self.env.template_from_str(template_content)
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
    fn generate_cargo_toml_fallback(
        &mut self,
        output_dir: &Path,
        context: &Value,
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
"#, self.get_program_name())
        };
        
        let tmpl = self.env.template_from_str(&template_content)
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
    fn generate_readme(
        &mut self,
        output_dir: &Path,
        context: &Value,
    ) -> std::result::Result<(), SoloresError> {
        // 使用include_str!直接包含README模板
        let template_content = include_str!("../minijinatemplates/common/readme.md.jinja");
        
        let tmpl = self.env.template_from_str(template_content)
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
    fn generate_readme_fallback(
        &mut self,
        output_dir: &Path,
        context: &Value,
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
"#, self.get_program_name())
        };
        
        let tmpl = self.env.template_from_str(&template_content)
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

    /// 构建账户Value，确保字段信息完整并修复命名问题
    fn build_account_value(&self, account: &crate::idl_format::anchor_idl::AnchorAccount) -> Value {
        // 首先尝试使用账户自己的字段
        let mut fields: Vec<Value> = if let Some(ref fields_vec) = account.fields {
            log::debug!("  └─ Account {} 有 {} 个直接字段", account.name, fields_vec.len());
            fields_vec.iter().map(|field| {
                self.build_field_value(field)
            }).collect()
        } else {
            Vec::new()
        };

        // 如果账户没有字段，尝试从当前IDL的types中查找同名类型的字段
        if fields.is_empty() {
            if let Some(matching_fields) = self.find_fields_from_types(&account.name) {
                log::debug!("  └─ Account {} 从types获取 {} 个字段", account.name, matching_fields.len());
                fields = matching_fields;
            } else {
                log::debug!("  └─ Account {} 无可用字段", account.name);
            }
        }

        context! {
            name => account.name.to_case(Case::Pascal),  // 确保PascalCase
            discriminator => account.discriminator,
            fields => fields,
            docs => account.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
        }
    }

    /// 构建指令Value，修复命名和字段问题
    fn build_instruction_value(&self, instruction: &crate::idl_format::anchor_idl::AnchorInstruction) -> Value {
        let args: Vec<Value> = if let Some(ref args_vec) = instruction.args {
            log::debug!("  └─ Instruction {} 有 {} 个参数", instruction.name, args_vec.len());
            args_vec.iter().map(|field| {
                self.build_field_value(field)
            }).collect()
        } else {
            log::debug!("  └─ Instruction {} 无参数", instruction.name);
            Vec::new()
        };

        let accounts: Vec<Value> = if let Some(ref accounts_vec) = instruction.accounts {
            accounts_vec.iter().map(|acc| {
                Value::from_serialize(acc)
            }).collect()
        } else {
            Vec::new()
        };

        context! {
            name => instruction.name.to_case(Case::Pascal),  // 修复PascalCase命名
            discriminator => instruction.discriminator,
            args => args.clone(),
            fields => args,  // 模板中使用fields，确保字段数据传递
            accounts => accounts,
            docs => instruction.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
        }
    }

    /// 构建事件Value，确保字段完整
    fn build_event_value(&self, event: &crate::idl_format::anchor_idl::AnchorEvent) -> Value {
        let mut fields: Vec<Value> = if let Some(ref fields_vec) = event.fields {
            log::debug!("  └─ Event {} 有 {} 个直接字段", event.name, fields_vec.len());
            fields_vec.iter().map(|field| {
                self.build_field_value(field)
            }).collect()
        } else {
            log::debug!("  └─ Event {} 无直接字段，尝试从types查找", event.name);
            Vec::new()
        };
        
        // 如果事件没有直接字段，尝试从types中查找同名类型的字段
        if fields.is_empty() {
            if let Some(matching_fields) = self.find_fields_from_types(&event.name) {
                log::debug!("  └─ Event {} 从types获取 {} 个字段", event.name, matching_fields.len());
                fields = matching_fields;
            } else {
                log::debug!("  └─ Event {} 无可用字段", event.name);
            }
        }

        context! {
            name => event.name.to_case(Case::Pascal),  // 确保PascalCase
            discriminator => event.discriminator,
            fields => fields,
            docs => event.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
        }
    }

    /// 手动构建类型Value
    fn build_type_value(&self, type_def: &crate::idl_format::anchor_idl::AnchorType) -> Value {
        let fields_or_variants = match &type_def.kind {
            Some(crate::idl_format::anchor_idl::AnchorTypeKind::Struct(fields)) => {
                fields.iter().map(|field| {
                    self.build_field_value(field)
                }).collect()
            },
            Some(crate::idl_format::anchor_idl::AnchorTypeKind::Enum(variants)) => {
                variants.iter().map(|variant| {
                    let fields: Vec<Value> = if let Some(ref fields_vec) = variant.fields {
                        fields_vec.iter().map(|f| self.build_field_value(f)).collect()
                    } else {
                        Vec::new()
                    };
                    context! {
                        name => variant.name.clone(),
                        fields => fields,
                        docs => variant.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
                    }
                }).collect()
            },
            Some(crate::idl_format::anchor_idl::AnchorTypeKind::Alias(_)) => {
                Vec::new()  // 别名类型不需要字段
            },
            None => Vec::new(),
        };

        context! {
            name => type_def.name.to_case(Case::Pascal),  // 确保PascalCase
            fields => fields_or_variants,
            kind => match &type_def.kind {
                Some(crate::idl_format::anchor_idl::AnchorTypeKind::Struct(_)) => "struct",
                Some(crate::idl_format::anchor_idl::AnchorTypeKind::Enum(_)) => "enum",
                Some(crate::idl_format::anchor_idl::AnchorTypeKind::Alias(_)) => "alias",
                None => "unknown"
            },
            docs => type_def.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
        }
    }

    /// 手动构建字段Value，包含完整的字段信息
    fn build_field_value(&self, field: &crate::idl_format::anchor_idl::AnchorField) -> Value {
        // 转换字段类型为Rust类型字符串
        let rust_type = self.convert_field_type_to_rust(&field.field_type);
        
        context! {
            name => field.name.clone(),
            rust_type => rust_type,
            is_big_array => self.is_big_array(&field.field_type),
            docs => field.docs.as_ref().map(|docs| docs.join("\n")).unwrap_or_default()
        }
    }

    /// 将AnchorFieldType转换为Rust类型字符串，使用完整路径引用
    fn convert_field_type_to_rust(&self, field_type: &crate::idl_format::anchor_idl::AnchorFieldType) -> String {
        use crate::idl_format::anchor_idl::AnchorFieldType;
        match field_type {
            AnchorFieldType::Basic(type_name) => {
                // 基础类型转换，使用完整路径
                match type_name.as_str() {
                    "u8" => "u8".to_string(),
                    "i8" => "i8".to_string(),
                    "u16" => "u16".to_string(),
                    "i16" => "i16".to_string(),
                    "u32" => "u32".to_string(),
                    "i32" => "i32".to_string(),
                    "u64" => "u64".to_string(),
                    "i64" => "i64".to_string(),
                    "u128" => "u128".to_string(),
                    "i128" => "i128".to_string(),
                    "bool" => "bool".to_string(),
                    "f32" => "f32".to_string(),
                    "f64" => "f64".to_string(),
                    "string" => "std::string::String".to_string(),
                    "publicKey" => "solana_pubkey::Pubkey".to_string(),
                    "pubkey" => "solana_pubkey::Pubkey".to_string(),
                    "bytes" => "std::vec::Vec<u8>".to_string(),
                    _ => type_name.clone(),
                }
            },
            AnchorFieldType::PrimitiveOrPubkey(type_name) => {
                match type_name.as_str() {
                    "publicKey" => "solana_pubkey::Pubkey".to_string(),
                    "pubkey" => "solana_pubkey::Pubkey".to_string(),
                    _ => type_name.clone(),
                }
            },
            AnchorFieldType::Complex { kind, params: _ } => {
                // 复合类型处理，使用完整路径
                match kind.as_str() {
                    "option" => "std::option::Option<T>".to_string(),
                    "vec" => "std::vec::Vec<T>".to_string(),
                    "array" => "[T; N]".to_string(),
                    _ => kind.clone(),
                }
            },
            AnchorFieldType::defined(name) => name.clone(),
            AnchorFieldType::array(element_type, size) => {
                let element_rust_type = self.convert_field_type_to_rust(element_type);
                format!("[{}; {}]", element_rust_type, size)
            },
            AnchorFieldType::vec(element_type) => {
                let element_rust_type = self.convert_field_type_to_rust(element_type);
                format!("std::vec::Vec<{}>", element_rust_type)
            },
            AnchorFieldType::option(inner_type) => {
                let inner_rust_type = self.convert_field_type_to_rust(inner_type);
                format!("std::option::Option<{}>", inner_rust_type)
            },
        }
    }

    /// 检查是否是大数组类型（需要serde_big_array处理）
    fn is_big_array(&self, field_type: &crate::idl_format::anchor_idl::AnchorFieldType) -> bool {
        use crate::idl_format::anchor_idl::AnchorFieldType;
        match field_type {
            AnchorFieldType::array(_, size) => *size > 32,  // Rust serde默认只支持到32的数组
            _ => false,
        }
    }

    /// NonAnchor账户构建方法 - 临时占位符
    fn build_non_anchor_account_value(&self, account: &crate::idl_format::non_anchor_idl::NonAnchorAccount) -> Value {
        // 简化实现 - 直接使用序列化
        Value::from_serialize(account)
    }

    /// NonAnchor指令构建方法 - 临时占位符
    fn build_non_anchor_instruction_value(&self, instruction: &crate::idl_format::non_anchor_idl::NonAnchorInstruction) -> Value {
        // 简化实现 - 直接使用序列化
        Value::from_serialize(instruction)
    }

    /// NonAnchor事件构建方法 - 临时占位符
    fn build_non_anchor_event_value(&self, event: &crate::idl_format::non_anchor_idl::NonAnchorEvent) -> Value {
        // 简化实现 - 直接使用序列化
        Value::from_serialize(event)
    }

    /// NonAnchor类型构建方法 - 临时占位符
    fn build_non_anchor_type_value(&self, type_def: &crate::idl_format::non_anchor_idl::NonAnchorType) -> Value {
        // 简化实现 - 直接使用序列化
        Value::from_serialize(type_def)
    }

    /// 从types中查找同名类型的字段（解决IDL中账户定义缺少字段的问题）
    fn find_fields_from_types(&self, account_name: &str) -> Option<Vec<Value>> {
        // 获取当前IDL的types数据
        match &self.idl_enum {
            crate::idl_format::IdlFormatEnum::Anchor(anchor_idl) => {
                if let Some(ref types) = anchor_idl.types {
                    for type_def in types {
                        if type_def.name == account_name {
                            // 找到同名类型，提取字段
                            if let Some(crate::idl_format::anchor_idl::AnchorTypeKind::Struct(fields)) = &type_def.kind {
                                return Some(fields.iter().map(|field| {
                                    self.build_field_value(field)
                                }).collect());
                            }
                        }
                    }
                }
            },
            crate::idl_format::IdlFormatEnum::NonAnchor(non_anchor_idl) => {
                // NonAnchor也可能有类似问题，暂时返回空
                // TODO: 如果需要，可以添加NonAnchor的处理逻辑
            }
        }
        None
    }
}

/// 处理蛇形命名的过滤器
fn to_snake_case_filter(value: String) -> String {
    value.to_case(Case::Snake)
}

/// 处理类型路径的过滤器
fn type_path_filter(value: String) -> String {
    value  // 简单返回原值，后续可以扩展
}

/// 处理 Rust 关键字字段名
fn rust_field_filter(value: String) -> String {
    match value.as_str() {
        "type" => "r#type".to_string(),
        "async" => "r#async".to_string(),
        "await" => "r#await".to_string(),
        "match" => "r#match".to_string(),
        "impl" => "r#impl".to_string(),
        "trait" => "r#trait".to_string(),
        "struct" => "r#struct".to_string(),
        "enum" => "r#enum".to_string(),
        "fn" => "r#fn".to_string(),
        "let" => "r#let".to_string(),
        "mut" => "r#mut".to_string(),
        "ref" => "r#ref".to_string(),
        "if" => "r#if".to_string(),
        "else" => "r#else".to_string(),
        "loop" => "r#loop".to_string(),
        "while" => "r#while".to_string(),
        "for" => "r#for".to_string(),
        "in" => "r#in".to_string(),
        "break" => "r#break".to_string(),
        "continue" => "r#continue".to_string(),
        "return" => "r#return".to_string(),
        "const" => "r#const".to_string(),
        "static" => "r#static".to_string(),
        "pub" => "r#pub".to_string(),
        "mod" => "r#mod".to_string(),
        "use" => "r#use".to_string(),
        "crate" => "r#crate".to_string(),
        "super" => "r#super".to_string(),
        "self" => "r#self".to_string(),
        "Self" => "r#Self".to_string(),
        "where" => "r#where".to_string(),
        "extern" => "r#extern".to_string(),
        "unsafe" => "r#unsafe".to_string(),
        _ => value,
    }
}

/// 检查字符串是否以指定前缀开始
fn starts_with_filter(value: String, prefix: String) -> bool {
    value.starts_with(&prefix)
}

/// 处理多行文档字符串，为每行添加///前缀
fn multiline_docs_filter(value: String) -> String {
    if value.is_empty() {
        return String::new();
    }
    
    value
        .lines()
        .map(|line| {
            if line.trim().is_empty() {
                "///".to_string()
            } else {
                format!("/// {}", line.trim())
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

impl MinijinjaTemplateGenerator {
    /// 生成accounts文件夹和每个账户文件 - 简化版本
    fn generate_accounts_folder(
        &mut self,
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
                            let filename = Self::struct_name_to_filename(account_name);
                            account_names.push(filename.trim_end_matches(".rs").to_string());
                            
                            // 创建单个账户上下文 - 使用官方API
                            let account_context = context! {
                                account => account.clone(),
                                crate_name => context.get_attr("crate_name").unwrap_or(Value::from("")),
                                has_serde => context.get_attr("has_serde").unwrap_or(Value::from(false))
                            };
                            
                            // 生成账户文件
                            self.generate_single_account_file(&accounts_dir, &account_context, template_type, &filename)?;
                        }
                    }
                }
            }
        }
        
        // 生成accounts/mod.rs
        self.generate_folder_mod_file(&accounts_dir, &account_names, "accounts", template_type)?;
        
        Ok(())
    }
    
    /// 生成instructions文件夹和每个指令文件
    fn generate_instructions_folder(
        &mut self,
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
            current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
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
                            let filename = Self::struct_name_to_filename(instruction_name);
                            instruction_names.push(filename.trim_end_matches(".rs").to_string());
                            
                            // 创建单个指令上下文 - 使用官方API
                            let instruction_context = context! {
                                instruction => instruction.clone(),
                                crate_name => context.get_attr("crate_name").unwrap_or(Value::from("")),
                                has_serde => context.get_attr("has_serde").unwrap_or(Value::from(false))
                            };
                            
                            // 生成指令文件
                            self.generate_single_instruction_file(&instructions_dir, &instruction_context, template_type, &filename)?;
                        }
                    }
                }
            }
        }
        
        // 生成instructions/mod.rs
        self.generate_folder_mod_file(&instructions_dir, &instruction_names, "instructions", template_type)?;
        
        Ok(())
    }
    
    /// 生成events文件夹和每个事件文件
    fn generate_events_folder(
        &mut self,
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
        
        // 修复条件检查：正确检查Vec长度
        if events == Value::UNDEFINED {
            log::debug!("❌ events数据为UNDEFINED，跳过生成");
            return Ok(());
        }
        
        let events_len = events.len().unwrap_or(0);
        if events_len == 0 {
            log::debug!("❌ events数据为空，跳过生成");
            return Ok(());
        }
        
        log::debug!("✅ 找到 {} 个events，开始生成目录", events_len);
        
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
        
        // 为每个事件生成单独文件
        for i in 0..events.len().unwrap_or(0) {
            if let Ok(event) = events.get_item(&Value::from(i)) {
                if event != Value::UNDEFINED {
                    if let Ok(name_value) = event.get_attr("name") {
                        if let Some(event_name) = name_value.as_str() {
                            let filename = Self::struct_name_to_filename(event_name);
                            event_names.push(filename.trim_end_matches(".rs").to_string());
                            
                            // 创建单个事件上下文 - 使用官方API
                            let event_context = context! {
                                event => event.clone(),
                                crate_name => context.get_attr("crate_name").unwrap_or(Value::from("")),
                                has_serde => context.get_attr("has_serde").unwrap_or(Value::from(false))
                            };
                            
                            // 生成事件文件
                            self.generate_single_event_file(&events_dir, &event_context, template_type, &filename)?;
                        }
                    }
                }
            }
        }
        
        // 生成events/mod.rs
        self.generate_folder_mod_file(&events_dir, &event_names, "events", template_type)?;
        
        Ok(())
    }
    
    /// 生成types文件夹和每个类型文件
    fn generate_types_folder(
        &mut self,
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
                            let filename = Self::struct_name_to_filename(type_name);
                            type_names.push(filename.trim_end_matches(".rs").to_string());
                            
                            // 创建单个类型上下文 - 使用官方API
                            let type_context = context! {
                                type_def => type_def.clone(),
                                crate_name => context.get_attr("crate_name").unwrap_or(Value::from("")),
                                has_serde => context.get_attr("has_serde").unwrap_or(Value::from(false))
                            };
                            
                            // 生成类型文件
                            self.generate_single_type_file(&types_dir, &type_context, template_type, &filename)?;
                        }
                    }
                }
            }
        }
        
        // 生成types/mod.rs
        self.generate_folder_mod_file(&types_dir, &type_names, "types", template_type)?;
        
        Ok(())
    }
    
    /// 生成单个账户文件
    fn generate_single_account_file(
        &mut self,
        folder: &Path,
        context: &Value,
        template_type: &str,
        filename: &str,
    ) -> std::result::Result<(), SoloresError> {
        let template_content = if template_type == "anchor" {
            include_str!("../minijinatemplates/anchor/accounts/single_account.rs.jinja")
        } else {
            include_str!("../minijinatemplates/non_anchor/accounts/single_account.rs.jinja")
        };
        
        let tmpl = self.env.template_from_str(template_content)
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
    
    /// 生成单个指令文件
    fn generate_single_instruction_file(
        &mut self,
        folder: &Path,
        context: &Value,
        template_type: &str,
        filename: &str,
    ) -> std::result::Result<(), SoloresError> {
        let template_content = if template_type == "anchor" {
            include_str!("../minijinatemplates/anchor/instructions/single_instruction.rs.jinja")
        } else {
            include_str!("../minijinatemplates/non_anchor/instructions/single_instruction.rs.jinja")
        };
        
        let tmpl = self.env.template_from_str(template_content)
            .map_err(|e| SoloresError::TemplateError {
                template_name: Some("instructions/single_instruction.rs.jinja".to_string()),
                message: format!("模板解析失败: {}", e),
                context: Some("解析单个指令模板".to_string()),
            })?;
        
        let rendered = tmpl.render(context)
            .map_err(|e| SoloresError::TemplateError {
                template_name: Some("instructions/single_instruction.rs.jinja".to_string()),
                message: format!("模板渲染失败: {}", e),
                context: Some("渲染单个指令模板".to_string()),
            })?;
        
        let output_path = folder.join(filename);
        fs::write(&output_path, rendered)
            .map_err(|e| SoloresError::FileOperationError {
                operation: "write single instruction file".to_string(),
                path: output_path.display().to_string(),
                current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
                resolved_path: None,
                source: e,
                suggestion: Some("检查文件权限".to_string()),
            })?;
        
        Ok(())
    }
    
    /// 生成单个事件文件
    fn generate_single_event_file(
        &mut self,
        folder: &Path,
        context: &Value,
        template_type: &str,
        filename: &str,
    ) -> std::result::Result<(), SoloresError> {
        let template_content = if template_type == "anchor" {
            include_str!("../minijinatemplates/anchor/events/single_event.rs.jinja")
        } else {
            include_str!("../minijinatemplates/non_anchor/events/single_event.rs.jinja")
        };
        
        let tmpl = self.env.template_from_str(template_content)
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
    
    /// 生成单个类型文件
    fn generate_single_type_file(
        &mut self,
        folder: &Path,
        context: &Value,
        template_type: &str,
        filename: &str,
    ) -> std::result::Result<(), SoloresError> {
        let template_content = if template_type == "anchor" {
            include_str!("../minijinatemplates/anchor/types/single_type.rs.jinja")
        } else {
            include_str!("../minijinatemplates/non_anchor/types/single_type.rs.jinja")
        };
        
        let tmpl = self.env.template_from_str(template_content)
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
    
    /// 生成文件夹的mod.rs
    fn generate_folder_mod_file(
        &mut self,
        folder: &Path,
        items: &[String],
        module_name: &str,
        template_type: &str,
    ) -> std::result::Result<(), SoloresError> {
        let template_content = match (template_type, module_name) {
            ("anchor", "accounts") => include_str!("../minijinatemplates/anchor/accounts/mod.rs.jinja"),
            ("anchor", "instructions") => include_str!("../minijinatemplates/anchor/instructions/mod.rs.jinja"),
            ("anchor", "events") => include_str!("../minijinatemplates/anchor/events/mod.rs.jinja"),
            ("anchor", "types") => include_str!("../minijinatemplates/anchor/types/mod.rs.jinja"),
            ("non_anchor", "accounts") => include_str!("../minijinatemplates/non_anchor/accounts/mod.rs.jinja"),
            ("non_anchor", "instructions") => include_str!("../minijinatemplates/non_anchor/instructions/mod.rs.jinja"),
            ("non_anchor", "events") => include_str!("../minijinatemplates/non_anchor/events/mod.rs.jinja"),
            ("non_anchor", "types") => include_str!("../minijinatemplates/non_anchor/types/mod.rs.jinja"),
            _ => include_str!("../minijinatemplates/anchor/accounts/mod.rs.jinja"), // 默认
        };
        
        // 创建mod.rs上下文 - 使用官方API
        let mod_context = context! {
            module_name => module_name,
            items => items,
            crate_name => ""
        };
        
        let tmpl = self.env.template_from_str(template_content)
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
    fn generate_lib_multi_folder(
        &mut self,
        src_dir: &Path,
        context: &Value,
        template_type: &str,
    ) -> std::result::Result<(), SoloresError> {
        let template_content = if template_type == "anchor" {
            include_str!("../minijinatemplates/anchor/lib.rs.jinja")
        } else {
            include_str!("../minijinatemplates/non_anchor/lib.rs.jinja")
        };
        
        let tmpl = self.env.template_from_str(template_content)
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
}