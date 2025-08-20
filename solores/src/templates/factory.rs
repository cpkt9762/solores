//! 模板工厂
//!
//! 极简二元架构：只支持 Anchor 和 NonAnchor 两种合约模式的统一模板创建接口

use crate::idl_format::{AnchorIdl, NonAnchorIdl};
use crate::Args;

use super::{
    AccountsParserTestGenerator, InstructionsParserTestGenerator,
    NonAnchorAccountsParserTestGenerator, NonAnchorInstructionsParserTestGenerator,
    ParsersTemplateGenerator, TemplateGenerator, TypesTemplateGenerator,
};

use super::anchor::{
    AnchorAccountsParserTestTemplate, AnchorAccountsTemplate, AnchorErrorsTemplate,
    AnchorEventsTemplate, AnchorInstructionsParserTestTemplate, AnchorInstructionsTemplate,
    AnchorParsersTemplate, AnchorTypesTemplate,
};

use super::non_anchor::{
    NonAnchorAccountsParserTestTemplate, NonAnchorAccountsTemplate, NonAnchorErrorsTemplate,
    NonAnchorEventsTemplate, NonAnchorInstructionsParserTestTemplate,
    NonAnchorInstructionsTemplate, NonAnchorParsersTemplate, NonAnchorTypesTemplate,
};

// 添加必要的导入

/// 极简模板工厂
pub struct TemplateFactory;

impl TemplateFactory {
    // ======= Anchor模板创建方法 =======

    /// 创建Anchor Instructions模板
    pub fn create_anchor_instructions_template<'a>(
        idl: &'a AnchorIdl,
        args: &'a Args,
    ) -> Box<dyn TemplateGenerator + 'a> {
        Box::new(AnchorInstructionsTemplate::new(idl, args))
    }

    /// 创建Anchor Accounts模板（统一使用字段分配机制）
    pub fn create_anchor_accounts_template<'a>(
        idl: &'a AnchorIdl,
        args: &'a Args,
    ) -> Box<dyn TemplateGenerator + 'a> {
        Box::new(AnchorAccountsTemplate::new(idl, args))
    }

    /// 创建Anchor Events模板（统一使用字段分配机制）
    pub fn create_anchor_events_template<'a>(
        idl: &'a AnchorIdl,
    ) -> Box<dyn TemplateGenerator + 'a> {
        Box::new(AnchorEventsTemplate::new(idl))
    }

    /// 创建Anchor Types模板
    pub fn create_anchor_types_template<'a>(
        idl: &'a AnchorIdl,
        args: &'a Args,
    ) -> Box<dyn TypesTemplateGenerator + 'a> {
        Box::new(AnchorTypesTemplate::new(idl, args))
    }

    /// 创建Anchor Errors模板
    pub fn create_anchor_errors_template<'a>(
        program_name: &'a str,
        errors: &'a [crate::idl_format::anchor_idl::AnchorError],
    ) -> Box<dyn TemplateGenerator + 'a> {
        Box::new(AnchorErrorsTemplate::new(program_name, errors))
    }

    /// 创建Anchor Parsers模板
    pub fn create_anchor_parsers_template<'a>(
        idl: &'a AnchorIdl,
        args: &'a Args,
    ) -> Box<dyn ParsersTemplateGenerator + 'a> {
        Box::new(AnchorParsersTemplate::new(idl, args))
    }

    // ======= NonAnchor模板创建方法 =======

    /// 创建NonAnchor Instructions模板
    pub fn create_non_anchor_instructions_template<'a>(
        idl: &'a NonAnchorIdl,
        args: &'a Args,
    ) -> Box<dyn TemplateGenerator + 'a> {
        let named_types: &'a [crate::idl_format::non_anchor_idl::NonAnchorType] = 
            idl.types.as_deref().unwrap_or(&[]);
        Box::new(NonAnchorInstructionsTemplate::new(idl, args, named_types))
    }

    /// 创建NonAnchor Accounts模板
    pub fn create_non_anchor_accounts_template<'a>(
        idl: &'a NonAnchorIdl,
        args: &'a Args,
    ) -> Box<dyn TemplateGenerator + 'a> {
        Box::new(NonAnchorAccountsTemplate::new(idl, args))
    }

    /// 创建NonAnchor Events模板
    pub fn create_non_anchor_events_template<'a>(
        idl: &'a NonAnchorIdl,
    ) -> Box<dyn TemplateGenerator + 'a> {
        Box::new(NonAnchorEventsTemplate::new(idl))
    }

    /// 创建NonAnchor Types模板
    pub fn create_non_anchor_types_template<'a>(
        idl: &'a NonAnchorIdl,
        args: &'a Args,
    ) -> Box<dyn TypesTemplateGenerator + 'a> {
        Box::new(NonAnchorTypesTemplate::new(idl, args))
    }

    /// 创建NonAnchor Errors模板
    pub fn create_non_anchor_errors_template<'a>(
        program_name: &'a str,
        errors: &'a [crate::idl_format::non_anchor_idl::NonAnchorError],
    ) -> Box<dyn TemplateGenerator + 'a> {
        Box::new(NonAnchorErrorsTemplate::new(program_name, errors))
    }

    /// 创建NonAnchor Parsers模板
    pub fn create_non_anchor_parsers_template<'a>(
        idl: &'a NonAnchorIdl,
        args: &'a Args,
    ) -> Box<dyn ParsersTemplateGenerator + 'a> {
        Box::new(NonAnchorParsersTemplate::new(idl, args))
    }

    // ======= 测试模板创建方法 =======

    /// 创建 Anchor Instructions Parser 测试模板
    pub fn create_anchor_instructions_parser_test_template(
    ) -> Box<dyn InstructionsParserTestGenerator> {
        Box::new(AnchorInstructionsParserTestTemplate::new())
    }

    /// 创建 NonAnchor Instructions Parser 测试模板
    pub fn create_non_anchor_instructions_parser_test_template(
    ) -> Box<dyn NonAnchorInstructionsParserTestGenerator> {
        Box::new(NonAnchorInstructionsParserTestTemplate::new())
    }

    /// 创建 Anchor Accounts Parser 测试模板
    pub fn create_anchor_accounts_parser_test_template() -> Box<dyn AccountsParserTestGenerator> {
        Box::new(AnchorAccountsParserTestTemplate::new())
    }

    /// 创建 NonAnchor Accounts Parser 测试模板
    pub fn create_non_anchor_accounts_parser_test_template(
    ) -> Box<dyn NonAnchorAccountsParserTestGenerator> {
        Box::new(NonAnchorAccountsParserTestTemplate::new())
    }

    // ======= 内部转换辅助方法 =======

    /// 将NonAnchorIdl转换为AnchorIdl兼容格式
    /// 这是临时方案，用于兼容当前的模板接口
    #[allow(dead_code)]
    fn convert_non_anchor_to_anchor_compat(idl: &NonAnchorIdl) -> AnchorIdl {
        // 转换instructions
        let instructions = idl.instructions.as_ref().map(|non_anchor_instructions| {
            non_anchor_instructions
                .iter()
                .map(|ni| {
                    crate::idl_format::anchor_idl::AnchorInstruction {
                        name: ni.name.clone(),
                        discriminator: {
                            // NonAnchor使用1字节discriminator，但我们需要8字节格式
                            let mut disc = [0u8; 8];
                            if let Some(ref discriminator) = ni.discriminator {
                                if !discriminator.is_empty() {
                                    disc[0] = discriminator[0]; // 只取第一个字节
                                }
                            }
                            disc
                        },
                        accounts: ni.accounts.as_ref().map(|accounts| {
                            accounts
                                .iter()
                                .map(|acc| {
                                    crate::idl_format::anchor_idl::AnchorAccountConstraint {
                                        name: acc.name.clone(),
                                        is_mut: false,     // 默认值
                                        is_signer: false,  // 默认值
                                        is_optional: None, // 默认值
                                        docs: acc.docs.clone(),
                                        constraints: None,
                                    }
                                })
                                .collect()
                        }),
                        args: ni.args.as_ref().map(|args| {
                            args.iter()
                                .map(|arg| crate::idl_format::anchor_idl::AnchorField {
                                    name: arg.name.clone(),
                                    field_type: Self::convert_non_anchor_field_type_to_anchor(
                                        &arg.field_type,
                                    ),
                                    kind: None,
                                    docs: arg.docs.clone(),
                                })
                                .collect()
                        }),
                        docs: ni.docs.clone(),
                    }
                })
                .collect()
        });

        // 转换accounts
        let accounts = idl.accounts.as_ref().map(|non_anchor_accounts| {
            non_anchor_accounts
                .iter()
                .map(|na| {
                    crate::idl_format::anchor_idl::AnchorAccount {
                        name: na.name.clone(),
                        discriminator: [0u8; 8], // 默认discriminator
                        fields: na.fields.as_ref().map(|fields| {
                            fields
                                .iter()
                                .map(|field| crate::idl_format::anchor_idl::AnchorField {
                                    name: field.name.clone(),
                                    field_type: Self::convert_non_anchor_field_type_to_anchor(
                                        &field.field_type,
                                    ),
                                    kind: None,
                                    docs: field.docs.clone(),
                                })
                                .collect()
                        }),
                        docs: na.docs.clone(),
                    }
                })
                .collect()
        });

        AnchorIdl {
            name: Some(idl.program_name().to_string()),
            version: Some(idl.program_version().to_string()),
            address: idl.address.clone(),
            instructions,
            accounts,
            types: None,  // 临时不转换复杂类型
            events: None, // 临时不转换事件
            errors: None, // 临时不转换错误
            constants: None,
            metadata: crate::idl_format::anchor_idl::AnchorMetadata {
                name: idl.program_name().to_string(),
                version: idl.program_version().to_string(),
                address: Some(idl.address.clone()),
                spec: "non-anchor-compat".to_string(),
                description: idl.metadata.as_ref().and_then(|m| m.description.clone()),
            },
            field_allocation_cache: std::sync::OnceLock::new(),
        }
    }

    /// 转换NonAnchor字段类型为Anchor字段类型
    #[allow(dead_code)]
    fn convert_non_anchor_field_type_to_anchor(
        field_type: &crate::idl_format::non_anchor_idl::NonAnchorFieldType,
    ) -> crate::idl_format::anchor_idl::AnchorFieldType {
        match field_type {
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Basic(type_str) => {
                match type_str.as_str() {
                    "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "u64" | "i64" | "u128"
                    | "i128" => {
                        crate::idl_format::anchor_idl::AnchorFieldType::Basic(type_str.clone())
                    }
                    "bool" => {
                        crate::idl_format::anchor_idl::AnchorFieldType::Basic("bool".to_string())
                    }
                    "string" | "String" => {
                        crate::idl_format::anchor_idl::AnchorFieldType::Basic("String".to_string())
                    }
                    "publicKey" | "pubkey" | "Pubkey" => {
                        crate::idl_format::anchor_idl::AnchorFieldType::Basic("Pubkey".to_string())
                    }
                    _ => crate::idl_format::anchor_idl::AnchorFieldType::Basic(type_str.clone()),
                }
            }
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Option { option } => {
                let inner_type = Self::convert_non_anchor_field_type_to_anchor(option);
                crate::idl_format::anchor_idl::AnchorFieldType::option(Box::new(inner_type))
            }
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Vec { vec } => {
                let inner_type = Self::convert_non_anchor_field_type_to_anchor(vec);
                crate::idl_format::anchor_idl::AnchorFieldType::vec(Box::new(inner_type))
            }
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Array { array } => {
                let (inner_type, size) = array;
                let inner_type_anchor = Self::convert_non_anchor_field_type_to_anchor(inner_type);
                crate::idl_format::anchor_idl::AnchorFieldType::array(Box::new(inner_type_anchor), *size)
            }
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Defined { defined } => {
                crate::idl_format::anchor_idl::AnchorFieldType::defined(defined.clone())
            }
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::HashMap { key: _, value: _ } => {
                // HashMap转换为简化的Basic类型处理
                crate::idl_format::anchor_idl::AnchorFieldType::Basic("HashMap".to_string())
            }
            crate::idl_format::non_anchor_idl::NonAnchorFieldType::Complex { kind, params: _ } => {
                match kind.as_str() {
                    "Vec" => {
                        crate::idl_format::anchor_idl::AnchorFieldType::Basic("Vec".to_string())
                    } // 简化处理
                    "Option" => {
                        crate::idl_format::anchor_idl::AnchorFieldType::Basic("Option".to_string())
                    } // 简化处理
                    _ => crate::idl_format::anchor_idl::AnchorFieldType::Basic(kind.clone()),
                }
            }
        }
    }
}

/// 完整的Anchor模板集合
pub struct AnchorTemplateSet<'a> {
    pub instructions: Box<dyn TemplateGenerator + 'a>,
    pub accounts: Box<dyn TemplateGenerator + 'a>,
    pub events: Box<dyn TemplateGenerator + 'a>,
    pub types: Box<dyn TypesTemplateGenerator + 'a>,
    pub errors: Box<dyn TemplateGenerator + 'a>,
    pub parsers: Box<dyn ParsersTemplateGenerator + 'a>,
    pub instructions_parser_test: Box<dyn InstructionsParserTestGenerator>,
    pub accounts_parser_test: Box<dyn AccountsParserTestGenerator>,
}

/// 完整的NonAnchor模板集合
pub struct NonAnchorTemplateSet<'a> {
    pub instructions: Box<dyn TemplateGenerator + 'a>,
    pub accounts: Box<dyn TemplateGenerator + 'a>,
    pub events: Box<dyn TemplateGenerator + 'a>,
    pub types: Box<dyn TypesTemplateGenerator + 'a>,
    pub errors: Box<dyn TemplateGenerator + 'a>,
    pub parsers: Box<dyn ParsersTemplateGenerator + 'a>,
    pub instructions_parser_test: Box<dyn NonAnchorInstructionsParserTestGenerator>,
    pub accounts_parser_test: Box<dyn NonAnchorAccountsParserTestGenerator>,
}

impl TemplateFactory {
    /// 为Anchor IDL创建完整的模板集合
    pub fn create_anchor_template_set<'a>(
        idl: &'a AnchorIdl,
        args: &'a Args,
    ) -> AnchorTemplateSet<'a> {
        AnchorTemplateSet {
            instructions: Self::create_anchor_instructions_template(idl, args),
            accounts: Self::create_anchor_accounts_template(idl, args),
            events: Self::create_anchor_events_template(idl),
            types: Self::create_anchor_types_template(idl, args),
            errors: Self::create_anchor_errors_template(
                idl.program_name(),
                idl.errors.as_deref().unwrap_or(&[]),
            ),
            parsers: Self::create_anchor_parsers_template(idl, args),
            instructions_parser_test: Self::create_anchor_instructions_parser_test_template(),
            accounts_parser_test: Self::create_anchor_accounts_parser_test_template(),
        }
    }

    /// 为NonAnchor IDL创建完整的模板集合
    pub fn create_non_anchor_template_set<'a>(
        idl: &'a NonAnchorIdl,
        args: &'a Args,
    ) -> NonAnchorTemplateSet<'a> {
        NonAnchorTemplateSet {
            instructions: Self::create_non_anchor_instructions_template(idl, args),
            accounts: Self::create_non_anchor_accounts_template(idl, args),
            events: Self::create_non_anchor_events_template(idl),
            types: Self::create_non_anchor_types_template(idl, args),
            errors: Self::create_non_anchor_errors_template(
                idl.program_name(),
                idl.errors.as_deref().unwrap_or(&[]),
            ),
            parsers: Self::create_non_anchor_parsers_template(idl, args),
            instructions_parser_test: Self::create_non_anchor_instructions_parser_test_template(),
            accounts_parser_test: Self::create_non_anchor_accounts_parser_test_template(),
        }
    }
}

// 空实现类已移除 - 现在使用真正的NonAnchor模板实现
