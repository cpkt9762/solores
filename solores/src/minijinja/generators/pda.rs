//! PDA生成器
//! 
//! 负责分析IDL中的PDA定义，生成PDA相关的辅助函数和方法

use crate::error::SoloresError;
use crate::idl_format::anchor_idl::{PdaDefinition, PdaSeed, AnchorAccountConstraint};
use minijinja::Value;
use log;
use convert_case::{Case, Casing};

/// PDA信息结构，用于模板渲染
#[derive(Debug, Clone)]
pub struct PdaInfo {
    /// 账户名称
    pub account_name: String,
    /// Seeds信息
    pub seeds: Vec<SeedInfo>,
    /// 是否有动态参数（需要传递给生成函数）
    pub has_dynamic_params: bool,
    /// 函数参数列表
    pub function_params: Vec<FunctionParam>,
    /// Seeds的Rust代码表示
    pub seeds_code: String,
}

/// Seed信息
#[derive(Debug, Clone)]
pub struct SeedInfo {
    /// Seed类型
    pub seed_type: String,
    /// 对于常量seed，是字符串表示；对于动态seed，是参数名
    pub value: String,
    /// 是否为动态参数
    pub is_dynamic: bool,
}

/// 函数参数信息
#[derive(Debug, Clone)]
pub struct FunctionParam {
    /// 参数名
    pub name: String,
    /// 参数类型
    pub param_type: String,
}

impl PdaInfo {
    /// 从PDA定义创建PDA信息
    pub fn from_pda_definition(
        account_name: &str, 
        pda_def: &PdaDefinition
    ) -> Result<Self, SoloresError> {
        let mut seeds = Vec::new();
        let mut function_params = Vec::new();
        let mut seeds_code_parts = Vec::new();
        let mut has_dynamic_params = false;

        log::debug!("🔍 分析PDA定义 for account: {}", account_name);
        log::debug!("🔍 PDA seeds count: {}", pda_def.seeds.len());

        for (i, seed) in pda_def.seeds.iter().enumerate() {
            match seed {
                PdaSeed::Const { value } => {
                    log::debug!("🔍 常量seed[{}]: {:?}", i, value);
                    
                    // 尝试将字节数组转换为字符串，如果不是有效UTF-8则使用字节数组
                    let seed_value = match String::from_utf8(value.clone()) {
                        Ok(s) => {
                            log::debug!("🔍 常量seed转换为字符串: {}", s);
                            format!("b\"{}\"", s)
                        },
                        Err(_) => {
                            log::debug!("🔍 常量seed使用字节数组: {:?}", value);
                            format!("&{:?}", value)
                        }
                    };
                    
                    seeds.push(SeedInfo {
                        seed_type: "const".to_string(),
                        value: seed_value.clone(),
                        is_dynamic: false,
                    });
                    
                    seeds_code_parts.push(seed_value);
                },
                PdaSeed::Account { path } => {
                    log::debug!("🔍 账户seed[{}]: path = {}", i, path);
                    
                    // 处理复杂路径（如 token_0_vault.mint_account）
                    let param_name = if path.contains('.') {
                        // 对于复杂路径，使用最后一个部分作为参数名
                        let parts: Vec<&str> = path.split('.').collect();
                        let last_part = parts.last().unwrap_or(&"unknown");
                        format!("{}_account", last_part.to_case(Case::Snake))
                    } else {
                        format!("{}_account", path.to_case(Case::Snake))
                    };
                    
                    let param_type = "&solana_pubkey::Pubkey".to_string();
                    
                    seeds.push(SeedInfo {
                        seed_type: "account".to_string(),
                        value: param_name.clone(),
                        is_dynamic: true,
                    });
                    
                    function_params.push(FunctionParam {
                        name: param_name.clone(),
                        param_type,
                    });
                    
                    seeds_code_parts.push(format!("{}.as_ref()", param_name));
                    has_dynamic_params = true;
                },
                PdaSeed::Arg { path } => {
                    log::debug!("🔍 参数seed[{}]: path = {}", i, path);
                    
                    // 处理指令数据字段引用（如 ix.bin_step）
                    if path.starts_with("ix.") {
                        // 这是指令数据字段，需要从指令数据中提取
                        let field_name = path.strip_prefix("ix.").unwrap_or(path);
                        let param_name = format!("{}_value", field_name.to_case(Case::Snake));
                        let param_type = "&[u8]".to_string();
                        
                        seeds.push(SeedInfo {
                            seed_type: "arg".to_string(),
                            value: param_name.clone(),
                            is_dynamic: true,
                        });
                        
                        function_params.push(FunctionParam {
                            name: param_name.clone(),
                            param_type,
                        });
                        
                        seeds_code_parts.push(param_name);
                        has_dynamic_params = true;
                    } else {
                        // 普通参数引用
                        let param_name = path.to_case(Case::Snake);
                        let param_type = "&[u8]".to_string();
                        
                        seeds.push(SeedInfo {
                            seed_type: "arg".to_string(),
                            value: param_name.clone(),
                            is_dynamic: true,
                        });
                        
                        function_params.push(FunctionParam {
                            name: param_name.clone(),
                            param_type,
                        });
                        
                        seeds_code_parts.push(param_name);
                        has_dynamic_params = true;
                    }
                },
            }
        }

        let seeds_code = format!("&[{}]", seeds_code_parts.join(", "));
        
        log::debug!("✅ PDA信息创建完成: account={}, has_dynamic_params={}, seeds_code={}", 
            account_name, has_dynamic_params, seeds_code);

        Ok(PdaInfo {
            account_name: account_name.to_string(),
            seeds,
            has_dynamic_params,
            function_params,
            seeds_code,
        })
    }
}

/// 分析指令中的PDA账户，返回PDA信息列表
pub fn analyze_pda_accounts(
    accounts: &[AnchorAccountConstraint]
) -> Result<Vec<PdaInfo>, SoloresError> {
    let mut pda_infos = Vec::new();
    
    log::debug!("🔍 开始分析PDA账户，总账户数: {}", accounts.len());
    
    for account in accounts {
        if let Some(pda_def) = &account.pda {
            log::debug!("🔍 发现PDA账户: {}", account.name);
            
            let pda_info = PdaInfo::from_pda_definition(&account.name, pda_def)?;
            pda_infos.push(pda_info);
        }
    }
    
    log::debug!("✅ PDA分析完成，发现 {} 个PDA账户", pda_infos.len());
    Ok(pda_infos)
}

/// 为模板系统准备PDA上下文
pub fn prepare_pda_context(pda_infos: &[PdaInfo]) -> Result<Value, SoloresError> {
    let pda_data: Vec<serde_json::Value> = pda_infos.iter().map(|pda| {
        serde_json::json!({
            "account_name": pda.account_name,
            "has_dynamic_params": pda.has_dynamic_params,
            "function_params": pda.function_params.iter().map(|p| {
                serde_json::json!({
                    "name": p.name,
                    "param_type": p.param_type
                })
            }).collect::<Vec<_>>(),
            "seeds_code": pda.seeds_code,
            "seeds": pda.seeds.iter().map(|s| {
                serde_json::json!({
                    "seed_type": s.seed_type,
                    "value": s.value,
                    "is_dynamic": s.is_dynamic
                })
            }).collect::<Vec<_>>()
        })
    }).collect();

    Ok(Value::from_serialize(&pda_data))
}

/// 检查账户是否为PDA
pub fn is_pda_account(account: &AnchorAccountConstraint) -> bool {
    account.pda.is_some()
}

/// 获取PDA账户的生成函数名
pub fn get_pda_function_name(account_name: &str) -> String {
    format!("find_{}_pda", account_name.to_case(Case::Snake))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::idl_format::anchor_idl::{PdaDefinition, PdaSeed};

    #[test]
    fn test_const_seed_pda() {
        let pda_def = PdaDefinition {
            seeds: vec![
                PdaSeed::Const {
                    value: b"vault_authority".to_vec(),
                }
            ],
            program: None,
        };

        let pda_info = PdaInfo::from_pda_definition("vault_authority", &pda_def).unwrap();
        
        assert_eq!(pda_info.account_name, "vault_authority");
        assert!(!pda_info.has_dynamic_params);
        assert_eq!(pda_info.function_params.len(), 0);
        assert_eq!(pda_info.seeds_code, "&[b\"vault_authority\"]");
    }

    #[test]
    fn test_account_seed_pda() {
        let pda_def = PdaDefinition {
            seeds: vec![
                PdaSeed::Const {
                    value: b"bonding_curve".to_vec(),
                },
                PdaSeed::Account {
                    path: "token_0_mint".to_string(),
                }
            ],
            program: None,
        };

        let pda_info = PdaInfo::from_pda_definition("bonding_curve", &pda_def).unwrap();
        
        assert_eq!(pda_info.account_name, "bonding_curve");
        assert!(pda_info.has_dynamic_params);
        assert_eq!(pda_info.function_params.len(), 1);
        assert_eq!(pda_info.function_params[0].name, "token_0_mint_account");
        assert_eq!(pda_info.function_params[0].param_type, "&solana_pubkey::Pubkey");
        assert_eq!(pda_info.seeds_code, "&[b\"bonding_curve\", token_0_mint_account.as_ref()]");
    }
}