use crate::Pubkey;

#[derive(Debug, Clone)]
pub struct WriteMemoAccounts {
    pub signers: Vec<Pubkey>,
}

#[derive(Debug, Clone)]
pub struct WriteMemoData {
    pub memo: Vec<u8>,
}

/// Memo Program instruction types
#[derive(Debug, Clone)]
pub enum MemoProgramIx {
    WriteMemo(WriteMemoAccounts, WriteMemoData),
}