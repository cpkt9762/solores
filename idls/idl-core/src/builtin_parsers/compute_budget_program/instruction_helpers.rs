use crate::Pubkey;

#[derive(Debug, Clone, Copy)]
pub struct RequestUnitsDeprecatedData {
    pub units: u32,
    pub additional_fee: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct RequestHeapFrameData {
    pub bytes: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct SetComputeUnitLimitData {
    pub units: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct SetComputeUnitPriceData {
    pub microlamports: u64,
}

/// Compute Budget Program instruction types
#[derive(Debug, Clone)]
pub enum ComputeBudgetProgramIx {
    RequestUnitsDeprecated(RequestUnitsDeprecatedData),
    RequestHeapFrame(RequestHeapFrameData),
    SetComputeUnitLimit(SetComputeUnitLimitData),
    SetComputeUnitPrice(SetComputeUnitPriceData),
}