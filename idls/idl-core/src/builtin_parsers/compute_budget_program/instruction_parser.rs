use crate::{
    instruction::InstructionUpdate, ParseError, ParseResult, Parser, Prefilter, ProgramParser,
};

#[allow(clippy::wildcard_imports)]
use super::instruction_helpers::*;
use crate::builtin_parsers::helpers::check_min_accounts_req;

#[derive(Debug, Clone, Copy)]
pub struct InstructionParser;

// Compute Budget Program ID
const COMPUTE_BUDGET_PROGRAM_ID: [u8; 32] = [
    3, 6, 26, 217, 206, 221, 88, 97, 20, 96, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
];

impl Parser for InstructionParser {
    type Input = InstructionUpdate;
    type Output = ComputeBudgetProgramIx;

    fn id(&self) -> std::borrow::Cow<str> { "compute_budget_program::InstructionParser".into() }

    fn prefilter(&self) -> Prefilter {
        Prefilter::builder()
            .transaction_accounts([COMPUTE_BUDGET_PROGRAM_ID])
            .build()
            .unwrap()
    }

    async fn parse(&self, ix_update: &InstructionUpdate) -> ParseResult<Self::Output> {
        if ix_update.program.equals_ref(COMPUTE_BUDGET_PROGRAM_ID) {
            InstructionParser::parse_impl(ix_update).map_err(|e| ParseError::Other(e))
        } else {
            Err(ParseError::Filtered)
        }
    }
}

impl ProgramParser for InstructionParser {
    #[inline]
    fn program_id(&self) -> crate::Pubkey { 
        COMPUTE_BUDGET_PROGRAM_ID.into()
    }
}

impl InstructionParser {
    pub(crate) fn parse_impl(ix: &InstructionUpdate) -> Result<ComputeBudgetProgramIx, crate::BoxedError> {
        if ix.data.is_empty() {
            return Err("Empty compute budget instruction data".into());
        }

        let discriminator = ix.data[0];
        let data = &ix.data[1..];

        match discriminator {
            0 => {
                // RequestUnitsDeprecated - 8 bytes: u32 units + u32 additional_fee
                if data.len() != 8 {
                    return Err("Invalid RequestUnitsDeprecated data length".into());
                }
                let units = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                let additional_fee = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                Ok(ComputeBudgetProgramIx::RequestUnitsDeprecated(
                    RequestUnitsDeprecatedData { units, additional_fee }
                ))
            },
            1 => {
                // RequestHeapFrame - 4 bytes: u32 bytes
                if data.len() != 4 {
                    return Err("Invalid RequestHeapFrame data length".into());
                }
                let bytes = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                Ok(ComputeBudgetProgramIx::RequestHeapFrame(
                    RequestHeapFrameData { bytes }
                ))
            },
            2 => {
                // SetComputeUnitLimit - 4 bytes: u32 units
                if data.len() != 4 {
                    return Err("Invalid SetComputeUnitLimit data length".into());
                }
                let units = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                Ok(ComputeBudgetProgramIx::SetComputeUnitLimit(
                    SetComputeUnitLimitData { units }
                ))
            },
            3 => {
                // SetComputeUnitPrice - 8 bytes: u64 microlamports
                if data.len() != 8 {
                    return Err("Invalid SetComputeUnitPrice data length".into());
                }
                let microlamports = u64::from_le_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7]
                ]);
                Ok(ComputeBudgetProgramIx::SetComputeUnitPrice(
                    SetComputeUnitPriceData { microlamports }
                ))
            },
            _ => Err(format!("Unknown compute budget instruction discriminator: {}", discriminator).into()),
        }
    }
}