//! MiniJinja模板生成器 - 重构版本
//! 
//! 核心生成器类，协调各个专门模块完成代码生成

use crate::error::SoloresError;
use crate::idl_format::IdlFormatEnum;
use log::info;
use minijinja::Environment;
use std::fs;
use std::path::Path;

use super::{
    filters::*,
    context,
    generators::{accounts, instructions, events, types, parsers, errors, config, common}
};

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
        let context = context::create_template_context(
            &self.idl_enum, 
            self.get_program_name(), 
            serde_feature, 
            generate_parser
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
}