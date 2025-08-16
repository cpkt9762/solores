//! Askama 模板生成器
//!
//! 简单版本，使用内联模板验证集成

use askama::Template;
use crate::{
    Args, SoloresError,
    idl_format::IdlFormat
};
use super::askama_templates::SimpleLibTemplate;

/// 简单Askama生成器 - 用于从dyn IdlFormat调用
pub struct SimpleAskamaGenerator<'a> {
    args: &'a Args,
    idl: &'a dyn IdlFormat,
}

impl<'a> SimpleAskamaGenerator<'a> {
    pub fn new(args: &'a Args, idl: &'a dyn IdlFormat) -> Self {
        Self { args, idl }
    }
    
    pub fn generate(&self) -> Result<(), SoloresError> {
        log::info!("🎨 使用Askama模板系统生成代码");
        
        // 创建输出目录
        std::fs::create_dir_all(&self.args.output_dir.join("src"))
            .map_err(|e| SoloresError::FileOperationError {
                operation: "创建目录".to_string(),
                path: self.args.output_dir.join("src").display().to_string(),
                current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
                resolved_path: None,
                source: e,
                suggestion: None,
            })?;
        
        // 获取程序信息
        let crate_name = self.idl.program_name().to_string();
        let program_id = self.get_program_id();
        
        // 使用Askama模板生成lib.rs
        let lib_template = SimpleLibTemplate {
            crate_name: crate_name.clone(),
            program_id: program_id.clone(),
        };
        
        let rendered = lib_template.render()
            .map_err(|e| SoloresError::CodeGenError {
                module: "lib.rs".to_string(),
                reason: format!("Askama模板渲染失败: {}", e),
                context: Some("SimpleLibTemplate".to_string()),
            })?;
        
        let output_path = self.args.output_dir.join("src").join("lib.rs");
        std::fs::write(&output_path, rendered)
            .map_err(|e| SoloresError::FileOperationError {
                operation: "写入文件".to_string(),
                path: output_path.display().to_string(),
                current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
                resolved_path: None,
                source: e,
                suggestion: None,
            })?;
            
        log::info!("✅ Askama生成完成: {}", output_path.display());
        Ok(())
    }
    
    fn get_program_id(&self) -> String {
        if let Some(program_id) = &self.args.program_id {
            program_id.clone()
        } else if let Some(address) = self.idl.program_address() {
            address.to_string()
        } else {
            "TH1S1SNoTAVAL1DPUBKEYDoNoTUSE11111111111111".to_string()
        }
    }
}