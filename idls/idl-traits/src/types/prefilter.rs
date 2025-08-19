//! 预过滤器类型定义 (简化版)

use crate::types::Pubkey;

/// 预过滤器 - 用于 Yellowstone 数据过滤
#[derive(Debug, Clone)]
pub struct Prefilter {
    pub transaction_accounts: Vec<Pubkey>,
    pub account_owners: Vec<Pubkey>,
}

impl Prefilter {
    /// 创建预过滤器构建器
    pub fn builder() -> PrefilterBuilder {
        PrefilterBuilder::default()
    }
}

/// 预过滤器构建器
#[derive(Debug, Default)]
pub struct PrefilterBuilder {
    transaction_accounts: Vec<Pubkey>,
    account_owners: Vec<Pubkey>,
}

impl PrefilterBuilder {
    /// 添加交易账户过滤
    pub fn transaction_accounts<I>(mut self, accounts: I) -> Self 
    where
        I: IntoIterator<Item = Pubkey>,
    {
        self.transaction_accounts.extend(accounts);
        self
    }
    
    /// 添加账户所有者过滤
    pub fn account_owners<I>(mut self, owners: I) -> Self
    where
        I: IntoIterator<Item = Pubkey>,
    {
        self.account_owners.extend(owners);
        self
    }
    
    /// 构建预过滤器
    pub fn build(self) -> Result<Prefilter, Box<dyn std::error::Error>> {
        Ok(Prefilter {
            transaction_accounts: self.transaction_accounts,
            account_owners: self.account_owners,
        })
    }
}