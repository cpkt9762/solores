//! MiniJinja模板生成器 - 重构版本
//! 
//! 核心生成器类，协调各个专门模块完成代码生成

use crate::error::SoloresError;
use crate::idl_format::{IdlFormat, IdlFormatEnum};
use log::info;
use minijinja::{Environment, Value, Error};
use std::fs;
use std::path::Path;

use super::{
    filters::*,
    context,
    generators::{accounts, instructions, events, types, parsers, errors, config, common}
};

// 统一库相关结构体定义
use std::path::PathBuf;

/// 协议组，一个组对应一个IDL文件
#[derive(Debug, Clone)]
pub struct ProtocolGroup {
    pub name: String,
    pub idls: Vec<ProtocolIdl>,
}

/// 协议IDL信息
#[derive(Debug, Clone)]
pub struct ProtocolIdl {
    pub file_path: PathBuf,
    pub program_name: String,
    pub idl: IdlFormatEnum,
}

/// 统一库配置
#[derive(Debug, Clone)]
pub struct UnifiedLibraryConfig {
    pub library_name: String,
    pub output_dir: PathBuf,
    pub protocol_groups: Vec<ProtocolGroup>,
    pub base_args: crate::Args,
}

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
        env.add_filter("pascal_case", to_pascal_case_filter);
        env.add_filter("upper", |value: String| -> String { value.to_uppercase() });
        env.add_filter("lower", |value: String| -> String { value.to_lowercase() });
        env.add_filter("type_path", type_path_filter);
        env.add_filter("rust_field", rust_field_filter);
        env.add_filter("starts_with", starts_with_filter);
        env.add_filter("multiline_docs", multiline_docs_filter);
        env.add_filter("regex_replace", regex_replace_filter);
        env.add_filter("is_copy_compatible", is_copy_compatible_filter);
        env.add_filter("is_eq_compatible", is_eq_compatible_filter);
        
        Ok(Self { idl_enum, env })
    }
    
    /// 生成多文件夹架构的完整Rust代码
    pub fn generate_multi_file_structure(
        &mut self,
        output_dir: &Path,
        serde_feature: bool,
        generate_parser: bool,
        no_empty_workspace: bool,
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
        let context = context::create_template_context(
            &self.idl_enum, 
            self.get_program_name(), 
            serde_feature, 
            generate_parser,
            no_empty_workspace,
            false  // is_unified_library
        )?;
        
        // 确定使用的模板类型
        let template_type = if self.is_anchor_idl() { "anchor" } else { "non_anchor" };
        
        // 生成多文件夹模块
        accounts::generate_accounts_folder(&mut self.env, &src_dir, &context, template_type)?;
        instructions::generate_instructions_folder(&mut self.env, &src_dir, &context, template_type)?;
        events::generate_events_folder(&mut self.env, &src_dir, &context, template_type)?;
        types::generate_types_folder(&mut self.env, &src_dir, &context, template_type)?;
        if generate_parser {
            parsers::generate_parsers_folder(&mut self.env, &src_dir, &context, template_type)?;
        }
        errors::generate_errors_single_file(&mut self.env, &src_dir, &context)?;
        common::generate_lib_multi_folder(&mut self.env, &src_dir, &context, template_type)?;
        
        // 生成配置文件
        config::generate_cargo_toml(&mut self.env, output_dir, &context)?;
        config::generate_readme(&mut self.env, output_dir, &context)?;
        config::generate_claude_md(&mut self.env, output_dir, &context)?;
        
        info!("MiniJinja多文件夹架构生成完成");
        Ok(())
    }
    
    /// 为统一库生成指令模块
    pub fn generate_instructions_for_unified(
        &mut self,
        output_dir: &Path,
        args: &crate::Args,
    ) -> Result<(), SoloresError> {
        let context = context::create_template_context(
            &self.idl_enum,
            self.get_program_name(),
            args.generate_to_json,
            args.generate_parser,
            args.no_empty_workspace,
            true  // is_unified_library
        )?;
        
        // Debug: Check if instructions are in context
        if let Ok(instructions) = context.get_attr("instructions") {
            log::debug!("🔍 Unified generation - instructions count: {}", instructions.len().unwrap_or(0));
        } else {
            log::warn!("⚠️ Unified generation - no instructions in context");
        }
        
        let template_type = if self.is_anchor_idl() { "anchor" } else { "non_anchor" };
        instructions::generate_instructions_folder(&mut self.env, output_dir, &context, template_type)
    }
    
    /// 为统一库生成账户模块
    pub fn generate_accounts_for_unified(
        &mut self,
        output_dir: &Path,
        args: &crate::Args,
    ) -> Result<(), SoloresError> {
        let context = context::create_template_context(
            &self.idl_enum,
            self.get_program_name(),
            args.generate_to_json,
            args.generate_parser,
            args.no_empty_workspace,
            true  // is_unified_library
        )?;
        let template_type = if self.is_anchor_idl() { "anchor" } else { "non_anchor" };
        accounts::generate_accounts_folder(&mut self.env, output_dir, &context, template_type)
    }
    
    /// 为统一库生成类型模块
    pub fn generate_types_for_unified(
        &mut self,
        output_dir: &Path,
        args: &crate::Args,
    ) -> Result<(), SoloresError> {
        let context = context::create_template_context(
            &self.idl_enum,
            self.get_program_name(),
            args.generate_to_json,
            args.generate_parser,
            args.no_empty_workspace,
            true  // is_unified_library
        )?;
        let template_type = if self.is_anchor_idl() { "anchor" } else { "non_anchor" };
        types::generate_types_folder(&mut self.env, output_dir, &context, template_type)
    }
    
    /// 为统一库生成错误模块
    pub fn generate_errors_for_unified(
        &mut self,
        output_dir: &Path,
        args: &crate::Args,
    ) -> Result<(), SoloresError> {
        let context = context::create_template_context(
            &self.idl_enum,
            self.get_program_name(),
            args.generate_to_json,
            args.generate_parser,
            args.no_empty_workspace,
            true  // is_unified_library
        )?;
        errors::generate_errors_single_file(&mut self.env, output_dir, &context)
    }
    
    /// 为统一库生成事件模块
    pub fn generate_events_for_unified(
        &mut self,
        output_dir: &Path,
        args: &crate::Args,
    ) -> Result<(), SoloresError> {
        let context = context::create_template_context(
            &self.idl_enum,
            self.get_program_name(),
            args.generate_to_json,
            args.generate_parser,
            args.no_empty_workspace,
            true  // is_unified_library
        )?;
        let template_type = if self.is_anchor_idl() { "anchor" } else { "non_anchor" };
        events::generate_events_folder(&mut self.env, output_dir, &context, template_type)
    }
    
    /// 为统一库生成解析器模块
    pub fn generate_parsers_for_unified(
        &mut self,
        output_dir: &Path,
        args: &crate::Args,
    ) -> Result<(), SoloresError> {
        let context = context::create_template_context(
            &self.idl_enum,
            self.get_program_name(),
            args.generate_to_json,
            args.generate_parser,
            args.no_empty_workspace,
            true  // is_unified_library
        )?;
        let template_type = if self.is_anchor_idl() { "anchor" } else { "non_anchor" };
        parsers::generate_parsers_folder(&mut self.env, output_dir, &context, template_type)
    }
    
    /// 为统一库生成mod.rs文件
    pub fn generate_mod_for_unified(
        &mut self,
        output_dir: &Path,
        args: &crate::Args,
    ) -> Result<(), SoloresError> {
        // 生成包含所有模块的 mod.rs
        let mut mod_content = String::new();
        mod_content.push_str("//! Protocol interface module\n\n");
        
        // 添加程序 ID 常量
        let program_id = match &self.idl_enum {
            crate::idl_format::IdlFormatEnum::Anchor(anchor_idl) => &anchor_idl.address,
            crate::idl_format::IdlFormatEnum::NonAnchor(non_anchor_idl) => &non_anchor_idl.address,
        };
        mod_content.push_str(&format!("/// Program ID\npub const ID: solana_pubkey::Pubkey = solana_pubkey::pubkey!(\"{}\");\n\n", program_id));
        
        // 检查并导出存在的模块
        let modules = ["instructions", "accounts", "types", "errors", "events"];
        let mut has_parsers = false;
        
        for module in &modules {
            mod_content.push_str(&format!("pub mod {};\n", module));
        }
        
        if args.generate_parser {
            mod_content.push_str("pub mod parsers;\n");
            has_parsers = true;
        }
        
        // Re-export all public items
        mod_content.push_str("\n// Re-export all public items\n");
        for module in &modules {
            mod_content.push_str(&format!("pub use {}::*;\n", module));
        }
        if has_parsers {
            mod_content.push_str("pub use parsers::*;\n");
        }
        
        let mod_path = output_dir.join("mod.rs");
        fs::write(mod_path, mod_content).map_err(|e| SoloresError::FileOperationError {
            operation: "write mod.rs".to_string(),
            path: output_dir.display().to_string(),
            current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
            resolved_path: None,
            source: e,
            suggestion: Some("检查目录权限".to_string()),
        })
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

    /// 生成统一接口库
    pub fn generate_unified_library(
        config: &UnifiedLibraryConfig
    ) -> Result<(), SoloresError> {
        info!("开始使用MiniJinja生成统一接口库: {}", config.library_name);
        
        let mut env = Environment::new();
        
        // 添加自定义过滤器
        env.add_filter("snake_case", to_snake_case_filter);
        env.add_filter("pascal_case", to_pascal_case_filter);
        env.add_filter("upper", |value: Value| -> Result<String, Error> {
            Ok(value.to_string().to_uppercase())
        });
        env.add_filter("lower", |value: Value| -> Result<String, Error> {
            Ok(value.to_string().to_lowercase())
        });
        env.add_filter("title", |value: Value| -> Result<String, Error> {
            let s = value.to_string();
            let mut chars = s.chars();
            Ok(match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            })
        });
        env.add_filter("basename", |value: Value| -> Result<String, Error> {
            let s = value.to_string();
            Ok(std::path::Path::new(&s)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&s)
                .to_string())
        });
        
        // 创建输出目录结构
        let lib_output_dir = config.output_dir.join(&config.library_name);
        let src_dir = lib_output_dir.join("src");
        
        fs::create_dir_all(&src_dir).map_err(|e| SoloresError::file_operation_error(
            "创建统一库目录",
            lib_output_dir.display().to_string(),
            e
        ))?;
        
        // 生成根 lib.rs
        Self::generate_unified_lib_rs(&mut env, &lib_output_dir, config)?;
        
        // 生成每个协议模块
        for group in &config.protocol_groups {
            Self::generate_protocol_module(&mut env, &src_dir, group, &config.base_args)?;
        }
        
        // 生成 Cargo.toml
        Self::generate_unified_cargo_toml(&mut env, &lib_output_dir, config)?;
        
        // 生成 README.md
        Self::generate_unified_readme(&mut env, &lib_output_dir, config)?;
        
        // 生成 .gitignore
        let gitignore_content = "target/\nCargo.lock\n.DS_Store\n";
        fs::write(lib_output_dir.join(".gitignore"), gitignore_content)
            .map_err(|e| SoloresError::file_operation_error(
                "创建.gitignore",
                lib_output_dir.join(".gitignore").display().to_string(),
                e
            ))?;
        
        info!("✅ 统一接口库生成完成: {}", lib_output_dir.display());
        Ok(())
    }

    /// 生成统一库的根 lib.rs 文件
    fn generate_unified_lib_rs(
        env: &mut Environment,
        output_dir: &Path,
        config: &UnifiedLibraryConfig,
    ) -> Result<(), SoloresError> {
        let template_str = include_str!("templates/unified/lib.rs.jinja");
        let template = env.template_from_str(template_str)
            .map_err(|e| SoloresError::TemplateError {
                message: e.to_string(),
                template_name: Some("unified/lib.rs.jinja".to_string()),
                context: None,
            })?;

        let context = serde_json::json!({
            "library_name": config.library_name,
            "protocol_groups": config.protocol_groups.iter().map(|g| {
                serde_json::json!({
                    "name": g.name.clone(),
                    "idls": g.idls.iter().map(|idl| {
                        serde_json::json!({
                            "program_name": idl.program_name.clone(),
                        })
                    }).collect::<Vec<_>>()
                })
            }).collect::<Vec<_>>(),
            "generation_time": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        });

        let rendered = template.render(&context)
            .map_err(|e| SoloresError::TemplateError {
                message: e.to_string(),
                template_name: Some("unified/lib.rs.jinja".to_string()),
                context: None,
            })?;

        fs::write(output_dir.join("src/lib.rs"), rendered)
            .map_err(|e| SoloresError::file_operation_error(
                "写入lib.rs",
                output_dir.join("src/lib.rs").display().to_string(),
                e
            ))?;

        Ok(())
    }

    /// 生成协议模块
    fn generate_protocol_module(
        _env: &mut Environment,
        src_dir: &Path,
        group: &ProtocolGroup,
        base_args: &crate::Args,
    ) -> Result<(), SoloresError> {
        let protocol_dir = src_dir.join(&group.name);
        fs::create_dir_all(&protocol_dir).map_err(|e| SoloresError::file_operation_error(
            "创建协议目录",
            protocol_dir.display().to_string(),
            e
        ))?;

        // 使用 UnifiedCodeGenerator trait 生成每个IDL的实际代码
        use crate::idl_format::UnifiedCodeGenerator;
        
        for idl_info in &group.idls {
            log::info!("为 {} 生成代码模块", idl_info.program_name);
            
            // 生成指令模块
            idl_info.idl.generate_instructions(&protocol_dir, base_args)?;
            
            // 生成账户模块
            idl_info.idl.generate_accounts(&protocol_dir, base_args)?;
            
            // 生成类型模块
            idl_info.idl.generate_types(&protocol_dir, base_args)?;
            
            // 生成错误模块
            idl_info.idl.generate_errors(&protocol_dir, base_args)?;
            
            // 生成事件模块
            idl_info.idl.generate_events(&protocol_dir, base_args)?;
            
            // 如果需要生成解析器
            if base_args.generate_parser {
                idl_info.idl.generate_parsers(&protocol_dir, base_args)?;
            }
            
            // 生成模块的 mod.rs 文件
            idl_info.idl.generate_mod_file(&protocol_dir, base_args)?;
        }

        Ok(())
    }

    /// 生成统一 Cargo.toml
    fn generate_unified_cargo_toml(
        env: &mut Environment,
        output_dir: &Path,
        config: &UnifiedLibraryConfig,
    ) -> Result<(), SoloresError> {
        let template_str = include_str!("templates/unified/Cargo.toml.jinja");
        let template = env.template_from_str(template_str)
            .map_err(|e| SoloresError::TemplateError {
                message: e.to_string(),
                template_name: Some("unified/Cargo.toml.jinja".to_string()),
                context: None,
            })?;

        let context = serde_json::json!({
            "library_name": config.library_name,
            "protocol_groups": config.protocol_groups.iter().map(|g| {
                serde_json::json!({
                    "name": g.name,
                })
            }).collect::<Vec<_>>(),
            "solana_program_vers": config.base_args.solana_program_vers,
            "borsh_vers": config.base_args.borsh_vers,
            "thiserror_vers": config.base_args.thiserror_vers,
            "num_derive_vers": config.base_args.num_derive_vers,
            "num_traits_vers": config.base_args.num_traits_vers,
            "serde_vers": config.base_args.serde_vers,
            "serde_with_vers": config.base_args.serde_with_vers,
            "serde_big_array_vers": config.base_args.serde_big_array_vers,
            "serde_json_vers": config.base_args.serde_json_vers,
            "bytemuck_vers": config.base_args.bytemuck_vers,
            "generation_time": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            "solores_version": env!("CARGO_PKG_VERSION"),
        });

        let rendered = template.render(&context)
            .map_err(|e| SoloresError::TemplateError {
                message: e.to_string(),
                template_name: Some("unified/Cargo.toml.jinja".to_string()),
                context: None,
            })?;

        fs::write(output_dir.join("Cargo.toml"), rendered)
            .map_err(|e| SoloresError::file_operation_error(
                "写入Cargo.toml",
                output_dir.join("Cargo.toml").display().to_string(),
                e
            ))?;

        Ok(())
    }

    /// 生成统一 README.md
    fn generate_unified_readme(
        env: &mut Environment,
        output_dir: &Path,
        config: &UnifiedLibraryConfig,
    ) -> Result<(), SoloresError> {
        let template_str = include_str!("templates/unified/README.md.jinja");
        let template = env.template_from_str(template_str)
            .map_err(|e| SoloresError::TemplateError {
                message: e.to_string(),
                template_name: Some("unified/README.md.jinja".to_string()),
                context: None,
            })?;

        let context = serde_json::json!({
            "library_name": config.library_name,
            "protocol_groups": config.protocol_groups.iter().map(|g| {
                serde_json::json!({
                    "name": g.name,
                    "idls": g.idls.iter().map(|idl| {
                        serde_json::json!({
                            "program_name": idl.program_name,
                            "file_path": idl.file_path.display().to_string(),
                            "idl": {
                                "program_address": idl.idl.program_address().unwrap_or_default()
                            }
                        })
                    }).collect::<Vec<_>>()
                })
            }).collect::<Vec<_>>(),
            "generation_time": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        });

        let rendered = template.render(&context)
            .map_err(|e| SoloresError::TemplateError {
                message: e.to_string(),
                template_name: Some("unified/README.md.jinja".to_string()),
                context: None,
            })?;

        fs::write(output_dir.join("README.md"), rendered)
            .map_err(|e| SoloresError::file_operation_error(
                "写入README.md",
                output_dir.join("README.md").display().to_string(),
                e
            ))?;

        Ok(())
    }
}

/// 自动分组协议
pub fn auto_group_protocols(idl_files: &[PathBuf]) -> Result<Vec<ProtocolGroup>, SoloresError> {
    use std::fs::File;
    use std::io::Read;
    
    let mut protocol_groups = Vec::new();
    
    for idl_path in idl_files {
        // 读取并解析IDL文件
        let mut file = File::open(idl_path)
            .map_err(|e| SoloresError::file_operation_error(
                "打开IDL文件",
                idl_path.display().to_string(),
                e
            ))?;
        
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| SoloresError::file_operation_error(
                "读取IDL文件",
                idl_path.display().to_string(),
                e
            ))?;
        
        // 解析IDL
        let idl = crate::idl_format::parse_idl_json(&content)
            .map_err(|e| SoloresError::InvalidIdlFormat {
                details: format!("无法解析IDL文件 {}: {}", idl_path.display(), e),
                expected_format: None,
            })?;
        
        // 获取程序名称作为协议名
        let program_name = idl.program_name().to_string();
        let protocol_name = program_name.replace("-", "_").replace(" ", "_").to_lowercase();
        
        // 创建协议组（每个IDL一个组）
        let protocol_idl = ProtocolIdl {
            file_path: idl_path.clone(),
            program_name: program_name.clone(),
            idl,
        };
        
        let group = ProtocolGroup {
            name: protocol_name,
            idls: vec![protocol_idl],
        };
        
        protocol_groups.push(group);
    }
    
    log::info!("📦 自动分组完成: {} 个协议", protocol_groups.len());
    Ok(protocol_groups)
}