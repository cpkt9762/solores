use crate::{
    Parser, ProgramParser, ParseResult, ParseError,
    instruction::InstructionUpdate, Prefilter,
};

// 引用生成的Raydium Launchpad接口
use sol_raydium_launchpad_interface::{
    parsers::instructions::{parse_instruction, RaydiumLaunchpadInstruction},
    ID as RAYDIUM_LAUNCHPAD_ID,
};

#[derive(Debug, Clone, Copy)]
pub struct InstructionParser;

impl Parser for InstructionParser {
    type Input = InstructionUpdate;
    type Output = RaydiumLaunchpadInstruction;

    fn id(&self) -> std::borrow::Cow<str> { "RaydiumLaunchpad::InstructionParser".into() }

    fn prefilter(&self) -> Prefilter {
        Prefilter::builder()
            .transaction_accounts([RAYDIUM_LAUNCHPAD_ID])
            .build()
            .unwrap()
    }

    async fn parse(&self, ix_update: &InstructionUpdate) -> ParseResult<Self::Output> {
        if ix_update.program.equals_ref(RAYDIUM_LAUNCHPAD_ID.to_bytes()) {
            // 转换账户格式
            let accounts: Vec<solana_pubkey::Pubkey> = ix_update.accounts
                .iter()
                .map(|addr| solana_pubkey::Pubkey::try_from(addr.as_slice()).unwrap_or_default())
                .collect();
                
            // 复用生成的解析函数
            parse_instruction(&ix_update.data, &accounts)
                .map_err(|e| ParseError::from(e.to_string()))
        } else {
            Err(ParseError::Filtered)
        }
    }
}

impl ProgramParser for InstructionParser {
    fn program_id(&self) -> crate::Pubkey { 
        RAYDIUM_LAUNCHPAD_ID.to_bytes().into() 
    }
}