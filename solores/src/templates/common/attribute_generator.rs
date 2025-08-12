//! 属性生成器
//!
//! 生成 derive 属性、cfg 属性、allow 属性等通用属性

use proc_macro2::TokenStream;
use quote::quote;

/// 属性生成器
pub struct AttributeGenerator;

impl AttributeGenerator {
    /// 生成标准的 derive 属性
    ///
    /// 包含 Clone, Debug, borsh::BorshDeserialize, borsh::BorshSerialize, PartialEq
    pub fn generate_standard_derives() -> TokenStream {
        quote! {
            #[derive(Clone, Debug, borsh::BorshDeserialize, borsh::BorshSerialize, PartialEq)]
        }
    }
    
    /// 生成带有 Eq 的标准 derive 属性
    ///
    /// 用于没有浮点数字段的结构体
    pub fn generate_standard_derives_with_eq() -> TokenStream {
        quote! {
            #[derive(Clone, Debug, borsh::BorshDeserialize, borsh::BorshSerialize, PartialEq, Eq)]
        }
    }
    
    /// 生成零拷贝相关的 derive 属性
    ///
    /// 包含 Pod, Zeroable 用于零拷贝优化
    pub fn generate_zerocopy_derives() -> TokenStream {
        quote! {
            #[derive(Clone, Debug, borsh::BorshDeserialize, borsh::BorshSerialize, PartialEq, Pod, Zeroable)]
        }
    }
    
    /// 生成条件性 serde derive 属性
    ///
    /// 只在启用 serde feature 时生效
    pub fn generate_serde_cfg_attr() -> TokenStream {
        quote! {
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        }
    }
    
    /// 生成测试模块属性
    ///
    /// 包含 cfg(test) 和未使用警告抑制
    pub fn generate_test_module_attrs() -> TokenStream {
        quote! {
            #[cfg(test)]
        }
    }
    
    /// 生成测试函数属性
    pub fn generate_test_fn_attrs() -> TokenStream {
        quote! {
            #[test]
        }
    }
    
    /// 生成未使用导入警告抑制属性
    ///
    /// 用于解决编译警告问题
    pub fn generate_allow_unused_imports() -> TokenStream {
        quote! {
            #[allow(unused_imports)]
        }
    }
    
    /// 生成未使用变量警告抑制属性
    pub fn generate_allow_unused_variables() -> TokenStream {
        quote! {
            #[allow(unused_variables)]
        }
    }
    
    /// 生成组合的完整属性集
    ///
    /// 包含标准 derives、serde cfg_attr 和必要的 allow 属性
    pub fn generate_complete_struct_attrs() -> TokenStream {
        let standard_derives = Self::generate_standard_derives_with_eq();
        let serde_attr = Self::generate_serde_cfg_attr();
        
        quote! {
            #standard_derives
            #serde_attr
        }
    }
    
    /// 生成枚举的完整属性集
    pub fn generate_enum_attrs() -> TokenStream {
        quote! {
            #[derive(Clone, Debug, PartialEq)]
        }
    }
}