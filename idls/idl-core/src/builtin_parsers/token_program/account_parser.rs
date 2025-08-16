use std::borrow::Cow;

use spl_token::{
    solana_program::{program_error::ProgramError, program_pack::Pack},
    state::{Account, Mint, Multisig},
};
use crate::{
    AccountUpdate, ParseError, ParseResult, Parser, Prefilter, ProgramParser,
};

#[derive(Debug)]
#[cfg_attr(feature = "tracing", derive(strum_macros::Display))]
pub enum TokenProgramState {
    TokenAccount(Account),
    Mint(Mint),
    Multisig(Multisig),
}

impl TokenProgramState {
    pub fn try_unpack(data_bytes: &[u8]) -> ParseResult<Self> {
        let acc = match data_bytes.len() {
            Mint::LEN => Mint::unpack(data_bytes).map(Self::Mint).map_err(Into::into),
            Account::LEN => Account::unpack(data_bytes)
                .map(Self::TokenAccount)
                .map_err(Into::into),
            Multisig::LEN => Multisig::unpack(data_bytes)
                .map(Self::Multisig)
                .map_err(Into::into),
            _ => Err(ParseError::from("Invalid Account data length".to_owned())),
        };

        #[cfg(feature = "tracing")]
        match &acc {
            Ok(acc) => {
                tracing::info!(
                    name: "correctly_parsed_account",
                    name = "account_update",
                    program = spl_token::ID.to_string(),
                    account = acc.to_string()
                );
            },
            Err(e) => {
                tracing::info!(
                    name: "incorrectly_parsed_account",
                    name = "account_update",
                    program = spl_token::ID.to_string(),
                    account = "error",
                    discriminator = ?data_bytes.len(),
                    error = ?e
                );
            },
        }

        acc
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AccountParser;

impl Parser for AccountParser {
    type Input = AccountUpdate;
    type Output = TokenProgramState;

    fn id(&self) -> Cow<str> { "token_program::AccountParser".into() }

    fn prefilter(&self) -> Prefilter {
        Prefilter::builder()
            .account_owners([spl_token::ID])
            .build()
            .unwrap()
    }

    async fn parse(&self, acct: &AccountUpdate) -> ParseResult<Self::Output> {
        let inner = acct.account.as_ref().ok_or(ProgramError::InvalidArgument)?;

        TokenProgramState::try_unpack(&inner.data)
    }
}

impl ProgramParser for AccountParser {
    #[inline]
    fn program_id(&self) -> crate::Pubkey { spl_token::ID.to_bytes().into() }
}

