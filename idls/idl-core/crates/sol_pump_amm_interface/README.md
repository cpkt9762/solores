# sol_pump_amm_interface

Generated Rust interface for the **pump_amm** Solana program.

- **Program Name**: pump_amm
- **Program Version**: 0.1.0
- **Program ID**: `pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA`

## Directory Structure

```
sol_pump_amm_interface/ 
├── Cargo.toml
├── idl.json              # Original IDL file
├── README.md             # This file
└── src/
    ├── accounts/     # Program account structures
    │   ├── bonding_curve.rs
    │   ├── global_config.rs
    │   ├── mod.rs            # Module declarations
    │   └── pool.rs
    ├── errors.rs            # Program error definitions
    ├── events/     # Event structures and discriminators
    │   ├── buy_event.rs
    │   ├── collect_coin_creator_fee_event.rs
    │   ├── create_config_event.rs
    │   ├── create_pool_event.rs
    │   ├── deposit_event.rs
    │   ├── disable_event.rs
    │   ├── extend_account_event.rs
    │   ├── mod.rs            # Module declarations
    │   ├── sell_event.rs
    │   ├── set_bonding_curve_coin_creator_event.rs
    │   ├── set_metaplex_coin_creator_event.rs
    │   ├── update_admin_event.rs
    │   ├── update_fee_config_event.rs
    │   └── withdraw_event.rs
    ├── instructions/     # Instruction definitions and account structures
    │   ├── buy.rs
    │   ├── collect_coin_creator_fee.rs
    │   ├── create_config.rs
    │   ├── create_pool.rs
    │   ├── deposit.rs
    │   ├── disable.rs
    │   ├── extend_account.rs
    │   ├── mod.rs            # Module declarations
    │   ├── sell.rs
    │   ├── set_coin_creator.rs
    │   ├── update_admin.rs
    │   ├── update_fee_config.rs
    │   └── withdraw.rs
    ├── lib.rs            # Program ID declaration and module exports
    ├── parsers/     # Account and instruction parsing functions
    │   ├── accounts.rs
    │   ├── instructions.rs
    │   └── mod.rs            # Module declarations
    └── types/     # Custom type definitions
        └── mod.rs            # Module declarations

```

## Usage

### Basic Import

```rust
use sol_pump_amm_interface::*;
```

### Program ID

The program ID is declared as a constant:

```rust
use sol_pump_amm_interface::ID;
// or
use solana_program::declare_id;
// ID is accessible as sol_pump_amm_interface::ID
```


### Instructions

```rust
use sol_pump_amm_interface::*;

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
use sol_pump_amm_interface::*;

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
solores idls/idl-core/pending_dex_idls/pump_amm.json --output-dir idls/idl-core/crates/sol_pump_amm_interface --output-crate-name sol_pump_amm_interface
```
