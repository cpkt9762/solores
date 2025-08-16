use spl_token_2022::extension::confidential_transfer_fee::instruction::ConfidentialTransferFeeInstruction;
use crate::{instruction::InstructionUpdate, Pubkey};

use super::helpers::{decode_extension_ix_type, ExtensionIxParser};
use crate::{builtin_parsers::helpers::check_min_accounts_req, Result};
#[derive(Debug, Clone, Copy)]
pub struct InitializeConfidentialTransferFeeConfigAccounts {
    pub mint: Pubkey,
}

#[derive(Debug)]
pub struct ConfidentialWithdrawWithheldTokensFromMintAccounts {
    pub mint: Pubkey,
    pub fee_recipient: Pubkey,
    pub sysvar: Pubkey,
    pub withdraw_withheld_authority: Pubkey,
    pub multisig_signers: Vec<Pubkey>,
}

#[derive(Debug)]
pub struct ConfidentialWithdrawWithheldTokensFromAccounts {
    pub mint: Pubkey,
    pub fee_recipient: Pubkey,
    pub sysvar: Pubkey,
    pub withdraw_withheld_authority: Pubkey,
    pub multisig_signers: Vec<Pubkey>,
    pub source_accounts: Vec<Pubkey>,
}

#[derive(Debug)]
pub struct ConfidentialHarvestWithheldTokensToMintAccounts {
    pub mint: Pubkey,
    pub source_accounts: Vec<Pubkey>,
}

#[derive(Debug)]
pub struct EnableHarvestToMintAccounts {
    pub mint: Pubkey,
    pub confidential_transfer_fee_authority: Pubkey,
    pub multisig_signers: Vec<Pubkey>,
}

#[derive(Debug)]
pub struct DisableHarvestToMintAccounts {
    pub account: Pubkey,
    pub confidential_transfer_fee_authority: Pubkey,
    pub multisig_signers: Vec<Pubkey>,
}

#[derive(Debug)]
pub enum ConfidentaltransferFeeIx {
    InitializeConfidentialTransferFeeConfig(InitializeConfidentialTransferFeeConfigAccounts),
    WithdrawWithheldTokensFromMint(ConfidentialWithdrawWithheldTokensFromMintAccounts),
    WithdrawWithheldTokensFromAccounts(ConfidentialWithdrawWithheldTokensFromAccounts),
    HarvestWithheldTokensToMint(ConfidentialHarvestWithheldTokensToMintAccounts),
    EnableHarvestToMint(EnableHarvestToMintAccounts),
    DisableHarvestToMint(DisableHarvestToMintAccounts),
}

impl ExtensionIxParser for ConfidentaltransferFeeIx {
    fn try_parse_extension_ix(ix: &InstructionUpdate) -> Result<Self> {
        let accounts_len = ix.accounts.len();
        let ix_type = decode_extension_ix_type(&ix.data)?;

        match ix_type {
            ConfidentialTransferFeeInstruction::InitializeConfidentialTransferFeeConfig => {
                check_min_accounts_req(accounts_len, 1)?;
                Ok(
                    ConfidentaltransferFeeIx::InitializeConfidentialTransferFeeConfig(
                        InitializeConfidentialTransferFeeConfigAccounts {
                            mint: ix.accounts[0],
                        },
                    ),
                )
            },

            ConfidentialTransferFeeInstruction::WithdrawWithheldTokensFromMint => {
                check_min_accounts_req(accounts_len, 4)?;
                Ok(ConfidentaltransferFeeIx::WithdrawWithheldTokensFromMint(
                    ConfidentialWithdrawWithheldTokensFromMintAccounts {
                        mint: ix.accounts[0],
                        fee_recipient: ix.accounts[1],
                        sysvar: ix.accounts[2],
                        withdraw_withheld_authority: ix.accounts[3],
                        multisig_signers: ix.accounts[4..].to_vec(),
                    },
                ))
            },

            ConfidentialTransferFeeInstruction::WithdrawWithheldTokensFromAccounts => {
                check_min_accounts_req(accounts_len, 5)?;
                Ok(
                    ConfidentaltransferFeeIx::WithdrawWithheldTokensFromAccounts(
                        ConfidentialWithdrawWithheldTokensFromAccounts {
                            mint: ix.accounts[0],
                            fee_recipient: ix.accounts[1],
                            sysvar: ix.accounts[2],
                            withdraw_withheld_authority: ix.accounts[3],
                            source_accounts: ix.accounts[4..].to_vec(),
                            multisig_signers: Vec::new(),
                        },
                    ),
                )
            },

            ConfidentialTransferFeeInstruction::HarvestWithheldTokensToMint => {
                check_min_accounts_req(accounts_len, 2)?;
                Ok(ConfidentaltransferFeeIx::HarvestWithheldTokensToMint(
                    ConfidentialHarvestWithheldTokensToMintAccounts {
                        mint: ix.accounts[0],
                        source_accounts: ix.accounts[1..].to_vec(),
                    },
                ))
            },

            ConfidentialTransferFeeInstruction::EnableHarvestToMint => {
                check_min_accounts_req(accounts_len, 2)?;
                Ok(ConfidentaltransferFeeIx::EnableHarvestToMint(
                    EnableHarvestToMintAccounts {
                        mint: ix.accounts[0],
                        confidential_transfer_fee_authority: ix.accounts[1],
                        multisig_signers: ix.accounts[2..].to_vec(),
                    },
                ))
            },

            ConfidentialTransferFeeInstruction::DisableHarvestToMint => {
                check_min_accounts_req(accounts_len, 2)?;
                Ok(ConfidentaltransferFeeIx::DisableHarvestToMint(
                    DisableHarvestToMintAccounts {
                        account: ix.accounts[0],
                        confidential_transfer_fee_authority: ix.accounts[1],
                        multisig_signers: ix.accounts[2..].to_vec(),
                    },
                ))
            },
        }
    }
}

