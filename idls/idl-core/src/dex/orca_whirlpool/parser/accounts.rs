use crate::{
    Parser, ProgramParser, ParseResult, ParseError,
    AccountUpdate, Prefilter,
};

// 引用生成的Orca Whirlpool账户接口
use sol_whirlpool_interface::{
    parsers::accounts::{try_unpack_account, WhirlpoolAccount},
    ID as WHIRLPOOL_ID,
};

#[derive(Debug, Clone, Copy)]
pub struct AccountParser;

impl Parser for AccountParser {
    type Input = AccountUpdate;
    type Output = WhirlpoolAccount;

    fn id(&self) -> std::borrow::Cow<str> { "OrcaWhirlpool::AccountParser".into() }

    fn prefilter(&self) -> Prefilter {
        Prefilter::builder()
            .account_owners([WHIRLPOOL_ID])
            .build()
            .unwrap()
    }

    async fn parse(&self, account_update: &AccountUpdate) -> ParseResult<Self::Output> {
        if let Some(account) = &account_update.account {
            if account.owner.as_slice() == WHIRLPOOL_ID.to_bytes() {
                try_unpack_account(&account.data)
                    .map_err(|e| ParseError::from(e.to_string()))
            } else {
                Err(ParseError::Filtered)
            }
        } else {
            Err(ParseError::from("Account data is missing".to_string()))
        }
    }
}

impl ProgramParser for AccountParser {
    fn program_id(&self) -> crate::Pubkey { 
        WHIRLPOOL_ID.to_bytes().into() 
    }
}