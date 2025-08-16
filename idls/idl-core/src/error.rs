//! Error types for idl-core
//! 
//! This module provides comprehensive error handling using thiserror for better
//! error messages and debugging experience.

use thiserror::Error;

/// Core error types for idl-core parsing operations
#[derive(Error, Debug)]
pub enum IdlCoreError {
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Invalid instruction data: {0}")]
    InvalidInstructionData(String),
    
    #[error("Insufficient accounts: expected {expected}, got {actual}")]
    InsufficientAccounts { expected: usize, actual: usize },
    
    #[error("Invalid program ID: expected {expected}, got {actual}")]
    InvalidProgramId { expected: String, actual: String },
    
    #[error("Token extension error: {0}")]
    TokenExtensionError(String),
    
    #[error("System program error: {0}")]
    SystemProgramError(String),
    
    #[error("SPL Token error: {0}")]
    SplTokenError(String),
    
    #[error("SPL Token 2022 error: {0}")]
    SplToken2022Error(String),
    
    #[error("Compute Budget error: {0}")]
    ComputeBudgetError(String),
    
    #[error("Memo program error: {0}")]
    MemoError(String),
    
    #[error("Account data missing")]
    AccountDataMissing,
    
    #[error("Empty instruction data")]
    EmptyInstructionData,
    
    #[error("Invalid discriminator: {0:?}")]
    InvalidDiscriminator(Vec<u8>),
    
    #[error("Failed to decode transaction data: {0}")]
    DecodeError(String),
    
    #[error("Unsupported encoding format")]
    UnsupportedEncoding,
    
    #[error("JSON error: {0}")]
    JsonError(String),
    
    #[error("Invalid UTF-8 in memo: {0}")]
    InvalidUtf8(#[from] std::str::Utf8Error),
    
    #[error("Bincode error: {0}")]
    BincodeError(#[from] bincode::Error),
    
    #[error("Base58 decode error: {0}")]
    Base58Error(#[from] bs58::decode::Error),
    
    #[error("Base64 decode error: {0}")]
    Base64Error(String),
    
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

/// Parser error types with filtering support
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Parser filtered this instruction (normal behavior)")]
    Filtered,
    
    #[error("Core parsing error: {0}")]
    Core(#[from] IdlCoreError),
    
    #[error("Other parsing error: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync + 'static>),
}

// Type aliases for convenience and backward compatibility
pub type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T> = std::result::Result<T, IdlCoreError>;
pub type ParseResult<T> = std::result::Result<T, ParseError>;

// Additional compatibility aliases for lib.rs
pub type PrefilterResult<T> = std::result::Result<T, PrefilterError>;
pub type KeyFromSliceError = std::array::TryFromSliceError;

// Conversion helpers - avoiding conflicting implementations
impl From<BoxedError> for ParseError {
    fn from(value: BoxedError) -> Self {
        Self::Other(value)
    }
}

impl From<String> for ParseError {
    fn from(value: String) -> Self {
        Self::Other(value.into())
    }
}

impl From<solana_program::program_error::ProgramError> for ParseError {
    fn from(value: solana_program::program_error::ProgramError) -> Self {
        Self::Other(Box::new(value))
    }
}

impl From<crate::instruction::ParseError> for ParseError {
    fn from(value: crate::instruction::ParseError) -> Self {
        match value {
            crate::instruction::ParseError::Missing(m) => {
                Self::Core(IdlCoreError::ParseError(format!("Missing field: {:?}", m)))
            }
            crate::instruction::ParseError::InvalidInnerInstructionIndex(i) => {
                Self::Core(IdlCoreError::ParseError(format!("Invalid inner instruction index: {}", i)))
            }
            crate::instruction::ParseError::AccountKey(e) => {
                Self::Other(Box::new(e))
            }
            crate::instruction::ParseError::DecodeError(s) => {
                Self::Core(IdlCoreError::DecodeError(s))
            }
            crate::instruction::ParseError::UnsupportedEncoding => {
                Self::Core(IdlCoreError::UnsupportedEncoding)
            }
            crate::instruction::ParseError::JsonError(s) => {
                Self::Core(IdlCoreError::JsonError(s))
            }
            crate::instruction::ParseError::Base64Error(e) => {
                Self::Core(IdlCoreError::Base58Error(e))
            }
        }
    }
}

impl From<String> for IdlCoreError {
    fn from(value: String) -> Self {
        Self::ParseError(value)
    }
}

impl From<&str> for IdlCoreError {
    fn from(value: &str) -> Self {
        Self::ParseError(value.to_string())
    }
}

// Helper for serde_json errors
impl From<serde_json::Error> for IdlCoreError {
    fn from(value: serde_json::Error) -> Self {
        Self::JsonError(value.to_string())
    }
}

impl From<std::array::TryFromSliceError> for IdlCoreError {
    fn from(value: std::array::TryFromSliceError) -> Self {
        Self::InvalidInstructionData(format!("Array conversion error: {}", value))
    }
}

// Helper for better error context
impl IdlCoreError {
    /// Create an insufficient accounts error
    pub fn insufficient_accounts(expected: usize, actual: usize) -> Self {
        Self::InsufficientAccounts { expected, actual }
    }
    
    /// Create a token extension error with context
    pub fn token_extension<S: Into<String>>(msg: S) -> Self {
        Self::TokenExtensionError(msg.into())
    }
    
    /// Create a system program error with context
    pub fn system_program<S: Into<String>>(msg: S) -> Self {
        Self::SystemProgramError(msg.into())
    }
    
    /// Create an SPL token error with context
    pub fn spl_token<S: Into<String>>(msg: S) -> Self {
        Self::SplTokenError(msg.into())
    }
    
    /// Create an invalid instruction data error
    pub fn invalid_instruction_data<S: Into<String>>(msg: S) -> Self {
        Self::InvalidInstructionData(msg.into())
    }
}

// Compatibility with existing error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum PrefilterError {
    #[error("Value already given for field {0}")]
    AlreadySet(&'static str),
    #[error("Invalid pubkey {}", bs58::encode(.0).into_string())]
    BadPubkey(Vec<u8>, std::array::TryFromSliceError),
}

impl From<PrefilterError> for IdlCoreError {
    fn from(value: PrefilterError) -> Self {
        match value {
            PrefilterError::AlreadySet(field) => {
                IdlCoreError::ParseError(format!("Field already set: {}", field))
            }
            PrefilterError::BadPubkey(bytes, err) => {
                IdlCoreError::InvalidInstructionData(format!("Bad pubkey: {:?}, error: {}", bytes, err))
            }
        }
    }
}

// KeyFromStrError compatibility
#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum KeyFromStrError<const LEN: usize = 32> {
    #[error("Invalid base58 string")]
    Bs58(#[from] bs58::decode::Error),
    #[error("Invalid key length, must be {LEN} bytes")]
    Len(#[from] std::array::TryFromSliceError),
}

impl<const LEN: usize> From<KeyFromStrError<LEN>> for IdlCoreError {
    fn from(value: KeyFromStrError<LEN>) -> Self {
        match value {
            KeyFromStrError::Bs58(e) => IdlCoreError::Base58Error(e),
            KeyFromStrError::Len(e) => IdlCoreError::InvalidInstructionData(format!("Invalid key length: {}", e)),
        }
    }
}