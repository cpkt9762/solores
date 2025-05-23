use serde::Deserialize;
use toml::{map::Map, Value};

use crate::write_cargotoml::{
    DependencyValue, FeaturesDependencyValue, OptionalDependencyValue, BORSH_CRATE, BYTEMUCK_CRATE,
    NUM_DERIVE_CRATE, NUM_TRAITS_CRATE, SERDE_CRATE, SOLANA_PROGRAM_CRATE, THISERROR_CRATE,
};

use super::{IdlCodegenModule, IdlFormat};

use self::{
    accounts::{AccountsCodegenModule, NamedAccount},
    errors::{ErrorEnumVariant, ErrorsCodegenModule},
    events::{Event, EventsCodegenModule},
    instructions::{IxCodegenModule, NamedInstruction},
    typedefs::{NamedType, TypedefsCodegenModule},
};

pub mod accounts;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod typedefs;

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
}

#[derive(Deserialize, Debug)]
pub struct Metadata {
    pub address: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub spec: Option<String>,
    pub description: Option<String>,
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
        if let Some(v) = &self.accounts {
            res.push(Box::new(AccountsCodegenModule {
                cli_args: args,
                named_accounts: v,
            }));
        }
        if let Some(v) = &self.r#types {
            res.push(Box::new(TypedefsCodegenModule {
                cli_args: args,
                named_types: v,
            }));
        }
        if let Some(v) = &self.instructions {
            res.push(Box::new(IxCodegenModule {
                program_name: self.program_name(),
                instructions: v,
            }));
        }
        if let Some(v) = &self.errors {
            res.push(Box::new(ErrorsCodegenModule {
                program_name: self.program_name(),
                variants: v,
            }));
        }
        if let Some(v) = &self.events {
            res.push(Box::new(EventsCodegenModule(v)));
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
}
