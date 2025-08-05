#![doc = include_str!("./README.md")]

use proc_macro2::TokenStream;
use toml::{map::Map, Value};

pub mod anchor;
pub mod bincode;
pub mod shank;

pub trait IdlCodegenModule {
    /// The module file's name e.g. "errors"
    fn name(&self) -> &str;

    /// Generate the headers to prefix the module file with.
    /// Typically import statements
    fn gen_head(&self) -> TokenStream;

    /// Generate the main body content of the module file
    fn gen_body(&self) -> TokenStream;

    /// Check if this module generates multiple files
    fn has_multiple_files(&self) -> bool {
        false
    }

    /// Generate multiple files with (filename, content) pairs
    /// Only called if has_multiple_files() returns true
    fn gen_files(&self) -> Vec<(String, TokenStream)> {
        vec![]
    }

    /// Generate the mod.rs file content for multi-file modules
    fn gen_mod_file(&self) -> TokenStream {
        TokenStream::new()
    }
}

pub trait IdlFormat {
    fn program_name(&self) -> &str;

    fn program_version(&self) -> &str;

    fn program_address(&self) -> Option<&str>;

    fn is_correct_idl_format(&self) -> bool;

    fn dependencies(&self, args: &crate::Args) -> Map<String, Value>;

    fn modules<'me>(&'me self, args: &'me crate::Args) -> Vec<Box<dyn IdlCodegenModule + 'me>>;

    /// Check if this IDL represents an Anchor contract
    /// Anchor contracts have discriminator fields in instructions/accounts
    fn is_anchor_contract(&self) -> bool;
}
