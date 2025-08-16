use crate::{
    Parser, ProgramParser, ParseResult, ParseError,
    AccountUpdate, Prefilter,
};

// 引用生成的Stable_Swap账户接口
use sol_stable_swap_interface::{
    parsers::accounts::{try_unpack_account, ProgramAccount},
    ID as ,
};

#[derive(Debug, Clone, Copy)]
pub struct AccountParser;

impl Parser for AccountParser {
    type Input = AccountUpdate;
    type Output = ProgramAccount;

    fn id(&self) -> std::borrow::Cow<str> { "Stable_Swap::AccountParser".into() }

    fn prefilter(&self) -> Prefilter {
        Prefilter::builder()
            .account_owners([])
            .build()
            .unwrap()
    }

    async fn parse(&self, account_update: &AccountUpdate) -> ParseResult<Self::Output> {
        if let Some(account) = &account_update.account {
            if account.owner.as_slice() == .to_bytes() {
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
        .to_bytes().into() 
    }
}