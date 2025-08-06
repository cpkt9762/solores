# sol_pumpfun_interface

Generated Rust interface for the **pumpfun** Solana program.

- **Program Name**: pumpfun
- **Program Version**: 0.1.0
- **Program ID**: `6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P`

## Directory Structure

```
sol_pumpfun_interface/ 
├── Cargo.toml
├── idl.json              # Original IDL file
├── README.md             # This file
└── src/
    ├── accounts/     # Program account structures
    │   ├── bondingcurve.rs
    │   ├── global.rs
    │   └── mod.rs            # Module declarations
    ├── errors.rs            # Program error definitions
    ├── events/     # Event structures and discriminators
    │   ├── complete_event.rs
    │   ├── create_event.rs
    │   ├── mod.rs            # Module declarations
    │   ├── set_params_event.rs
    │   └── trade_event.rs
    ├── instructions/     # Instruction definitions and account structures
    │   ├── buy.rs
    │   ├── create.rs
    │   ├── initialize.rs
    │   ├── mod.rs            # Module declarations
    │   ├── sell.rs
    │   ├── set_params.rs
    │   └── withdraw.rs
    ├── lib.rs            # Program ID declaration and module exports
    └── parsers/     # Account and instruction parsing functions
        ├── accounts.rs
        ├── instructions.rs
        └── mod.rs            # Module declarations

```

## Usage

### Basic Import

```rust
use sol_pumpfun_interface::*;
```

### Program ID

The program ID is declared as a constant:

```rust
use sol_pumpfun_interface::ID;
// or
use solana_program::declare_id;
// ID is accessible as sol_pumpfun_interface::ID
```


### Instructions

```rust
use sol_pumpfun_interface::*;

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
use sol_pumpfun_interface::*;

// Deserialize account data
let account_data = SomeAccount::try_from_slice(&account_info.data.borrow())?;
```

### Parsing

```rust
use sol_pumpfun_interface::*;

// Parse account data
let parsed_account = try_unpack_account(&account_data)?;

// Parse instruction data  
let parsed_instruction = parse_instruction(&instruction_data)?;
```

## Modules

- **instructions/**: Contains instruction definitions, account structures, and helper functions for invoking program instructions.
- **accounts/**: Program account structures with Borsh serialization support.
- **events/**: Event structures emitted by the program with discriminators for parsing.
- **errors.rs**: Program-specific error definitions with conversion to `ProgramError`.
- **parsers/**: Utility functions for parsing account data and instruction data based on discriminators.

## Generated with

This crate was generated using [solores](https://github.com/cpkt9762/solores) - a Solana IDL to Rust client code generator.

```bash
solores idls/pump-fun-idl.json --output-dir ./test_batch_headers/sol_pumpfun_interface --output-crate-name sol_pumpfun_interface
```
