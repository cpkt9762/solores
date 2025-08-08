//! 统一接口设计
//!
//! 提供新旧系统之间的统一接口，支持渐进式迁移

use crate::idl_format::{IdlCodegenModule, IdlFormat};
use crate::templates::{TemplateFactory, ContractMode, BoxedTemplateAdapter};
use crate::idl_format::anchor_idl::AnchorIdl;
use crate::Args;

/// 统一代码生成模块类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleType {
    Instructions,
    Accounts,
    Events,
    Types,
    Errors,
    Parsers,
}

/// 统一的代码生成接口
/// 
/// 此 trait 提供了一个统一的接口来创建代码生成模块，
/// 无论底层使用的是旧的 IdlCodegenModule 系统还是新的模板系统
pub trait UnifiedCodegen {
    /// 创建指定类型的代码生成模块
    /// 
    /// # Arguments
    /// * `module_type` - 模块类型 (Instructions, Accounts, Events, etc.)
    /// * `idl` - IDL 定义
    /// * `args` - 生成参数
    /// 
    /// # Returns
    /// * Box<dyn IdlCodegenModule> - 统一的代码生成模块接口
    fn create_module<'a>(
        &self,
        module_type: ModuleType,
        idl: &'a dyn IdlFormat,
        args: &'a Args,
    ) -> Box<dyn IdlCodegenModule + 'a>;
    
    /// 检测是否应该使用新模板系统
    fn should_use_template_system(&self, idl: &dyn IdlFormat) -> bool;
    
    /// 获取合约模式（Anchor 或非 Anchor）
    fn get_contract_mode(&self, idl: &dyn IdlFormat) -> Option<ContractMode>;
}

/// 新模板系统的统一代码生成器
pub struct TemplateUnifiedCodegen;

impl TemplateUnifiedCodegen {
    pub fn new() -> Self {
        Self
    }
    
    /// 将 IDL 转换为 AnchorIdl（如果可能）
    /// 
    /// 这是一个临时的转换方法，因为新模板系统目前只支持 AnchorIdl
    /// 将来需要扩展以支持其他 IDL 格式
    fn try_cast_to_anchor_idl(idl: &dyn IdlFormat) -> Option<&AnchorIdl> {
        // 这里使用 unsafe 转换，在生产环境中应该使用更安全的方法
        // 比如添加一个 IdlFormat::as_anchor_idl() 方法
        unsafe {
            let ptr = idl as *const dyn IdlFormat as *const AnchorIdl;
            Some(&*ptr)
        }
    }
}

impl UnifiedCodegen for TemplateUnifiedCodegen {
    fn create_module<'a>(
        &self,
        module_type: ModuleType,
        idl: &'a dyn IdlFormat,
        args: &'a Args,
    ) -> Box<dyn IdlCodegenModule + 'a> {
        // 尝试转换为 AnchorIdl
        if let Some(anchor_idl) = Self::try_cast_to_anchor_idl(idl) {
            // 使用Anchor工厂方法（假设都是Anchor合约）
            match module_type {
                ModuleType::Instructions => {
                    let template = TemplateFactory::create_anchor_instructions_template(anchor_idl, args);
                    Box::new(BoxedTemplateAdapter::new(template))
                },
                ModuleType::Accounts => {
                    let template = TemplateFactory::create_anchor_accounts_template(anchor_idl, args);
                    Box::new(BoxedTemplateAdapter::new(template))
                },
                ModuleType::Events => {
                    let template = TemplateFactory::create_anchor_events_template(anchor_idl);
                    Box::new(BoxedTemplateAdapter::new(template))
                },
                ModuleType::Types => {
                    // TypesTemplateGenerator 需要特殊处理，暂时使用空实现
                    Box::new(EmptyTypesTemplateAdapter)
                },
                ModuleType::Errors => {
                    let template = TemplateFactory::create_anchor_errors_template(
                        anchor_idl.program_name(),
                        anchor_idl.errors.as_deref().unwrap_or(&[])
                    );
                    Box::new(BoxedTemplateAdapter::new(template))
                },
                ModuleType::Parsers => {
                    // ParsersTemplateGenerator 需要特殊处理，暂时使用空实现
                    Box::new(EmptyParsersTemplateAdapter)
                },
            }
        } else {
            // 回退到旧系统（这里需要实现回退逻辑）
            panic!("Fallback to old system not yet implemented for non-AnchorIdl formats")
        }
    }
    
    fn should_use_template_system(&self, idl: &dyn IdlFormat) -> bool {
        // 目前新模板系统只支持 AnchorIdl
        Self::try_cast_to_anchor_idl(idl).is_some()
    }
    
    fn get_contract_mode(&self, idl: &dyn IdlFormat) -> Option<ContractMode> {
        if let Some(_anchor_idl) = Self::try_cast_to_anchor_idl(idl) {
            // 所有AnchorIdl都是Anchor模式
            Some(ContractMode::Anchor)
        } else {
            None
        }
    }
}

impl Default for TemplateUnifiedCodegen {
    fn default() -> Self {
        Self::new()
    }
}

/// 旧系统的统一代码生成器（保留用于向后兼容）
pub struct LegacyUnifiedCodegen;

impl LegacyUnifiedCodegen {
    pub fn new() -> Self {
        Self
    }
}

impl UnifiedCodegen for LegacyUnifiedCodegen {
    fn create_module<'a>(
        &self,
        _module_type: ModuleType,
        _idl: &'a dyn IdlFormat,
        _args: &'a Args,
    ) -> Box<dyn IdlCodegenModule + 'a> {
        // 这里应该调用旧系统的模块创建逻辑
        // 目前先返回一个占位符实现
        todo!("Legacy system module creation not yet implemented")
    }
    
    fn should_use_template_system(&self, _idl: &dyn IdlFormat) -> bool {
        false // 旧系统不使用模板
    }
    
    fn get_contract_mode(&self, _idl: &dyn IdlFormat) -> Option<ContractMode> {
        None // 旧系统不区分合约模式
    }
}

impl Default for LegacyUnifiedCodegen {
    fn default() -> Self {
        Self::new()
    }
}

/// 统一代码生成器工厂
pub struct UnifiedCodegenFactory;

impl UnifiedCodegenFactory {
    /// 创建合适的统一代码生成器
    /// 
    /// # Arguments
    /// * `prefer_template_system` - 是否优先使用模板系统
    /// 
    /// # Returns
    /// * Box<dyn UnifiedCodegen> - 统一代码生成器
    pub fn create_codegen(prefer_template_system: bool) -> Box<dyn UnifiedCodegen> {
        if prefer_template_system {
            Box::new(TemplateUnifiedCodegen::new())
        } else {
            Box::new(LegacyUnifiedCodegen::new())
        }
    }
    
    /// 为特定 IDL 创建最合适的代码生成器
    /// 
    /// # Arguments
    /// * `idl` - IDL 定义
    /// 
    /// # Returns
    /// * Box<dyn UnifiedCodegen> - 最合适的统一代码生成器
    pub fn create_codegen_for_idl(idl: &dyn IdlFormat) -> Box<dyn UnifiedCodegen> {
        let template_codegen = TemplateUnifiedCodegen::new();
        
        if template_codegen.should_use_template_system(idl) {
            Box::new(template_codegen)
        } else {
            Box::new(LegacyUnifiedCodegen::new())
        }
    }
}

/// 模块生成策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationStrategy {
    /// 使用新模板系统
    Template,
    /// 使用旧代码生成系统
    Legacy,
    /// 混合模式：某些模块使用模板，某些使用旧系统
    Hybrid,
}

/// 代码生成上下文
/// 
/// 包含生成过程中需要的所有上下文信息
pub struct CodegenContext<'a> {
    pub idl: &'a dyn IdlFormat,
    pub args: &'a Args,
    pub strategy: GenerationStrategy,
    pub codegen: Box<dyn UnifiedCodegen>,
}

impl<'a> CodegenContext<'a> {
    /// 创建新的代码生成上下文
    pub fn new(
        idl: &'a dyn IdlFormat,
        args: &'a Args,
        strategy: GenerationStrategy,
    ) -> Self {
        let codegen = match strategy {
            GenerationStrategy::Template => UnifiedCodegenFactory::create_codegen(true),
            GenerationStrategy::Legacy => UnifiedCodegenFactory::create_codegen(false),
            GenerationStrategy::Hybrid => UnifiedCodegenFactory::create_codegen_for_idl(idl),
        };
        
        Self {
            idl,
            args,
            strategy,
            codegen,
        }
    }
    
    /// 创建指定类型的模块
    pub fn create_module(&self, module_type: ModuleType) -> Box<dyn IdlCodegenModule + 'a> {
        self.codegen.create_module(module_type, self.idl, self.args)
    }
    
    /// 批量创建所有类型的模块
    pub fn create_all_modules(&self) -> Vec<(ModuleType, Box<dyn IdlCodegenModule + 'a>)> {
        vec![
            ModuleType::Instructions,
            ModuleType::Accounts,
            ModuleType::Events,
            ModuleType::Types,
            ModuleType::Errors,
            ModuleType::Parsers,
        ]
        .into_iter()
        .map(|module_type| (module_type, self.create_module(module_type)))
        .collect()
    }
}

/// 空的 Types 模板适配器
struct EmptyTypesTemplateAdapter;

impl crate::idl_format::IdlCodegenModule for EmptyTypesTemplateAdapter {
    fn name(&self) -> &str {
        "types"
    }

    fn gen_head(&self) -> proc_macro2::TokenStream {
        quote::quote! {}
    }

    fn gen_body(&self) -> proc_macro2::TokenStream {
        quote::quote! {
            // 临时实现：Types 模板正在开发中
        }
    }

    fn has_multiple_files(&self) -> bool {
        false
    }

    fn gen_files(&self) -> Vec<(String, proc_macro2::TokenStream)> {
        vec![("types.rs".to_string(), self.gen_body())]
    }
    
    fn gen_mod_file(&self) -> proc_macro2::TokenStream {
        quote::quote! {
            //! Types module (empty implementation)
            pub use super::types::*;
        }
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 空的 Parsers 模板适配器
struct EmptyParsersTemplateAdapter;

impl crate::idl_format::IdlCodegenModule for EmptyParsersTemplateAdapter {
    fn name(&self) -> &str {
        "parsers"
    }

    fn gen_head(&self) -> proc_macro2::TokenStream {
        quote::quote! {}
    }

    fn gen_body(&self) -> proc_macro2::TokenStream {
        quote::quote! {
            // 临时实现：Parsers 模板正在开发中
        }
    }

    fn has_multiple_files(&self) -> bool {
        false
    }

    fn gen_files(&self) -> Vec<(String, proc_macro2::TokenStream)> {
        vec![("parsers.rs".to_string(), self.gen_body())]
    }
    
    fn gen_mod_file(&self) -> proc_macro2::TokenStream {
        quote::quote! {
            //! Parsers module (empty implementation)
            pub use super::parsers::*;
        }
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}