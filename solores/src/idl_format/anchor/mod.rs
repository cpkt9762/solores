use serde::Deserialize;
use toml::{map::Map, Value};

use crate::write_cargotoml::{
    DependencyValue, FeaturesDependencyValue, OptionalDependencyValue, BORSH_CRATE, BYTEMUCK_CRATE,
    NUM_DERIVE_CRATE, NUM_TRAITS_CRATE, SERDE_CRATE, SERDE_WITH_CRATE, SOLANA_PROGRAM_CRATE, THISERROR_CRATE,
};

use super::{IdlCodegenModule, IdlFormat};

use self::{
    accounts::{AccountsCodegenModule, NamedAccount},
    constants::ConstantsCodegenModule,
    errors::{ErrorEnumVariant, ErrorsCodegenModule},
    events::{Event, EventsCodegenModule},
    instructions::{IxCodegenModule, NamedInstruction},
    parsers::{AccountsParserModule, InstructionsParserModule},
    typedefs::{NamedType},
    types::{TypesCodegenModule},
};

pub mod accounts;
pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod parsers;
pub mod typedefs;
pub mod types;

#[derive(Deserialize, Debug)]
pub struct AnchorIdl {
    pub address: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub metadata: Option<Metadata>,
    pub accounts: Option<Vec<NamedAccount>>,
    pub types: Option<Vec<NamedType>>,
    pub instructions: Option<Vec<NamedInstruction>>,
    pub errors: Option<Vec<ErrorEnumVariant>>,
    pub events: Option<Vec<Event>>,
    
    // Additional fields
    pub constants: Option<Vec<Constant>>,  // Constants definition (e.g., in dlmm.json)
}

#[derive(Deserialize, Debug)]
pub struct Constant {
    pub name: String,
    #[serde(rename = "type")]
    pub const_type: serde_json::Value,  // Can be string like "i32" or object like {"defined": "usize"}
    pub value: String,
}

#[derive(Deserialize, Debug)]
pub struct Metadata {
    pub address: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub spec: Option<String>,
    pub description: Option<String>,
}

impl AnchorIdl {
    pub fn accounts(&self) -> &[NamedAccount] {
        self.accounts.as_deref().unwrap_or(&[])
    }

    pub fn instructions(&self) -> &[NamedInstruction] {
        self.instructions.as_deref().unwrap_or(&[])
    }
}

impl IdlFormat for AnchorIdl {
    fn program_name(&self) -> &str {
        if let Some(name) = &self.name {
            name.as_str()
        } else if let Some(name) = self.metadata.as_ref().map(|m| m.name.as_ref()) {
            name.unwrap()
        } else {
            "anchor"
        }
    }

    fn program_version(&self) -> &str {
        if let Some(version) = &self.version {
            version.as_str()
        } else if let Some(version) = self.metadata.as_ref().map(|m| m.version.as_ref()) {
            version.unwrap()
        } else {
            "0.0.0"
        }
    }

    fn program_address(&self) -> Option<&str> {
        if let Some(address) = &self.address {
            Some(address.as_str())
        } else if let Some(address) = self.metadata.as_ref().map(|m| m.address.as_ref()) {
            address.map(|s| s.as_str())
        } else {
            None
        }
    }

    /// Anchor IDLs dont seem to have an identifier,
    /// assume unindentified IDLs are anchor by default.
    /// -> Make sure to try deserializing Anchor last
    fn is_correct_idl_format(&self) -> bool {
        true
    }

    fn modules<'me>(&'me self, args: &'me crate::Args) -> Vec<Box<dyn IdlCodegenModule + 'me>> {
        let mut res: Vec<Box<dyn IdlCodegenModule + 'me>> = Vec::new();
        
        // Determine what modules to generate based on mode
        let generate_interface = !args.parser_only;
        let generate_parsers = args.generate_parser || args.parser_only;
        let needs_dependencies_for_parsers = args.parser_only && generate_parsers;
        
        // Generate regular interface modules unless parser_only is set
        if generate_interface {
            // Add constants module if present
            if let Some(constants) = &self.constants {
                if !constants.is_empty() {
                    res.push(Box::new(ConstantsCodegenModule { constants }));
                }
            }
            
            if let Some(v) = &self.accounts {
                res.push(Box::new(AccountsCodegenModule {
                    cli_args: args,
                    named_accounts: v,
                    typedefs: self.r#types.as_deref().unwrap_or(&[]),
                    is_anchor_contract: self.is_anchor_contract(),
                }));
            }
            if let Some(v) = &self.r#types {
                res.push(Box::new(TypesCodegenModule {
                    cli_args: args,
                    named_types: v,
                }));
            }
            if let Some(v) = &self.instructions {
                res.push(Box::new(IxCodegenModule {
                    program_name: self.program_name(),
                    instructions: v,
                    is_anchor_contract: self.is_anchor_contract(),
                }));
            }
            if let Some(v) = &self.errors {
                res.push(Box::new(ErrorsCodegenModule {
                    program_name: self.program_name(),
                    variants: v,
                }));
            }
            if let Some(v) = &self.events {
                res.push(Box::new(EventsCodegenModule {
                    events: v,
                    named_types: self.r#types.as_deref().unwrap_or(&[]),
                }));
            }
        }
        
        // In parser-only mode, generate minimal dependencies that parsers need
        if needs_dependencies_for_parsers {
            log::debug!("🔧 --parser-only模式：生成Parser依赖的最小模块集");
            
            // Generate accounts module if parsers need it
            if !self.accounts().is_empty() {
                log::debug!("  ✓ 生成accounts模块 (AccountsParser dependency)");
                res.push(Box::new(AccountsCodegenModule {
                    cli_args: args,
                    named_accounts: self.accounts.as_ref().unwrap(),
                    typedefs: self.r#types.as_deref().unwrap_or(&[]),
                    is_anchor_contract: self.is_anchor_contract(),
                }));
            }
            
            // Generate instructions module if parsers need it  
            if !self.instructions().is_empty() {
                log::debug!("  ✓ 生成instructions模块 (InstructionsParser dependency)");
                res.push(Box::new(IxCodegenModule {
                    program_name: self.program_name(),
                    instructions: self.instructions.as_ref().unwrap(),
                    is_anchor_contract: self.is_anchor_contract(),
                }));
            }
            
            // Generate types module if needed
            if let Some(v) = &self.r#types {
                log::debug!("  ✓ 生成types模块 (Parser可能需要的类型定义)");
                res.push(Box::new(TypesCodegenModule {
                    cli_args: args,
                    named_types: v,
                }));
            }
        }

        // Generate parser modules if requested
        if generate_parsers {
            if !self.accounts().is_empty() {
                res.push(Box::new(AccountsParserModule::new(self, args)));
            }
            if !self.instructions().is_empty() {
                res.push(Box::new(InstructionsParserModule::new(self, args)));
            }
        }

        res
    }

    fn dependencies(&self, args: &crate::Args) -> Map<String, Value> {
        let mut map = Map::new();
        map.insert(BORSH_CRATE.into(), DependencyValue(&args.borsh_vers).into());
        if !args.zero_copy.is_empty() {
            map.insert(
                BYTEMUCK_CRATE.into(),
                FeaturesDependencyValue {
                    dependency: DependencyValue(&args.bytemuck_vers),
                    features: vec!["derive".into()],
                }
                .into(),
            );
        }
        map.insert(
            SOLANA_PROGRAM_CRATE.into(),
            DependencyValue(&args.solana_program_vers).into(),
        );
        map.insert(
            SERDE_CRATE.into(),
            OptionalDependencyValue(DependencyValue(&args.serde_vers)).into(),
        );
        map.insert(
            SERDE_WITH_CRATE.into(),
            OptionalDependencyValue(DependencyValue(&args.serde_with_vers)).into(),
        );
        if self.errors.is_some() {
            map.insert(
                THISERROR_CRATE.into(),
                DependencyValue(&args.thiserror_vers).into(),
            );
            map.insert(
                NUM_DERIVE_CRATE.into(),
                DependencyValue(&args.num_derive_vers).into(),
            );
            map.insert(
                NUM_TRAITS_CRATE.into(),
                DependencyValue(&args.num_traits_vers).into(),
            );
        }
        map
    }

    fn is_anchor_contract(&self) -> bool {
        // Check if instructions have 8-byte discriminators (Anchor) or 1-byte discriminators (non-Anchor)
        if let Some(instructions) = &self.instructions {
            log::debug!("🔍 IDL contract type detection:");
            log::debug!("  Instructions count: {}", instructions.len());
            
            let mut anchor_indicators = 0;
            let mut non_anchor_indicators = 0;
            
            for (idx, ix) in instructions.iter().enumerate() {
                if let Some(ref disc) = ix.discriminator {
                    log::debug!("  Instruction[{}] '{}': discriminator={:?}", idx, ix.name, disc);
                    
                    if disc.len() == 8 {
                        anchor_indicators += 1;
                        log::debug!("    → 8-byte discriminator (Anchor indicator)");
                    } else if disc.len() == 1 {
                        non_anchor_indicators += 1;
                        log::debug!("    → 1-byte discriminator (non-Anchor indicator)");
                    } else {
                        log::debug!("    → Unusual discriminator length: {}", disc.len());
                    }
                } else {
                    log::debug!("  Instruction[{}] '{}': no discriminator", idx, ix.name);
                }
            }
            
            log::debug!("  Analysis: anchor_indicators={}, non_anchor_indicators={}", anchor_indicators, non_anchor_indicators);
            
            // If we have 8-byte discriminators, it's Anchor
            // If we have 1-byte discriminators, it's non-Anchor  
            // If mixed or no discriminators, default based on majority
            let is_anchor = anchor_indicators > non_anchor_indicators;
            log::debug!("  Final result: is_anchor_contract={}", is_anchor);
            return is_anchor;
        }
        log::debug!("🔍 No instructions found, defaulting to non-Anchor");
        false
    }
}
