# sol_raydium_interface

Generated Rust interface for the **raydium** Solana program.

- **Program Name**: raydium
- **Program Version**: 0.3.0
- **Program ID**: `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8`

## Directory Structure

```
sol_raydium_interface/ 
├── Cargo.toml
├── idl.json              # Original IDL file
├── README.md             # This file
└── src/
    ├── accounts/     # Program account structures
    │   ├── ammconfig.rs
    │   ├── amminfo.rs
    │   ├── mod.rs            # Module declarations
    │   └── targetorders.rs
    ├── errors.rs            # Program error definitions
    ├── instructions/     # Instruction definitions and account structures
    │   ├── admin_cancel_orders.rs
    │   ├── create_config_account.rs
    │   ├── deposit.rs
    │   ├── initialize.rs
    │   ├── initialize2.rs
    │   ├── migrate_to_open_book.rs
    │   ├── mod.rs            # Module declarations
    │   ├── monitor_step.rs
    │   ├── pre_initialize.rs
    │   ├── set_params.rs
    │   ├── simulate_info.rs
    │   ├── swap_base_in.rs
    │   ├── swap_base_out.rs
    │   ├── update_config_account.rs
    │   ├── withdraw.rs
    │   ├── withdraw_pnl.rs
    │   └── withdraw_srm.rs
    ├── lib.rs            # Program ID declaration and module exports
    ├── parsers/     # Account and instruction parsing functions
    │   ├── accounts.rs
    │   ├── instructions.rs
    │   └── mod.rs            # Module declarations
    └── types/     # Custom type definitions
        ├── fees.rs
        ├── last_order_distance.rs
        ├── mod.rs            # Module declarations
        ├── need_take.rs
        ├── out_put_data.rs
        ├── swap_instruction_base_in.rs
        ├── swap_instruction_base_out.rs
        ├── target_order.rs
        ├── withdraw_dest_token.rs
        └── withdraw_queue.rs

```

## Usage

### Basic Import

```rust
use sol_raydium_interface::*;
```

### Program ID

The program ID is declared as a constant:

```rust
use sol_raydium_interface::ID;
// or
use solana_program::declare_id;
// ID is accessible as sol_raydium_interface::ID
```


### Instructions

```rust
use sol_raydium_interface::*;

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
use sol_raydium_interface::*;

// Deserialize account data
let account_data = SomeAccount::try_from_slice(&account_info.data.borrow())?;
```

### Parsing

```rust
use sol_raydium_interface::*;

// Parse account data
let parsed_account = try_unpack_account(&account_data)?;

// Parse instruction data  
let parsed_instruction = parse_instruction(&instruction_data)?;
```

## Modules

- **instructions/**: Contains instruction definitions, account structures, and helper functions for invoking program instructions.
- **accounts/**: Program account structures with Borsh serialization support.
- **types/**: Custom type definitions used by the program, including enums and structs.
- **errors.rs**: Program-specific error definitions with conversion to `ProgramError`.
- **parsers/**: Utility functions for parsing account data and instruction data based on discriminators.

## Generated with

This crate was generated using [solores](https://github.com/cpkt9762/solores) - a Solana IDL to Rust client code generator.

```bash
solores idls/raydium.json --output-dir ./test_batch_headers/sol_raydium_interface --output-crate-name sol_raydium_interface
```
