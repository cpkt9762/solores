use crate::{
    Parser, ProgramParser, ParseResult, ParseError,
    AccountUpdate, Prefilter,
};

// 引用生成的PumpFun账户接口
use sol_pumpfun_interface::{
    parsers::accounts::{try_unpack_account, PumpfunAccount},
    ID as PUMPFUN_ID,
};

#[derive(Debug, Clone, Copy)]
pub struct AccountParser;

impl Parser for AccountParser {
    type Input = AccountUpdate;
    type Output = PumpfunAccount;

    fn id(&self) -> std::borrow::Cow<str> { "PumpFun::AccountParser".into() }

    fn prefilter(&self) -> Prefilter {
        Prefilter::builder()
            .account_owners([PUMPFUN_ID])
            .build()
            .unwrap()
    }

    async fn parse(&self, account_update: &AccountUpdate) -> ParseResult<Self::Output> {
        if let Some(account) = &account_update.account {
            if account.owner.as_slice() == PUMPFUN_ID.to_bytes() {
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
        PUMPFUN_ID.to_bytes().into() 
    }
}