use crate::Pubkey;

#[derive(Debug, Clone)]
pub struct CreateAccountAccounts {
    pub from: Pubkey,
    pub to: Pubkey,
}

#[derive(Debug, Clone)]
pub struct CreateAccountData {
    pub lamports: u64,
    pub space: u64,
    pub owner: Pubkey,
}

#[derive(Debug, Clone)]
pub struct CreateAccountWithSeedAccounts {
    pub from: Pubkey,
    pub to: Pubkey,
    pub base: Pubkey,
}

#[derive(Debug, Clone)]
pub struct CreateAccountWithSeedData {
    pub base: Pubkey,
    pub seed: String,
    pub lamports: u64,
    pub space: u64,
    pub owner: Pubkey,
}

#[derive(Debug, Clone)]
pub struct TransferAccounts {
    pub from: Pubkey,
    pub to: Pubkey,
}

#[derive(Debug, Clone)]
pub struct TransferData {
    pub lamports: u64,
}

#[derive(Debug, Clone)]
pub struct TransferWithSeedAccounts {
    pub from: Pubkey,
    pub from_base: Pubkey,
    pub to: Pubkey,
}

#[derive(Debug, Clone)]
pub struct TransferWithSeedData {
    pub lamports: u64,
    pub from_seed: String,
    pub from_owner: Pubkey,
}

#[derive(Debug, Clone)]
pub struct AdvanceNonceAccountAccounts {
    pub nonce_account: Pubkey,
    pub recent_blockhashes_sysvar: Pubkey,
    pub nonce_authority: Pubkey,
}

#[derive(Debug, Clone)]
pub struct WithdrawNonceAccountAccounts {
    pub nonce_account: Pubkey,
    pub to: Pubkey,
    pub recent_blockhashes_sysvar: Pubkey,
    pub rent_sysvar: Pubkey,
    pub nonce_authority: Pubkey,
}

#[derive(Debug, Clone)]
pub struct WithdrawNonceAccountData {
    pub lamports: u64,
}

#[derive(Debug, Clone)]
pub struct AllocateAccounts {
    pub account: Pubkey,
}

#[derive(Debug, Clone)]
pub struct AllocateData {
    pub space: u64,
}

#[derive(Debug, Clone)]
pub struct AllocateWithSeedAccounts {
    pub account: Pubkey,
    pub base: Pubkey,
}

#[derive(Debug, Clone)]
pub struct AllocateWithSeedData {
    pub base: Pubkey,
    pub seed: String,
    pub space: u64,
    pub owner: Pubkey,
}

#[derive(Debug, Clone)]
pub struct AssignAccounts {
    pub account: Pubkey,
}

#[derive(Debug, Clone)]
pub struct AssignData {
    pub owner: Pubkey,
}

#[derive(Debug, Clone)]
pub struct AssignWithSeedAccounts {
    pub account: Pubkey,
    pub base: Pubkey,
}

#[derive(Debug, Clone)]
pub struct AssignWithSeedData {
    pub base: Pubkey,
    pub seed: String,
    pub owner: Pubkey,
}

/// System Program instruction types
#[derive(Debug, Clone)]
pub enum SystemProgramIx {
    CreateAccount(CreateAccountAccounts, CreateAccountData),
    CreateAccountWithSeed(CreateAccountWithSeedAccounts, CreateAccountWithSeedData),
    Transfer(TransferAccounts, TransferData),
    TransferWithSeed(TransferWithSeedAccounts, TransferWithSeedData),
    AdvanceNonceAccount(AdvanceNonceAccountAccounts),
    WithdrawNonceAccount(WithdrawNonceAccountAccounts, WithdrawNonceAccountData),
    Allocate(AllocateAccounts, AllocateData),
    AllocateWithSeed(AllocateWithSeedAccounts, AllocateWithSeedData),
    Assign(AssignAccounts, AssignData),
    AssignWithSeed(AssignWithSeedAccounts, AssignWithSeedData),
}