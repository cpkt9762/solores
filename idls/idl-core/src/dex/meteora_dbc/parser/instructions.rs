use crate::{
    Parser, ProgramParser, ParseResult, ParseError,
    instruction::InstructionUpdate, Prefilter,
};

// 直接引用生成的Meteora DBC接口
use sol_meteora_dbc_interface::{
    parsers::instructions::{parse_instruction, MeteoraDbcInstruction},
    ID as METEORA_DBC_ID,
};

#[derive(Debug, Clone, Copy)]
pub struct InstructionParser;

impl Parser for InstructionParser {
    type Input = InstructionUpdate;
    type Output = MeteoraDbcInstruction;

    fn id(&self) -> std::borrow::Cow<str> { "MeteoraDbc::InstructionParser".into() }

    fn prefilter(&self) -> Prefilter {
        Prefilter::builder()
            .transaction_accounts([METEORA_DBC_ID])
            .build()
            .unwrap()
    }

    async fn parse(&self, ix_update: &InstructionUpdate) -> ParseResult<Self::Output> {
        if ix_update.program.equals_ref(METEORA_DBC_ID.to_bytes()) {
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
        METEORA_DBC_ID.to_bytes().into() 
    }
}