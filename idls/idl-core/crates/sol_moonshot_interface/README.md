# sol_moonshot_interface

Generated Rust interface for the **moonshot** Solana program.

- **Program Name**: moonshot
- **Program Version**: 1.0.0
- **Program ID**: `MoonCVVNZFSYkqNXP6bxHLPL6QQJiMagDL3qcqUQTrG`

## Directory Structure

```
sol_moonshot_interface/ 
├── Cargo.toml
├── idl.json              # Original IDL file
├── README.md             # This file
└── src/
    ├── accounts/     # Program account structures
    │   ├── config_account.rs
    │   ├── curve_account.rs
    │   └── mod.rs            # Module declarations
    ├── errors.rs            # Program error definitions
    ├── events/     # Event structures and discriminators
    │   ├── migration_event.rs
    │   ├── mod.rs            # Module declarations
    │   └── trade_event.rs
    ├── instructions/     # Instruction definitions and account structures
    │   ├── buy.rs
    │   ├── config_init.rs
    │   ├── config_update.rs
    │   ├── migrate_funds.rs
    │   ├── mod.rs            # Module declarations
    │   ├── sell.rs
    │   └── token_mint.rs
    ├── lib.rs            # Program ID declaration and module exports
    ├── parsers/     # Account and instruction parsing functions
    │   ├── accounts.rs
    │   ├── instructions.rs
    │   └── mod.rs            # Module declarations
    └── types/     # Custom type definitions
        ├── config_params.rs
        ├── currency.rs
        ├── curve_type.rs
        ├── fixed_side.rs
        ├── migration_target.rs
        ├── mod.rs            # Module declarations
        ├── token_mint_params.rs
        ├── trade_params.rs
        └── trade_type.rs

```

## Usage

### Basic Import

```rust
use sol_moonshot_interface::*;
```

### Program ID

The program ID is declared as a constant:

```rust
use sol_moonshot_interface::ID;
// or
use solana_program::declare_id;
// ID is accessible as sol_moonshot_interface::ID
```


### Instructions

```rust
use sol_moonshot_interface::*;

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
use sol_moonshot_interface::*;

// Deserialize account data
let account_data = SomeAccount::try_from_slice(&account_info.data.borrow())?;
```

## Modules

- **instructions/**: Contains instruction definitions, account structures, and helper functions for invoking program instructions.
- **accounts/**: Program account structures with Borsh serialization support.
- **types/**: Custom type definitions used by the program, including enums and structs.
- **events/**: Event structures emitted by the program with discriminators for parsing.
- **errors.rs**: Program-specific error definitions with conversion to `ProgramError`.

## Generated with

This crate was generated using [solores](https://github.com/cpkt9762/solores) - a Solana IDL to Rust client code generator.

```bash
solores idls/idl-core/pending_dex_idls/moonshot.json --output-dir idls/idl-core/crates/sol_moonshot_interface --output-crate-name sol_moonshot_interface
```
