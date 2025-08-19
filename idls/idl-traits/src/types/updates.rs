//! Yellowstone 数据更新类型定义 (简化版)

use crate::types::Pubkey;

/// 指令更新数据 (来自交易)
#[derive(Debug, Clone)]
pub struct InstructionUpdate {
    /// 程序ID
    pub program: Pubkey,
    /// 指令数据
    pub data: Vec<u8>,
    /// 涉及的账户列表
    pub accounts: Vec<Pubkey>,
    /// 堆栈高度
    pub stack_height: u32,
}

/// 账户更新数据 (来自 Yellowstone)
#[derive(Debug, Clone)]
pub struct AccountUpdate {
    /// 账户地址
    pub pubkey: Pubkey,
    /// 账户所有者 (程序ID)
    pub owner: Pubkey,
    /// 账户数据
    pub data: Vec<u8>,
    /// 余额 (lamports)
    pub lamports: u64,
    /// 是否可执行
    pub executable: bool,
    /// 租金纪元
    pub rent_epoch: u64,
}