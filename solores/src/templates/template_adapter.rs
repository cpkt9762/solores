//! 模板适配器
//!
//! 将新的模板系统适配到现有的 IdlCodegenModule 接口中

use proc_macro2::TokenStream;
use crate::idl_format::IdlCodegenModule;
use crate::templates::TemplateGenerator;

/// 模板适配器，将 TemplateGenerator 转换为 IdlCodegenModule
pub struct TemplateAdapter<T: TemplateGenerator> {
    template: T,
}

impl<T: TemplateGenerator> TemplateAdapter<T> {
    /// 创建新的模板适配器
    pub fn new(template: T) -> Self {
        Self { template }
    }
}

impl<T: TemplateGenerator> IdlCodegenModule for TemplateAdapter<T> {
    fn name(&self) -> &str {
        // 返回一个默认的模块名
        "template_module"
    }

    fn gen_head(&self) -> TokenStream {
        // 新模板系统在 generate_code 中包含了所有内容
        quote::quote! {}
    }

    fn gen_body(&self) -> TokenStream {
        // 新模板系统在 gen_files 中包含了所有内容
        quote::quote! {}
    }

    fn has_multiple_files(&self) -> bool {
        // 大多数模板生成单文件，特殊情况可以通过自定义适配器处理
        false
    }

    fn gen_files(&self) -> Vec<(String, TokenStream)> {
        self.template.gen_files()
    }
    
    fn gen_mod_file(&self) -> TokenStream {
        self.template.gen_mod_file()
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        // 返回一个稳定的引用以避免生命周期问题
        &() as &dyn std::any::Any
    }
}