use crate::instruction::InstructionUpdate;
use crate::error::{IdlCoreError, Result};

pub fn decode_extension_ix_type<T: TryFrom<u8>>(ix_data: &[u8]) -> Result<T>
where T::Error: std::error::Error + Send + Sync + 'static {
    T::try_from(ix_data[0]).map_err(|e| IdlCoreError::token_extension(format!("Error decoding instruction data: {}", e)))
}

pub trait ExtensionIxParser: Sized {
    fn try_parse_extension_ix(ix: &InstructionUpdate) -> Result<Self>;
}
