# sol_raydium_launchpad_interface

Generated Rust interface for the **raydium_launchpad** Solana program.

- **Program Name**: raydium_launchpad
- **Program Version**: 0.1.0
- **Program ID**: `LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj`

## Directory Structure

```
sol_raydium_launchpad_interface/ 
├── Cargo.toml
├── idl.json              # Original IDL file
├── README.md             # This file
└── src/
    ├── accounts/     # Program account structures
    │   ├── globalconfig.rs
    │   ├── mod.rs            # Module declarations
    │   ├── platformconfig.rs
    │   ├── poolstate.rs
    │   └── vestingrecord.rs
    ├── errors.rs            # Program error definitions
    ├── events/     # Event structures and discriminators
    │   ├── claim_vested_event.rs
    │   ├── create_vesting_event.rs
    │   ├── mod.rs            # Module declarations
    │   ├── pool_create_event.rs
    │   └── trade_event.rs
    ├── instructions/     # Instruction definitions and account structures
    │   ├── buy_exact_in.rs
    │   ├── buy_exact_out.rs
    │   ├── claim_platform_fee.rs
    │   ├── claim_vested_token.rs
    │   ├── collect_fee.rs
    │   ├── collect_migrate_fee.rs
    │   ├── create_config.rs
    │   ├── create_platform_config.rs
    │   ├── create_vesting_account.rs
    │   ├── initialize.rs
    │   ├── migrate_to_amm.rs
    │   ├── migrate_to_cpswap.rs
    │   ├── mod.rs            # Module declarations
    │   ├── sell_exact_in.rs
    │   ├── sell_exact_out.rs
    │   ├── update_config.rs
    │   └── update_platform_config.rs
    ├── lib.rs            # Program ID declaration and module exports
    ├── parsers/     # Account and instruction parsing functions
    │   ├── accounts.rs
    │   ├── instructions.rs
    │   └── mod.rs            # Module declarations
    └── types/     # Custom type definitions
        ├── constant_curve.rs
        ├── curve_params.rs
        ├── fixed_curve.rs
        ├── linear_curve.rs
        ├── migrate_nft_info.rs
        ├── mint_params.rs
        ├── mod.rs            # Module declarations
        ├── platform_config_param.rs
        ├── platform_params.rs
        ├── pool_status.rs
        ├── trade_direction.rs
        ├── vesting_params.rs
        └── vesting_schedule.rs

```

## Usage

### Basic Import

```rust
use sol_raydium_launchpad_interface::*;
```

### Program ID

The program ID is declared as a constant:

```rust
use sol_raydium_launchpad_interface::ID;
// or
use solana_program::declare_id;
// ID is accessible as sol_raydium_launchpad_interface::ID
```


### Instructions

```rust
use sol_raydium_launchpad_interface::*;

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
use sol_raydium_launchpad_interface::*;

// Deserialize account data
let account_data = SomeAccount::try_from_slice(&account_info.data.borrow())?;
```

### Parsing

```rust
use sol_raydium_launchpad_interface::*;

// Parse account data
let parsed_account = try_unpack_account(&account_data)?;

// Parse instruction data  
let parsed_instruction = parse_instruction(&instruction_data)?;
```

## Modules

- **instructions/**: Contains instruction definitions, account structures, and helper functions for invoking program instructions.
- **accounts/**: Program account structures with Borsh serialization support.
- **types/**: Custom type definitions used by the program, including enums and structs.
- **events/**: Event structures emitted by the program with discriminators for parsing.
- **errors.rs**: Program-specific error definitions with conversion to `ProgramError`.
- **parsers/**: Utility functions for parsing account data and instruction data based on discriminators.

## Generated with

This crate was generated using [solores](https://github.com/cpkt9762/solores) - a Solana IDL to Rust client code generator.

```bash
solores idls/raydium_launchpad.json --output-dir ./test_batch_headers/sol_raydium_launchpad_interface --output-crate-name sol_raydium_launchpad_interface
```
