use spl_token::instruction::TokenInstruction;
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
    type Output = TokenProgramIx;

    fn id(&self) -> std::borrow::Cow<str> { "token_program::InstructionParser".into() }

    fn prefilter(&self) -> Prefilter {
        Prefilter::builder()
            .transaction_accounts([spl_token::ID])
            .build()
            .unwrap()
    }

    async fn parse(&self, ix_update: &InstructionUpdate) -> ParseResult<Self::Output> {
        if ix_update.program.equals_ref(spl_token::ID) {
            InstructionParser::parse_impl(ix_update).map_err(|e| ParseError::Core(e))
        } else {
            Err(ParseError::Filtered)
        }
    }
}

impl ProgramParser for InstructionParser {
    #[inline]
    fn program_id(&self) -> crate::Pubkey { spl_token::ID.to_bytes().into() }
}

impl InstructionParser {
    #[allow(clippy::too_many_lines)]
    pub(crate) fn parse_impl(ix: &InstructionUpdate) -> crate::Result<TokenProgramIx> {
        let ix_type = TokenInstruction::unpack(&ix.data)
            .map_err(|e| crate::error::IdlCoreError::spl_token(format!("Error unpacking token instruction data: {}", e)))?;
        let accounts_len = ix.accounts.len();
        let ix = match ix_type {
            TokenInstruction::Transfer { amount } => {
                check_min_accounts_req(accounts_len, 3)?;
                Ok(TokenProgramIx::Transfer(
                    TransferAccounts {
                        source: into_vixen_pubkey(ix.accounts[0]),
                        destination: into_vixen_pubkey(ix.accounts[1]),
                        owner: into_vixen_pubkey(ix.accounts[2]),
                        multisig_signers: ix.accounts[3..].iter().map(|acc| into_vixen_pubkey(*acc)).collect(),
                    },
                    TransferData { amount },
                ))
            },
            TokenInstruction::InitializeAccount => {
                check_min_accounts_req(accounts_len, 3)?;
                Ok(TokenProgramIx::InitializeAccount(
                    InitializeAccountAccounts {
                        account: ix.accounts[0],
                        mint: ix.accounts[1],
                        owner: ix.accounts[2],
                    },
                ))
            },
            TokenInstruction::InitializeMint {
                decimals,
                mint_authority,
                freeze_authority,
            } => {
                check_min_accounts_req(accounts_len, 1)?;
                Ok(TokenProgramIx::InitializeMint(
                    InitializeMintAccounts {
                        mint: ix.accounts[0],
                    },
                    InitializeMintData {
                        decimals,
                        mint_authority: into_vixen_pubkey(mint_authority),
                        freeze_authority: freeze_authority.map(into_vixen_pubkey).into(),
                    },
                ))
            },
            TokenInstruction::InitializeMint2 {
                decimals,
                mint_authority,
                freeze_authority,
            } => {
                check_min_accounts_req(accounts_len, 1)?;
                Ok(TokenProgramIx::InitializeMint(
                    InitializeMintAccounts {
                        mint: ix.accounts[0],
                    },
                    InitializeMintData {
                        decimals,
                        mint_authority: into_vixen_pubkey(mint_authority),
                        freeze_authority: freeze_authority.map(into_vixen_pubkey).into(),
                    },
                ))
            },

            TokenInstruction::InitializeAccount2 { owner } => {
                check_min_accounts_req(accounts_len, 2)?;
                Ok(TokenProgramIx::InitializeAccount2(
                    InitializeAccount2Accounts {
                        account: ix.accounts[0],
                        mint: ix.accounts[1],
                    },
                    InitializeAccountData2 {
                        owner: into_vixen_pubkey(owner),
                    },
                ))
            },

            TokenInstruction::InitializeAccount3 { owner } => {
                check_min_accounts_req(accounts_len, 2)?;
                Ok(TokenProgramIx::InitializeAccount3(
                    InitializeAccount2Accounts {
                        account: ix.accounts[0],
                        mint: ix.accounts[1],
                    },
                    InitializeAccountData2 {
                        owner: into_vixen_pubkey(owner),
                    },
                ))
            },
            TokenInstruction::InitializeMultisig { m } => {
                check_min_accounts_req(accounts_len, 3)?;
                Ok(TokenProgramIx::InitializeMultisig(
                    InitializeMultisigAccounts {
                        multisig: ix.accounts[0],
                        signers: ix.accounts[2..].to_vec(),
                    },
                    InitializeMultisigData { m },
                ))
            },

            TokenInstruction::InitializeMultisig2 { m } => {
                check_min_accounts_req(accounts_len, 2)?;
                Ok(TokenProgramIx::InitializeMultisig(
                    InitializeMultisigAccounts {
                        multisig: ix.accounts[0],
                        signers: ix.accounts[1..].to_vec(),
                    },
                    InitializeMultisigData { m },
                ))
            },

            TokenInstruction::Approve { amount } => {
                check_min_accounts_req(accounts_len, 3)?;
                Ok(TokenProgramIx::Approve(
                    ApproveAccounts {
                        source: ix.accounts[0],
                        delegate: ix.accounts[1],
                        owner: ix.accounts[2],
                        multisig_signers: ix.accounts[3..].to_vec(),
                    },
                    ApproveData { amount },
                ))
            },

            TokenInstruction::Revoke => {
                check_min_accounts_req(accounts_len, 2)?;
                Ok(TokenProgramIx::Revoke(RevokeAccounts {
                    source: ix.accounts[0],
                    owner: ix.accounts[1],
                    multisig_signers: ix.accounts[2..].to_vec(),
                }))
            },

            TokenInstruction::SetAuthority {
                authority_type,
                new_authority,
            } => {
                check_min_accounts_req(accounts_len, 2)?;
                Ok(TokenProgramIx::SetAuthority(
                    SetAuthorityAccounts {
                        account: ix.accounts[0],
                        current_authority: ix.accounts[1],
                        multisig_signers: ix.accounts[2..].to_vec(),
                    },
                    SetAuthorityData {
                        authority_type,
                        new_authority: new_authority.map(into_vixen_pubkey).into(),
                    },
                ))
            },

            TokenInstruction::MintTo { amount } => {
                check_min_accounts_req(accounts_len, 3)?;
                Ok(TokenProgramIx::MintTo(
                    MintToAccounts {
                        mint: ix.accounts[0],
                        account: ix.accounts[1],
                        mint_authority: ix.accounts[2],
                        multisig_signers: ix.accounts[3..].to_vec(),
                    },
                    MintToData { amount },
                ))
            },

            TokenInstruction::Burn { amount } => {
                check_min_accounts_req(accounts_len, 3)?;
                Ok(TokenProgramIx::Burn(
                    BurnAccounts {
                        account: ix.accounts[0],
                        mint: ix.accounts[1],
                        owner: ix.accounts[2],
                        multisig_signers: ix.accounts[3..].to_vec(),
                    },
                    BurnData { amount },
                ))
            },

            TokenInstruction::CloseAccount => {
                check_min_accounts_req(accounts_len, 3)?;
                Ok(TokenProgramIx::CloseAccount(CloseAccountAccounts {
                    account: ix.accounts[0],
                    destination: ix.accounts[1],
                    owner: ix.accounts[2],
                    multisig_signers: ix.accounts[3..].to_vec(),
                }))
            },

            TokenInstruction::FreezeAccount => {
                check_min_accounts_req(accounts_len, 3)?;
                Ok(TokenProgramIx::FreezeAccount(FreezeAccountAccounts {
                    account: ix.accounts[0],
                    mint: ix.accounts[1],
                    mint_freeze_authority: ix.accounts[2],
                    multisig_signers: ix.accounts[3..].to_vec(),
                }))
            },

            TokenInstruction::ThawAccount => {
                check_min_accounts_req(accounts_len, 3)?;
                Ok(TokenProgramIx::ThawAccount(ThawAccountAccounts {
                    account: ix.accounts[0],
                    mint: ix.accounts[1],
                    mint_freeze_authority: ix.accounts[2],
                    multisig_signers: ix.accounts[3..].to_vec(),
                }))
            },

            TokenInstruction::TransferChecked { amount, decimals } => {
                check_min_accounts_req(accounts_len, 4)?;
                Ok(TokenProgramIx::TransferChecked(
                    TransferCheckedAccounts {
                        source: ix.accounts[0],
                        mint: ix.accounts[1],
                        destination: ix.accounts[2],
                        owner: ix.accounts[3],
                        multisig_signers: ix.accounts[4..].to_vec(),
                    },
                    TransferCheckedData { amount, decimals },
                ))
            },

            TokenInstruction::ApproveChecked { amount, decimals } => {
                check_min_accounts_req(accounts_len, 4)?;
                Ok(TokenProgramIx::ApproveChecked(
                    ApproveCheckedAccounts {
                        source: ix.accounts[0],
                        mint: ix.accounts[1],
                        delegate: ix.accounts[2],
                        owner: ix.accounts[3],
                        multisig_signers: ix.accounts[4..].to_vec(),
                    },
                    ApproveCheckedData { amount, decimals },
                ))
            },

            TokenInstruction::MintToChecked { amount, decimals } => {
                check_min_accounts_req(accounts_len, 3)?;
                Ok(TokenProgramIx::MintToChecked(
                    MintToCheckedAccounts {
                        mint: ix.accounts[0],
                        account: ix.accounts[1],
                        mint_authority: ix.accounts[2],
                        multisig_signers: ix.accounts[3..].to_vec(),
                    },
                    MintToCheckedData { amount, decimals },
                ))
            },

            TokenInstruction::BurnChecked { amount, decimals } => {
                //TODO : this ix needs 3 accounts , but only 1 account is available in the instruction
                check_min_accounts_req(accounts_len, 3)?;
                Ok(TokenProgramIx::BurnChecked(
                    BurnCheckedAccounts {
                        account: ix.accounts[0],
                        mint: ix.accounts[1],
                        owner: ix.accounts[2],
                        multisig_signers: ix.accounts[3..].to_vec(),
                    },
                    BurnCheckedData { amount, decimals },
                ))
            },

            TokenInstruction::SyncNative => {
                check_min_accounts_req(accounts_len, 1)?;
                Ok(TokenProgramIx::SyncNative(SyncNativeAccounts {
                    account: ix.accounts[0],
                }))
            },

            TokenInstruction::GetAccountDataSize => {
                check_min_accounts_req(accounts_len, 1)?;
                Ok(TokenProgramIx::GetAccountDataSize(
                    GetAccountDataSizeAccounts {
                        mint: ix.accounts[0],
                    },
                ))
            },

            TokenInstruction::InitializeImmutableOwner => {
                check_min_accounts_req(accounts_len, 1)?;
                Ok(TokenProgramIx::InitializeImmutableOwner(
                    InitializeImmutableOwnerAccounts {
                        account: ix.accounts[0],
                    },
                ))
            },

            TokenInstruction::AmountToUiAmount { amount } => {
                check_min_accounts_req(accounts_len, 1)?;
                Ok(TokenProgramIx::AmountToUiAmount(
                    AmountToUiAmountAccounts {
                        mint: ix.accounts[0],
                    },
                    AmountToUiAmountData { amount },
                ))
            },

            TokenInstruction::UiAmountToAmount { ui_amount } => {
                check_min_accounts_req(accounts_len, 1)?;
                Ok(TokenProgramIx::UiAmountToAmount(
                    UiAmountToAmountAccounts {
                        mint: ix.accounts[0],
                    },
                    UiAmountToAmountData {
                        ui_amount: ui_amount.into(),
                    },
                ))
            },
        };

        #[cfg(feature = "tracing")]
        match &ix {
            Ok(ix) => {
                tracing::info!(
                    name: "correctly_parsed_instruction",
                    name = "ix_update",
                    program = spl_token::ID.to_string(),
                    ix = ix.to_string()
                );
            },
            Err(e) => {
                tracing::info!(
                    name: "incorrectly_parsed_instruction",
                    name = "ix_update",
                    program = spl_token::ID.to_string(),
                    ix = "error",
                    // discriminator = ix_type,
                    error = ?e
                );
            },
        }

        ix
    }
}

