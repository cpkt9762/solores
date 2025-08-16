use crate::{
    instruction::InstructionUpdate, ParseError, ParseResult, Parser, Prefilter, ProgramParser,
};

#[allow(clippy::wildcard_imports)]
use super::instruction_helpers::*;
use crate::builtin_parsers::helpers::into_vixen_pubkey;

#[derive(Debug, Clone, Copy)]
pub struct InstructionParser;

// Memo Program ID: MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr
const MEMO_PROGRAM_ID: [u8; 32] = [
    5, 78, 193, 174, 4, 69, 1, 191, 212, 127, 185, 21, 204, 91, 84, 75, 65, 251, 123, 201, 151, 
    220, 105, 138, 126, 217, 51, 212, 126, 93, 166, 165
];

impl Parser for InstructionParser {
    type Input = InstructionUpdate;
    type Output = MemoProgramIx;

    fn id(&self) -> std::borrow::Cow<str> { "memo_program::InstructionParser".into() }

    fn prefilter(&self) -> Prefilter {
        Prefilter::builder()
            .transaction_accounts([MEMO_PROGRAM_ID])
            .build()
            .unwrap()
    }

    async fn parse(&self, ix_update: &InstructionUpdate) -> ParseResult<Self::Output> {
        if ix_update.program.equals_ref(MEMO_PROGRAM_ID) {
            InstructionParser::parse_impl(ix_update).map_err(|e| ParseError::Other(e))
        } else {
            Err(ParseError::Filtered)
        }
    }
}

impl ProgramParser for InstructionParser {
    #[inline]
    fn program_id(&self) -> crate::Pubkey { 
        MEMO_PROGRAM_ID.into()
    }
}

impl InstructionParser {
    pub(crate) fn parse_impl(ix: &InstructionUpdate) -> Result<MemoProgramIx, crate::BoxedError> {
        // Memo program is simple: all instruction data is memo content
        // All accounts are signers that must sign the transaction
        
        // Validate memo is valid UTF-8 (optional, but good practice)
        let _memo_text = std::str::from_utf8(&ix.data)
            .map_err(|e| format!("Invalid UTF-8 memo: {}", e))?;

        Ok(MemoProgramIx::WriteMemo(
            WriteMemoAccounts {
                signers: ix.accounts.iter().map(|&addr| into_vixen_pubkey(addr)).collect(),
            },
            WriteMemoData {
                memo: ix.data.clone(),
            },
        ))
    }
}