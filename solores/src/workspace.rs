//! Workspace generation functionality for Solores
//! 
//! This module handles the generation of Cargo workspace structures
//! when multiple crates need to be organized together.

use std::{
    fs,
    path::PathBuf,
};

use crate::error::SoloresError;

/// Configuration for workspace generation
#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
    /// Name of the workspace
    pub name: String,
    /// Base output directory where the workspace will be created
    pub output_dir: PathBuf,
    /// List of member crate names
    pub members: Vec<String>,
    /// Dependency versions to be shared across the workspace
    pub dependency_versions: DependencyVersions,
}

/// Shared dependency versions for the workspace
#[derive(Debug, Clone)]
pub struct DependencyVersions {
    pub solana_program_vers: String,
    pub borsh_vers: String,
    pub thiserror_vers: String,
    pub num_derive_vers: String,
    pub num_traits_vers: String,
    pub serde_vers: String,
    pub serde_with_vers: String,
    pub bytemuck_vers: String,
}

impl DependencyVersions {
    /// Create dependency versions from Args
    pub fn from_args(args: &crate::Args) -> Self {
        Self {
            solana_program_vers: args.solana_program_vers.clone(),
            borsh_vers: args.borsh_vers.clone(),
            thiserror_vers: args.thiserror_vers.clone(),
            num_derive_vers: args.num_derive_vers.clone(),
            num_traits_vers: args.num_traits_vers.clone(),
            serde_vers: args.serde_vers.clone(),
            serde_with_vers: args.serde_with_vers.clone(),
            bytemuck_vers: args.bytemuck_vers.clone(),
        }
    }
}

/// Generate a workspace Cargo.toml file
pub fn generate_workspace_cargo_toml(config: &WorkspaceConfig) -> Result<String, SoloresError> {
    let members_list = config.members
        .iter()
        .map(|member| format!("    \"{}\"", member))
        .collect::<Vec<_>>()
        .join(",\n");

    let workspace_cargo_toml = format!(
        r#"[workspace]
resolver = "2"
members = [
{}
]

[workspace.package]
version = "0.0.0"
edition = "2021"
license = "MIT OR Apache-2.0"
authors = ["Solores Generator <noreply@solores.dev>"]
description = "Generated Solana program interfaces"
repository = "https://github.com/cpkt9762/solores"
keywords = ["solana", "blockchain", "web3"]
categories = ["cryptography::cryptocurrencies"]

[workspace.dependencies]
# Solana dependencies
solana-program = "{}"
solana-pubkey = "2.0"

# Serialization
borsh = "{}"
serde = {{ version = "{}", features = ["derive"] }}
serde_with = {{ version = "{}" }}

# Error handling
thiserror = "{}"

# Numeric types
num-derive = "{}"
num-traits = "{}"

# Zero-copy serialization
bytemuck = {{ version = "{}", features = ["derive"] }}

[profile.release]
overflow-checks = true
lto = "fat"
codegen-units = 1

[profile.release.build-override]
opt-level = 3
incremental = false
codegen-units = 1
"#,
        members_list,
        config.dependency_versions.solana_program_vers,
        config.dependency_versions.borsh_vers,
        config.dependency_versions.serde_vers,
        config.dependency_versions.serde_with_vers,
        config.dependency_versions.thiserror_vers,
        config.dependency_versions.num_derive_vers,
        config.dependency_versions.num_traits_vers,
        config.dependency_versions.bytemuck_vers,
    );

    Ok(workspace_cargo_toml)
}

/// Write workspace Cargo.toml to disk
pub fn write_workspace_cargo_toml(config: &WorkspaceConfig) -> Result<(), SoloresError> {
    let cargo_toml_content = generate_workspace_cargo_toml(config)?;
    let cargo_toml_path = config.output_dir.join("Cargo.toml");

    // Ensure output directory exists
    if let Some(parent_dir) = cargo_toml_path.parent() {
        fs::create_dir_all(parent_dir)
            .map_err(|e| SoloresError::file_operation_error("create directory", parent_dir.display().to_string(), e))?;
    }

    fs::write(&cargo_toml_path, cargo_toml_content)
        .map_err(|e| SoloresError::file_operation_error("write workspace Cargo.toml", cargo_toml_path.display().to_string(), e))?;

    log::info!("✅ Generated workspace Cargo.toml at: {}", cargo_toml_path.display());
    Ok(())
}

/// Generate member crate Cargo.toml content that uses workspace dependencies
pub fn generate_member_cargo_toml(
    crate_name: &str,
    program_id: &str,
    generate_parser: bool,
    test: bool,
    zero_copy: &[String],
) -> String {
    // Determine required features
    let mut features = Vec::new();
    let mut optional_deps = Vec::new();

    if zero_copy.len() > 0 {
        features.push("bytemuck");
        optional_deps.push("bytemuck = { workspace = true, optional = true }");
    }

    // Base dependencies
    let mut dependencies = vec![
        "borsh = { workspace = true }".to_string(),
        "solana-program = { workspace = true }".to_string(),
        "solana-pubkey = { workspace = true }".to_string(),
        "thiserror = { workspace = true }".to_string(),
        "num-derive = { workspace = true }".to_string(),
        "num-traits = { workspace = true }".to_string(),
    ];

    // Add optional dependencies
    dependencies.extend(optional_deps.into_iter().map(|s| s.to_string()));

    let dependencies_section = dependencies.join("\n");

    // Features section
    let features_section = if features.is_empty() {
        String::new()
    } else {
        format!("\n[features]\ndefault = []\nserde = [\"dep:serde\", \"dep:serde_with\"]\n{} = [\"dep:bytemuck\"]\n", 
                features.join("\", \""))
    };

    format!(
        r#"[package]
name = "{}"
version.workspace = true
edition.workspace = true
license.workspace = true
authors.workspace = true
description.workspace = true
repository.workspace = true
keywords.workspace = true
categories.workspace = true

[dependencies]
{}{}


[package.metadata]
solana-program-id = "{}"
"#,
        crate_name,
        dependencies_section,
        features_section,
        program_id,
    )
}

/// Check if workspace mode should be enabled and validate configuration
pub fn validate_workspace_config(args: &crate::Args) -> Result<Option<WorkspaceConfig>, SoloresError> {
    if !args.workspace {
        return Ok(None);
    }

    // Workspace mode only makes sense with batch processing
    if !args.batch {
        log::warn!("⚠️  Workspace mode is intended for batch processing. Consider using --batch flag.");
        log::warn!("   Single IDL processing will create a workspace with only one member.");
    }

    let config = WorkspaceConfig {
        name: args.workspace_name.clone(),
        output_dir: args.batch_output_dir.clone(),
        members: Vec::new(), // Will be populated during batch processing
        dependency_versions: DependencyVersions::from_args(args),
    };

    Ok(Some(config))
}

/// Update workspace configuration with a new member
pub fn add_workspace_member(config: &mut WorkspaceConfig, member_name: String) {
    if !config.members.contains(&member_name) {
        config.members.push(member_name);
        log::debug!("📦 Added workspace member: {}", config.members.last().unwrap());
    }
}

/// Finalize workspace by writing the workspace Cargo.toml
pub fn finalize_workspace(config: &WorkspaceConfig) -> Result<(), SoloresError> {
    if config.members.is_empty() {
        log::warn!("⚠️  No workspace members found. Skipping workspace Cargo.toml generation.");
        return Ok(());
    }

    log::info!("🏗️  Finalizing workspace '{}' with {} members", config.name, config.members.len());
    for member in &config.members {
        log::info!("   - {}", member);
    }

    write_workspace_cargo_toml(config)?;
    
    log::info!("✅ Workspace '{}' created successfully at: {}", 
               config.name, config.output_dir.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_generate_workspace_cargo_toml() {
        let config = WorkspaceConfig {
            name: "test_workspace".to_string(),
            output_dir: PathBuf::from("/tmp/test"),
            members: vec!["crate_a".to_string(), "crate_b".to_string()],
            dependency_versions: DependencyVersions {
                solana_program_vers: "^2.0".to_string(),
                borsh_vers: "^1.5".to_string(),
                thiserror_vers: "^1.0".to_string(),
                num_derive_vers: "0.4.2".to_string(),
                num_traits_vers: "^0.2".to_string(),
                serde_vers: "^1.0".to_string(),
                serde_with_vers: "^3.0".to_string(),
                bytemuck_vers: "^1.16".to_string(),
            },
        };

        let result = generate_workspace_cargo_toml(&config).unwrap();
        
        assert!(result.contains("[workspace]"));
        assert!(result.contains("crate_a"));
        assert!(result.contains("crate_b"));
        assert!(result.contains("solana-program = \"^2.0\""));
        assert!(result.contains("borsh = \"^1.5\""));
    }

    #[test]
    fn test_generate_member_cargo_toml() {
        let result = generate_member_cargo_toml(
            "test_crate",
            "11111111111111111111111111111112",
            true,
            false,
            &[]
        );

        assert!(result.contains("name = \"test_crate\""));
        assert!(result.contains("workspace = true"));
        assert!(result.contains("11111111111111111111111111111112"));
        assert!(result.contains("borsh = { workspace = true }"));
    }
}