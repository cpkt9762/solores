use spl_token_2022::extension::confidential_transfer::instruction::ConfidentialTransferInstruction;
use crate::{instruction::InstructionUpdate, Pubkey};

use super::helpers::{decode_extension_ix_type, ExtensionIxParser};
use crate::builtin_parsers::helpers::check_min_accounts_req;
use crate::builtin_parsers::token_program::InitializeMintAccounts;
use crate::error::Result;

const SOLANA_ZK_PROOF_PROGRAM_ID: &str = "ZkTokenProof1111111111111111111111111111111";

#[derive(Debug, Clone, Copy)]
pub struct UpdateMintAccounts {
    pub mint: Pubkey,
    pub authority: Pubkey,
}

#[derive(Debug, Clone)]
pub struct ConfigureAccountAccounts {
    pub account: Pubkey,
    pub mint: Pubkey,
    pub sysvar: Pubkey,
    pub owner: Pubkey,
    pub multisig_signers: Vec<Pubkey>,
}

#[derive(Debug, Clone, Copy)]
pub struct ApproveAccountAccounts {
    pub account: Pubkey,
    pub mint: Pubkey,
    pub authority: Pubkey,
}

#[derive(Debug)]
pub struct EmptyAccountAccounts {
    pub account: Pubkey,
    pub sysvar: Pubkey,
    pub owner: Pubkey,
    pub multisig_signers: Vec<Pubkey>,
}

#[derive(Debug)]

pub struct DepositAccounts {
    pub account: Pubkey,
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub multisig_signers: Vec<Pubkey>,
}

#[derive(Debug)]
pub struct WithdrawAccounts {
    pub source_account: Pubkey,
    pub mint: Pubkey,
    pub destination: Pubkey,
    pub owner: Pubkey,
    pub multisig_signers: Vec<Pubkey>,
}

#[derive(Debug)]

pub struct ConfidentialTransferAccounts {
    pub source_account: Pubkey,
    pub mint: Pubkey,
    pub destination: Pubkey,
    pub owner: Pubkey,
    pub context_account: Pubkey, // Sysvar account or context state account
    pub multisig_signers: Vec<Pubkey>,
}

#[derive(Debug)]
pub struct ApplyPendingBalanceAccounts {
    pub account: Pubkey,
    pub owner: Pubkey,
    pub multisig_signers: Vec<Pubkey>,
}

#[derive(Debug)]

pub struct CreditsAccounts {
    pub account: Pubkey,
    pub owner: Pubkey,
    pub multisig_signers: Vec<Pubkey>,
}

#[derive(Debug)]
pub struct TransferWithSplitProofsAccounts {
    pub source_account: Pubkey,
    pub mint: Pubkey,
    pub destination: Pubkey,
    pub verify_ciphertext_commitment_equality_proof: Pubkey,
    pub verify_batched_grouped_cipher_text_2_handles_validity_proof: Pubkey,
    pub verify_batched_range_proof_u128: Option<Pubkey>,
    pub verify_batched_range_proof_u256: Option<Pubkey>,
    pub verify_batched_grouped_cipher_text_2_handles_validity_proof_next: Option<Pubkey>,
    pub verify_fee_sigma_proof: Option<Pubkey>,
    pub destination_account_for_lamports: Option<Pubkey>,
    pub context_state_account_owner: Option<Pubkey>,
    pub zk_token_proof_program: Option<Pubkey>,
    pub owner: Option<Pubkey>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum ConfidentaltransferIx {
    InitializeMint(InitializeMintAccounts),
    UpdateMint(UpdateMintAccounts),
    ConfigureAccount(ConfigureAccountAccounts),
    ApproveAccount(ApproveAccountAccounts),
    EmptyAccount(EmptyAccountAccounts),
    Deposit(DepositAccounts),
    Withdraw(WithdrawAccounts),
    Transfer(ConfidentialTransferAccounts),
    ApplyPendingBalance(ApplyPendingBalanceAccounts),
    EnableConfidentialCredits(CreditsAccounts),
    DisableConfidentialCredits(CreditsAccounts),
    EnableNonConfidentialCredits(CreditsAccounts),
    DisableNonConfidentialCredits(CreditsAccounts),
    TransferWithSplitProofs(TransferWithSplitProofsAccounts),
}

impl ExtensionIxParser for ConfidentaltransferIx {
    #[allow(clippy::too_many_lines)]
    fn try_parse_extension_ix(ix: &InstructionUpdate) -> Result<Self> {
        let accounts_len = ix.accounts.len();
        let ix_type = decode_extension_ix_type(&ix.data)?;
        match ix_type {
            ConfidentialTransferInstruction::InitializeMint => {
                check_min_accounts_req(accounts_len, 1)?;
                Ok(ConfidentaltransferIx::InitializeMint(
                    InitializeMintAccounts {
                        mint: ix.accounts[0],
                    },
                ))
            },
            ConfidentialTransferInstruction::UpdateMint => {
                check_min_accounts_req(accounts_len, 2)?;
                Ok(ConfidentaltransferIx::UpdateMint(UpdateMintAccounts {
                    mint: ix.accounts[0],
                    authority: ix.accounts[1],
                }))
            },
            ConfidentialTransferInstruction::ConfigureAccount => {
                check_min_accounts_req(accounts_len, 4)?;
                Ok(ConfidentaltransferIx::ConfigureAccount(
                    ConfigureAccountAccounts {
                        account: ix.accounts[0],
                        mint: ix.accounts[1],
                        sysvar: ix.accounts[2],
                        owner: ix.accounts[3],
                        multisig_signers: ix.accounts[4..].to_vec(),
                    },
                ))
            },

            ConfidentialTransferInstruction::ApproveAccount => {
                check_min_accounts_req(accounts_len, 3)?;
                Ok(ConfidentaltransferIx::ApproveAccount(
                    ApproveAccountAccounts {
                        account: ix.accounts[0],
                        mint: ix.accounts[1],
                        authority: ix.accounts[2],
                    },
                ))
            },

            ConfidentialTransferInstruction::EmptyAccount => {
                check_min_accounts_req(accounts_len, 3)?;
                Ok(ConfidentaltransferIx::EmptyAccount(EmptyAccountAccounts {
                    account: ix.accounts[0],
                    sysvar: ix.accounts[1],
                    owner: ix.accounts[2],
                    multisig_signers: ix.accounts[3..].to_vec(),
                }))
            },

            ConfidentialTransferInstruction::Deposit => {
                check_min_accounts_req(accounts_len, 3)?;
                Ok(ConfidentaltransferIx::Deposit(DepositAccounts {
                    account: ix.accounts[0],
                    mint: ix.accounts[1],
                    owner: ix.accounts[2],
                    multisig_signers: ix.accounts[3..].to_vec(),
                }))
            },

            ConfidentialTransferInstruction::Withdraw => {
                check_min_accounts_req(accounts_len, 4)?;
                Ok(ConfidentaltransferIx::Withdraw(WithdrawAccounts {
                    source_account: ix.accounts[0],
                    mint: ix.accounts[1],
                    destination: ix.accounts[2],
                    owner: ix.accounts[3],
                    multisig_signers: ix.accounts[4..].to_vec(),
                }))
            },

            ConfidentialTransferInstruction::Transfer => {
                check_min_accounts_req(accounts_len, 5)?;
                Ok(ConfidentaltransferIx::Transfer(
                    ConfidentialTransferAccounts {
                        source_account: ix.accounts[0],
                        mint: ix.accounts[1],
                        destination: ix.accounts[2],
                        context_account: ix.accounts[3],
                        owner: ix.accounts[4],
                        multisig_signers: ix.accounts[5..].to_vec(),
                    },
                ))
            },

            ConfidentialTransferInstruction::ApplyPendingBalance => {
                check_min_accounts_req(accounts_len, 2)?;
                Ok(ConfidentaltransferIx::ApplyPendingBalance(
                    ApplyPendingBalanceAccounts {
                        account: ix.accounts[0],
                        owner: ix.accounts[1],
                        multisig_signers: ix.accounts[2..].to_vec(),
                    },
                ))
            },

            ConfidentialTransferInstruction::EnableConfidentialCredits => {
                check_min_accounts_req(accounts_len, 2)?;
                Ok(ConfidentaltransferIx::EnableConfidentialCredits(
                    CreditsAccounts {
                        account: ix.accounts[0],
                        owner: ix.accounts[1],
                        multisig_signers: ix.accounts[2..].to_vec(),
                    },
                ))
            },

            ConfidentialTransferInstruction::DisableConfidentialCredits => {
                check_min_accounts_req(accounts_len, 2)?;
                Ok(ConfidentaltransferIx::DisableConfidentialCredits(
                    CreditsAccounts {
                        account: ix.accounts[0],
                        owner: ix.accounts[1],
                        multisig_signers: ix.accounts[2..].to_vec(),
                    },
                ))
            },

            ConfidentialTransferInstruction::EnableNonConfidentialCredits => {
                check_min_accounts_req(accounts_len, 2)?;
                Ok(ConfidentaltransferIx::EnableNonConfidentialCredits(
                    CreditsAccounts {
                        account: ix.accounts[0],
                        owner: ix.accounts[1],
                        multisig_signers: ix.accounts[2..].to_vec(),
                    },
                ))
            },

            ConfidentialTransferInstruction::DisableNonConfidentialCredits => {
                check_min_accounts_req(accounts_len, 2)?;
                Ok(ConfidentaltransferIx::DisableNonConfidentialCredits(
                    CreditsAccounts {
                        account: ix.accounts[0],
                        owner: ix.accounts[1],
                        multisig_signers: ix.accounts[2..].to_vec(),
                    },
                ))
            },

            ConfidentialTransferInstruction::TransferWithSplitProofs => {
                check_min_accounts_req(accounts_len, 13)?;

                match accounts_len {
                    7 => Ok(ConfidentaltransferIx::TransferWithSplitProofs(
                        TransferWithSplitProofsAccounts {
                            source_account: ix.accounts[0],
                            mint: ix.accounts[1],
                            destination: ix.accounts[2],
                            verify_ciphertext_commitment_equality_proof: ix.accounts[3],
                            verify_batched_grouped_cipher_text_2_handles_validity_proof: ix
                                .accounts[4],
                            verify_batched_range_proof_u128: Some(ix.accounts[5]),
                            owner: Some(ix.accounts[6]),
                            // Optional accounts
                            verify_batched_range_proof_u256: None,
                            verify_batched_grouped_cipher_text_2_handles_validity_proof_next: None,
                            verify_fee_sigma_proof: None,
                            destination_account_for_lamports: None,
                            context_state_account_owner: None,
                            zk_token_proof_program: None,
                        },
                    )),
                    9 => {
                        let ninth_account = ix.accounts[8];
                        if ninth_account.to_string() == SOLANA_ZK_PROOF_PROGRAM_ID {
                            Ok(ConfidentaltransferIx::TransferWithSplitProofs(
                                TransferWithSplitProofsAccounts {
                                    source_account: ix.accounts[0],
                                    mint: ix.accounts[1],
                                    destination: ix.accounts[2],
                                    verify_ciphertext_commitment_equality_proof: ix.accounts[3],
                                    verify_batched_grouped_cipher_text_2_handles_validity_proof: ix
                                    .accounts[4],
                                    verify_batched_range_proof_u128: Some(ix.accounts[5]),
                                    destination_account_for_lamports: Some(ix.accounts[6]),
                                    context_state_account_owner: Some(ix.accounts[7]),
                                    zk_token_proof_program: Some(ix.accounts[8]),

                                    // Optional accounts
                                    owner: None,
                                    verify_fee_sigma_proof: None,
                                    verify_batched_range_proof_u256: None,
                                    verify_batched_grouped_cipher_text_2_handles_validity_proof_next:
                                    None,
                                },
                            ))
                        } else {
                            Ok(ConfidentaltransferIx::TransferWithSplitProofs(
                                TransferWithSplitProofsAccounts {
                                    source_account: ix.accounts[0],
                                    mint: ix.accounts[1],
                                    destination: ix.accounts[2],
                                    verify_ciphertext_commitment_equality_proof: ix.accounts[3],
                                    verify_batched_grouped_cipher_text_2_handles_validity_proof: ix
                                    .accounts[4],
                                    verify_fee_sigma_proof: Some(ix.accounts[5]),
                                    verify_batched_range_proof_u256: Some(ix.accounts[6]),
                                    verify_batched_grouped_cipher_text_2_handles_validity_proof_next:
                                    Some(ix.accounts[7]),
                                    owner: Some(ix.accounts[8]),

                                    // Optional accounts
                                    verify_batched_range_proof_u128: None,
                                    destination_account_for_lamports: None,
                                    context_state_account_owner: None,
                                    zk_token_proof_program: None,
                                },
                            ))
                        }
                    },

                    11 => Ok(ConfidentaltransferIx::TransferWithSplitProofs(
                        TransferWithSplitProofsAccounts {
                            source_account: ix.accounts[0],
                            mint: ix.accounts[1],
                            destination: ix.accounts[2],
                            verify_ciphertext_commitment_equality_proof: ix.accounts[3],
                            verify_batched_grouped_cipher_text_2_handles_validity_proof: ix
                                .accounts[4],
                            verify_batched_range_proof_u256: Some(ix.accounts[5]),
                            verify_batched_grouped_cipher_text_2_handles_validity_proof_next: Some(
                                ix.accounts[6],
                            ),
                            verify_fee_sigma_proof: Some(ix.accounts[7]),
                            destination_account_for_lamports: Some(ix.accounts[8]),
                            context_state_account_owner: Some(ix.accounts[9]),
                            zk_token_proof_program: Some(ix.accounts[10]),
                            verify_batched_range_proof_u128: None,
                            owner: None,
                        },
                    )),

                    _ => Err(crate::error::IdlCoreError::token_extension(format!(
                        "Invalid number of accounts for TransferWithSplitProofs: {accounts_len}"
                    ))),
                }
            },
        }
    }
}

