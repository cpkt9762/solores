/// Check minimum accounts requirement
pub fn check_min_accounts_req(actual: usize, expected: usize) -> crate::Result<()> {
    if actual < expected {
        Err(crate::error::IdlCoreError::insufficient_accounts(expected, actual))
    } else {
        Ok(())
    }
}

/// Convert array bytes to vixen pubkey
pub fn into_vixen_pubkey(bytes: impl AsRef<[u8]>) -> crate::Pubkey {
    let bytes = bytes.as_ref();
    if bytes.len() >= 32 {
        let mut pubkey_bytes = [0u8; 32];
        pubkey_bytes.copy_from_slice(&bytes[..32]);
        crate::Pubkey::new(pubkey_bytes)
    } else {
        crate::Pubkey::new([0u8; 32])
    }
}

/// Convert vixen pubkey to solana pubkey  
pub fn from_vixen_pubkey(pubkey: crate::Pubkey) -> solana_program::pubkey::Pubkey {
    solana_program::pubkey::Pubkey::new_from_array(pubkey.into_bytes())
}

// Utility functions for SPL programs
use solana_program::{program_option::COption, pubkey::Pubkey as SolanaPubkey};

pub trait FromCOptionPubkeyToOptString {
    fn to_opt_string(self) -> Option<String>;
}

pub trait FromVecPubkeyToVecString {
    fn to_string_vec(self) -> Vec<String>;
}

pub trait FromOptPubkeyToOptString {
    fn to_opt_string(self) -> Option<String>;
}

impl FromOptPubkeyToOptString for crate::Pubkey {
    fn to_opt_string(self) -> Option<String> { Some(self.to_string()) }
}

impl<T: ToString> FromVecPubkeyToVecString for Vec<T> {
    fn to_string_vec(self) -> Vec<String> { 
        self.into_iter().map(|p| p.to_string()).collect() 
    }
}

impl<A: ToString> FromOptPubkeyToOptString for Option<A> {
    fn to_opt_string(self) -> Option<String> { 
        self.map(|p| p.to_string()) 
    }
}

impl<A: ToString> FromCOptionPubkeyToOptString for COption<A> {
    fn to_opt_string(self) -> Option<String> {
        match self {
            COption::Some(val) => Some(val.to_string()),
            COption::None => None,
        }
    }
}
