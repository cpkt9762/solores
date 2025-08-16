# sol_lifinity_interface

Generated Rust interface for the **lifinity** Solana program.

- **Program Name**: lifinity
- **Program Version**: 0.1.1
- **Program ID**: `EewxydAPCCVuNEyrVN68PuSYdQ7wKn27V9Gjeoi8dy3S`

## Directory Structure

```
sol_lifinity_interface/ 
├── Cargo.toml
├── idl.json              # Original IDL file
├── README.md             # This file
└── src/
    ├── accounts/     # Program account structures
    │   ├── amm.rs
    │   └── mod.rs            # Module declarations
    ├── errors.rs            # Program error definitions
    ├── instructions/     # Instruction definitions and account structures
    │   ├── deposit_all_token_types.rs
    │   ├── mod.rs            # Module declarations
    │   ├── swap.rs
    │   └── withdraw_all_token_types.rs
    ├── lib.rs            # Program ID declaration and module exports
    ├── parsers/     # Account and instruction parsing functions
    │   ├── accounts.rs
    │   ├── instructions.rs
    │   └── mod.rs            # Module declarations
    └── types/     # Custom type definitions
        ├── amm_config.rs
        ├── amm_curve.rs
        ├── amm_fees.rs
        ├── curve_type.rs
        └── mod.rs            # Module declarations

```

## Usage

### Basic Import

```rust
use sol_lifinity_interface::*;
```

### Program ID

The program ID is declared as a constant:

```rust
use sol_lifinity_interface::ID;
// or
use solana_program::declare_id;
// ID is accessible as sol_lifinity_interface::ID
```


### Instructions

```rust
use sol_lifinity_interface::*;

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
use sol_lifinity_interface::*;

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
solores idls/lifinity.json --output-dir idls/idl-core/crates/sol_lifinity_interface/sol_lifinity_interface --output-crate-name sol_lifinity_interface
```
