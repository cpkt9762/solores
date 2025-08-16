use crate::{
    Parser, ProgramParser, ParseResult, ParseError,
    AccountUpdate, Prefilter,
};

// 引用生成的Raydium AMM账户接口
use sol_raydium_interface::{
    parsers::accounts::{try_unpack_account, RaydiumAccount},
    ID as RAYDIUM_AMM_ID,
};

#[derive(Debug, Clone, Copy)]
pub struct AccountParser;

impl Parser for AccountParser {
    type Input = AccountUpdate;
    type Output = RaydiumAccount;

    fn id(&self) -> std::borrow::Cow<str> { "Raydium::AmmAccountParser".into() }

    fn prefilter(&self) -> Prefilter {
        Prefilter::builder()
            .account_owners([RAYDIUM_AMM_ID])
            .build()
            .unwrap()
    }

    async fn parse(&self, account_update: &AccountUpdate) -> ParseResult<Self::Output> {
        if let Some(account) = &account_update.account {
            if account.owner.as_slice() == RAYDIUM_AMM_ID.to_bytes() {
                // 复用生成的账户解析函数
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
        RAYDIUM_AMM_ID.to_bytes().into() 
    }
}