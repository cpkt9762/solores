# sol_saros_interface

Generated Rust interface for the **saros** Solana program.

- **Program Name**: saros
- **Program Version**: 0.1.0
- **Program ID**: `SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ`

## Directory Structure

```
sol_saros_interface/ 
├── Cargo.toml
├── idl.json              # Original IDL file
├── README.md             # This file
└── src/
    ├── accounts/     # Program account structures
    │   ├── mod.rs            # Module declarations
    │   └── transmissions.rs
    ├── errors.rs            # Program error definitions
    ├── instructions/     # Instruction definitions and account structures
    │   ├── create_feed.rs
    │   ├── mod.rs            # Module declarations
    │   ├── query.rs
    │   └── submit_feed.rs
    ├── lib.rs            # Program ID declaration and module exports
    ├── parsers/     # Account and instruction parsing functions
    │   ├── accounts.rs
    │   ├── instructions.rs
    │   └── mod.rs            # Module declarations
    └── types/     # Custom type definitions
        ├── mod.rs            # Module declarations
        ├── round.rs
        └── scope.rs

```

## Usage

### Basic Import

```rust
use sol_saros_interface::*;
```

### Program ID

The program ID is declared as a constant:

```rust
use sol_saros_interface::ID;
// or
use solana_program::declare_id;
// ID is accessible as sol_saros_interface::ID
```


### Instructions

```rust
use sol_saros_interface::*;

// Create instruction
let instruction = some_instruction_ix(
    SomeInstructionKeys { /* account keys */ },
    SomeInstructionIxArgs { /* instruction args */ }
)?;

// Invoke instruction  
some_instruction_invoke(accounts, args)?;
```

### Accounts

```rust
use sol_saros_interface::*;

// Deserialize account data
let account_data = SomeAccount::try_from_slice(&account_info.data.borrow())?;
```

## Modules

- **instructions/**: Contains instruction definitions, account structures, and helper functions for invoking program instructions.
- **accounts/**: Program account structures with Borsh serialization support.
- **types/**: Custom type definitions used by the program, including enums and structs.
- **errors.rs**: Program-specific error definitions with conversion to `ProgramError`.

## Generated with

This crate was generated using [solores](https://github.com/cpkt9762/solores) - a Solana IDL to Rust client code generator.

```bash
solores idls/idl-core/pending_dex_idls/saros.json --output-dir idls/idl-core/crates/sol_saros_interface --output-crate-name sol_saros_interface
```
