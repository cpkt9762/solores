use solana_program::system_instruction::SystemInstruction;
use crate::{
    instruction::InstructionUpdate, ParseError, ParseResult, Parser, Prefilter, ProgramParser,
};

#[allow(clippy::wildcard_imports)]
use super::instruction_helpers::*;
use crate::builtin_parsers::helpers::{check_min_accounts_req, into_vixen_pubkey};

#[derive(Debug, Clone, Copy)]
pub struct InstructionParser;

impl Parser for InstructionParser {
    type Input = InstructionUpdate;
    type Output = SystemProgramIx;

    fn id(&self) -> std::borrow::Cow<str> { "system_program::InstructionParser".into() }

    fn prefilter(&self) -> Prefilter {
        Prefilter::builder()
            .transaction_accounts([solana_program::system_program::ID])
            .build()
            .unwrap()
    }

    async fn parse(&self, ix_update: &InstructionUpdate) -> ParseResult<Self::Output> {
        if ix_update.program.equals_ref(solana_program::system_program::ID) {
            InstructionParser::parse_impl(ix_update).map_err(|e| ParseError::Other(e.into()))
        } else {
            Err(ParseError::Filtered)
        }
    }
}

impl ProgramParser for InstructionParser {
    #[inline]
    fn program_id(&self) -> crate::Pubkey { 
        solana_program::system_program::ID.to_bytes().into() 
    }
}

impl InstructionParser {
    pub(crate) fn parse_impl(ix: &InstructionUpdate) -> crate::Result<SystemProgramIx> {
        let ix_type = bincode::deserialize::<SystemInstruction>(&ix.data)
            .map_err(|e| crate::error::IdlCoreError::system_program(format!("Failed to deserialize system instruction: {}", e)))?;
        let accounts_len = ix.accounts.len();
        
        let ix = match ix_type {
            SystemInstruction::CreateAccount { lamports, space, owner } => {
                check_min_accounts_req(accounts_len, 2)?;
                Ok(SystemProgramIx::CreateAccount(
                    CreateAccountAccounts {
                        from: into_vixen_pubkey(ix.accounts[0]),
                        to: into_vixen_pubkey(ix.accounts[1]),
                    },
                    CreateAccountData {
                        lamports,
                        space,
                        owner: owner.to_bytes().into(),
                    },
                ))
            },
            SystemInstruction::CreateAccountWithSeed { 
                base, seed, lamports, space, owner 
            } => {
                check_min_accounts_req(accounts_len, 3)?;
                Ok(SystemProgramIx::CreateAccountWithSeed(
                    CreateAccountWithSeedAccounts {
                        from: into_vixen_pubkey(ix.accounts[0]),
                        to: into_vixen_pubkey(ix.accounts[1]),
                        base: into_vixen_pubkey(ix.accounts[2]),
                    },
                    CreateAccountWithSeedData {
                        base: base.to_bytes().into(),
                        seed,
                        lamports,
                        space,
                        owner: owner.to_bytes().into(),
                    },
                ))
            },
            SystemInstruction::Transfer { lamports } => {
                check_min_accounts_req(accounts_len, 2)?;
                Ok(SystemProgramIx::Transfer(
                    TransferAccounts {
                        from: into_vixen_pubkey(ix.accounts[0]),
                        to: into_vixen_pubkey(ix.accounts[1]),
                    },
                    TransferData { lamports },
                ))
            },
            SystemInstruction::TransferWithSeed { 
                lamports, from_seed, from_owner 
            } => {
                check_min_accounts_req(accounts_len, 3)?;
                Ok(SystemProgramIx::TransferWithSeed(
                    TransferWithSeedAccounts {
                        from: into_vixen_pubkey(ix.accounts[0]),
                        from_base: into_vixen_pubkey(ix.accounts[1]),
                        to: into_vixen_pubkey(ix.accounts[2]),
                    },
                    TransferWithSeedData {
                        lamports,
                        from_seed,
                        from_owner: from_owner.to_bytes().into(),
                    },
                ))
            },
            SystemInstruction::AdvanceNonceAccount => {
                check_min_accounts_req(accounts_len, 3)?;
                Ok(SystemProgramIx::AdvanceNonceAccount(
                    AdvanceNonceAccountAccounts {
                        nonce_account: into_vixen_pubkey(ix.accounts[0]),
                        recent_blockhashes_sysvar: into_vixen_pubkey(ix.accounts[1]),
                        nonce_authority: into_vixen_pubkey(ix.accounts[2]),
                    },
                ))
            },
            SystemInstruction::WithdrawNonceAccount(lamports) => {
                check_min_accounts_req(accounts_len, 5)?;
                Ok(SystemProgramIx::WithdrawNonceAccount(
                    WithdrawNonceAccountAccounts {
                        nonce_account: into_vixen_pubkey(ix.accounts[0]),
                        to: into_vixen_pubkey(ix.accounts[1]),
                        recent_blockhashes_sysvar: into_vixen_pubkey(ix.accounts[2]),
                        rent_sysvar: into_vixen_pubkey(ix.accounts[3]),
                        nonce_authority: into_vixen_pubkey(ix.accounts[4]),
                    },
                    WithdrawNonceAccountData { lamports },
                ))
            },
            SystemInstruction::Allocate { space } => {
                check_min_accounts_req(accounts_len, 1)?;
                Ok(SystemProgramIx::Allocate(
                    AllocateAccounts {
                        account: into_vixen_pubkey(ix.accounts[0]),
                    },
                    AllocateData { space },
                ))
            },
            SystemInstruction::AllocateWithSeed { 
                base, seed, space, owner 
            } => {
                check_min_accounts_req(accounts_len, 2)?;
                Ok(SystemProgramIx::AllocateWithSeed(
                    AllocateWithSeedAccounts {
                        account: into_vixen_pubkey(ix.accounts[0]),
                        base: into_vixen_pubkey(ix.accounts[1]),
                    },
                    AllocateWithSeedData {
                        base: base.to_bytes().into(),
                        seed,
                        space,
                        owner: owner.to_bytes().into(),
                    },
                ))
            },
            SystemInstruction::Assign { owner } => {
                check_min_accounts_req(accounts_len, 1)?;
                Ok(SystemProgramIx::Assign(
                    AssignAccounts {
                        account: into_vixen_pubkey(ix.accounts[0]),
                    },
                    AssignData {
                        owner: owner.to_bytes().into(),
                    },
                ))
            },
            SystemInstruction::AssignWithSeed { 
                base, seed, owner 
            } => {
                check_min_accounts_req(accounts_len, 2)?;
                Ok(SystemProgramIx::AssignWithSeed(
                    AssignWithSeedAccounts {
                        account: into_vixen_pubkey(ix.accounts[0]),
                        base: into_vixen_pubkey(ix.accounts[1]),
                    },
                    AssignWithSeedData {
                        base: base.to_bytes().into(),
                        seed,
                        owner: owner.to_bytes().into(),
                    },
                ))
            },
            _ => Err("Unsupported system instruction".into()),
        };

        ix
    }
}