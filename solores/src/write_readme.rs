use std::io::Write;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::BTreeMap;

use crate::{idl_format::IdlFormat, utils::open_file_create_overwrite, Args};

#[derive(Debug, Clone)]
struct FileNode {
    name: String,
    is_directory: bool,
    children: BTreeMap<String, FileNode>,
    comment: Option<String>,
}

impl FileNode {
    fn new(name: String, is_directory: bool) -> Self {
        Self {
            name,
            is_directory,
            children: BTreeMap::new(),
            comment: None,
        }
    }
    
    fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }
}

/// Scan all generated files in the output directory
fn scan_generated_files(output_dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let src_dir = output_dir.join("src");
    let mut files = Vec::new();
    
    if src_dir.exists() {
        scan_directory_recursive(&src_dir, &src_dir, &mut files)?;
    }
    
    // Sort files for consistent output
    files.sort();
    Ok(files)
}

/// Recursively scan directory and collect .rs files
fn scan_directory_recursive(dir: &Path, base_dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            scan_directory_recursive(&path, base_dir, files)?;
        } else if path.extension().map_or(false, |ext| ext == "rs") {
            // Store relative path from src directory
            if let Ok(relative_path) = path.strip_prefix(base_dir) {
                files.push(relative_path.to_path_buf());
            }
        }
    }
    Ok(())
}

/// Build file tree structure from file list
fn build_file_tree(files: &[PathBuf]) -> FileNode {
    let mut root = FileNode::new("src".to_string(), true);
    
    for file in files {
        let mut current = &mut root;
        let components: Vec<&str> = file.components()
            .map(|c| c.as_os_str().to_str().unwrap())
            .collect();
            
        // Navigate through path components
        for (i, component) in components.iter().enumerate() {
            let is_last = i == components.len() - 1;
            let is_directory = !is_last;
            
            if !current.children.contains_key(*component) {
                let mut node = FileNode::new(component.to_string(), is_directory);
                
                // Add comments for special files  
                if !is_directory {
                    node = match component {
                        &"lib.rs" => node.with_comment("Program ID declaration and module exports".to_string()),
                        &"mod.rs" => node.with_comment("Module declarations".to_string()),
                        &"errors.rs" => node.with_comment("Program error definitions".to_string()),
                        &"constants.rs" => node.with_comment("Program constants".to_string()),
                        _ => node,
                    };
                }
                
                current.children.insert(component.to_string(), node);
            }
            
            if !is_last {
                current = current.children.get_mut(*component).unwrap();
            }
        }
    }
    
    // Add comments for directories
    add_directory_comments(&mut root);
    root
}

/// Add comments for known directories
fn add_directory_comments(node: &mut FileNode) {
    if node.is_directory {
        node.comment = match node.name.as_str() {
            "instructions" => Some("Instruction definitions and account structures".to_string()),
            "accounts" => Some("Program account structures".to_string()),
            "types" => Some("Custom type definitions".to_string()),
            "events" => Some("Event structures and discriminators".to_string()),
            "parsers" => Some("Account and instruction parsing functions".to_string()),
            _ => node.comment.clone(),
        };
    }
    
    // Recursively add comments to children
    for child in node.children.values_mut() {
        add_directory_comments(child);
    }
}

/// Format file tree as a tree structure string
fn format_file_tree(root: &FileNode, crate_name: &str) -> String {
    let mut result = format!("{}/ \n", crate_name);
    result.push_str("├── Cargo.toml\n");
    result.push_str("├── idl.json              # Original IDL file\n");
    result.push_str("├── README.md             # This file\n");
    result.push_str("└── ");
    
    format_node_recursive(root, "", true, &mut result);
    result
}

/// Recursively format a node with proper tree characters
fn format_node_recursive(node: &FileNode, prefix: &str, is_last: bool, result: &mut String) {
    // Add the node name
    result.push_str(&node.name);
    if node.is_directory {
        result.push('/');
    }
    
    // Add comment if present
    if let Some(comment) = &node.comment {
        if node.is_directory {
            result.push_str(&format!("     # {}", comment));
        } else {
            result.push_str(&format!("            # {}", comment));
        }
    }
    result.push('\n');
    
    // Format children
    if node.is_directory && !node.children.is_empty() {
        let child_count = node.children.len();
        for (i, child) in node.children.values().enumerate() {
            let is_last_child = i == child_count - 1;
            let child_prefix = if is_last { "    " } else { "│   " };
            let tree_char = if is_last_child { "└── " } else { "├── " };
            
            result.push_str(prefix);
            result.push_str(child_prefix);
            result.push_str(tree_char);
            
            let new_prefix = format!("{}{}", prefix, child_prefix);
            format_node_recursive(child, &new_prefix, is_last_child, result);
        }
    }
}

/// Write a README.md file for the generated crate
pub fn write_readme(args: &Args, idl: &dyn IdlFormat) -> std::io::Result<()> {
    let program_name = idl.program_name();
    let program_version = idl.program_version();
    let program_address = idl.program_address();
    let crate_name = &args.output_crate_name;
    
    // Scan generated files to build detailed directory structure
    let generated_files = scan_generated_files(&args.output_dir)?;
    let file_tree = build_file_tree(&generated_files);
    let directory_structure = format_file_tree(&file_tree, crate_name);
    
    // Determine what modules exist (for usage examples)
    let modules = idl.modules(args);
    let has_instructions = modules.iter().any(|m| m.name() == "instructions");
    let has_accounts = modules.iter().any(|m| m.name() == "accounts");  
    let has_types = modules.iter().any(|m| m.name() == "types");
    let has_events = modules.iter().any(|m| m.name() == "events");
    let has_errors = modules.iter().any(|m| m.name() == "errors");
    let has_constants = modules.iter().any(|m| m.name() == "constants");
    let has_parsers = modules.iter().any(|m| m.name() == "accounts_parser" || m.name() == "instructions_parser");
    
    let readme_content = format!(
r#"# {crate_name}

Generated Rust interface for the **{program_name}** Solana program.

- **Program Name**: {program_name}
- **Program Version**: {program_version}
- **Program ID**: `{program_id}`

## Directory Structure

```
{directory_structure}
```

## Usage

### Basic Import

```rust
use {crate_name}::*;
```

### Program ID

The program ID is declared as a constant:

```rust
use {crate_name}::ID;
// or
use solana_program::declare_id;
// ID is accessible as {crate_name}::ID
```
{usage_examples}

## Modules

{module_descriptions}

## Generated with

This crate was generated using [solores](https://github.com/cpkt9762/solores) - a Solana IDL to Rust client code generator.

```bash
solores {idl_path} --output-dir {output_dir} --output-crate-name {crate_name}
```
"#,
        crate_name = crate_name,
        program_name = program_name,
        program_version = program_version,
        program_id = program_address.unwrap_or("11111111111111111111111111111111"),
        directory_structure = directory_structure,
        usage_examples = generate_usage_examples(crate_name, has_instructions, has_accounts, has_parsers),
        module_descriptions = generate_module_descriptions(has_instructions, has_accounts, has_types, has_events, has_constants, has_errors, has_parsers),
        idl_path = args.idl_path.display(),
        output_dir = args.output_dir.display(),
    );
    
    let path = args.output_dir.join("README.md");
    let mut file = open_file_create_overwrite(path)?;
    file.write_all(readme_content.as_bytes())?;
    file.flush()?;
    
    println!("README.md generated successfully");
    Ok(())
}

fn generate_usage_examples(crate_name: &str, has_instructions: bool, has_accounts: bool, has_parsers: bool) -> String {
    let mut examples = String::new();
    
    if has_instructions {
        examples.push_str(&format!(
r#"

### Instructions

```rust
use {crate_name}::*;

// Create instruction
let instruction = some_instruction_ix(
    SomeInstructionKeys {{ /* account keys */ }},
    SomeInstructionIxArgs {{ /* instruction args */ }}
)?;

// Invoke instruction  
some_instruction_invoke(accounts, args)?;
```"#,
            crate_name = crate_name
        ));
    }
    
    if has_accounts {
        examples.push_str(&format!(
r#"

### Accounts

```rust
use {crate_name}::*;

// Deserialize account data
let account_data = SomeAccount::try_from_slice(&account_info.data.borrow())?;
```"#,
            crate_name = crate_name
        ));
    }
    
    if has_parsers {
        examples.push_str(&format!(
r#"

### Parsing

```rust
use {crate_name}::*;

// Parse account data
let parsed_account = try_unpack_account(&account_data)?;

// Parse instruction data  
let parsed_instruction = parse_instruction(&instruction_data)?;
```"#,
            crate_name = crate_name
        ));
    }
    
    examples
}

fn generate_module_descriptions(has_instructions: bool, has_accounts: bool, has_types: bool, has_events: bool, has_constants: bool, has_errors: bool, has_parsers: bool) -> String {
    let mut descriptions = Vec::new();
    
    if has_instructions {
        descriptions.push("- **instructions/**: Contains instruction definitions, account structures, and helper functions for invoking program instructions.");
    }
    
    if has_accounts {
        descriptions.push("- **accounts/**: Program account structures with Borsh serialization support.");
    }
    
    if has_types {
        descriptions.push("- **types/**: Custom type definitions used by the program, including enums and structs.");
    }
    
    if has_events {
        descriptions.push("- **events/**: Event structures emitted by the program with discriminators for parsing.");
    }
    
    if has_constants {
        descriptions.push("- **constants.rs**: Program constants and configuration values.");
    }
    
    if has_errors {
        descriptions.push("- **errors.rs**: Program-specific error definitions with conversion to `ProgramError`.");
    }
    
    if has_parsers {
        descriptions.push("- **parsers/**: Utility functions for parsing account data and instruction data based on discriminators.");
    }
    
    if descriptions.is_empty() {
        "No modules generated.".to_string()
    } else {
        descriptions.join("\n")
    }
}