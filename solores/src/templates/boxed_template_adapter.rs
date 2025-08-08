//! Boxed Template Adapter
//!
//! 专门用于处理 Box<dyn TemplateGenerator> 到 IdlCodegenModule 的转换

use proc_macro2::TokenStream;
use crate::idl_format::IdlCodegenModule;
use crate::templates::TemplateGenerator;

/// 用于包装 Box<dyn TemplateGenerator> 的适配器
pub struct BoxedTemplateAdapter<'a> {
    template: Box<dyn TemplateGenerator + 'a>,
    name: String,
}

impl<'a> BoxedTemplateAdapter<'a> {
    /// 创建新的 boxed 模板适配器
    pub fn new(template: Box<dyn TemplateGenerator + 'a>) -> Self {
        // 使用模板的标准模块名称，不再从文件名推断
        let module_name = template.get_standard_module_name().to_string();
        
        Self { template, name: module_name }
    }
    
    /// 检查是否是根目录单文件模板（如errors.rs）
    pub fn is_single_root_file(&self) -> bool {
        self.template.is_single_root_file()
    }
}

impl<'a> IdlCodegenModule for BoxedTemplateAdapter<'a> {
    fn name(&self) -> &str {
        &self.name
    }

    fn gen_head(&self) -> TokenStream {
        // 新模板系统在 generate_code 中包含了所有内容
        quote::quote! {}
    }

    fn gen_body(&self) -> TokenStream {
        // 如果是单根文件模板（如errors.rs），返回文件内容
        if self.template.is_single_root_file() {
            let files = self.template.gen_files();
            if let Some((_, content)) = files.first() {
                content.clone()
            } else {
                quote::quote! {}
            }
        } else {
            // 多文件模式不使用gen_body
            quote::quote! {}
        }
    }

    fn has_multiple_files(&self) -> bool {
        // 检查模板是否为单根文件（如errors.rs）
        !self.template.is_single_root_file()
    }

    fn gen_files(&self) -> Vec<(String, TokenStream)> {
        log::debug!("🔧 BoxedTemplateAdapter开始生成文件，模板: {}", self.name);
        let result = self.template.gen_files();
        log::debug!("🔧 BoxedTemplateAdapter生成了{}个文件", result.len());
        result
    }
    
    fn gen_mod_file(&self) -> TokenStream {
        self.template.gen_mod_file()
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        // 在 BoxedTemplateAdapter 的情况下返回一个 dummy 值
        // write_src.rs现在直接使用has_multiple_files()来判断，不依赖downcast
        &() as &dyn std::any::Any
    }
}