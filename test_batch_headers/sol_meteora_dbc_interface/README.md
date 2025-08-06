# sol_meteora_dbc_interface

Generated Rust interface for the **meteora_dbc** Solana program.

- **Program Name**: meteora_dbc
- **Program Version**: 0.1.3
- **Program ID**: `dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN`

## Directory Structure

```
sol_meteora_dbc_interface/ 
├── Cargo.toml
├── idl.json              # Original IDL file
├── README.md             # This file
└── src/
    ├── accounts/     # Program account structures
    │   ├── claimfeeoperator.rs
    │   ├── config.rs
    │   ├── lockescrow.rs
    │   ├── meteoradammmigrationmetadata.rs
    │   ├── meteoradammv2metadata.rs
    │   ├── mod.rs            # Module declarations
    │   ├── partnermetadata.rs
    │   ├── poolconfig.rs
    │   ├── virtualpool.rs
    │   └── virtualpoolmetadata.rs
    ├── errors.rs            # Program error definitions
    ├── events/     # Event structures and discriminators
    │   ├── evt_claim_creator_trading_fee.rs
    │   ├── evt_claim_protocol_fee.rs
    │   ├── evt_claim_trading_fee.rs
    │   ├── evt_close_claim_fee_operator.rs
    │   ├── evt_create_claim_fee_operator.rs
    │   ├── evt_create_config.rs
    │   ├── evt_create_damm_v2_migration_metadata.rs
    │   ├── evt_create_meteora_migration_metadata.rs
    │   ├── evt_creator_withdraw_surplus.rs
    │   ├── evt_curve_complete.rs
    │   ├── evt_initialize_pool.rs
    │   ├── evt_partner_metadata.rs
    │   ├── evt_partner_withdraw_migration_fee.rs
    │   ├── evt_partner_withdraw_surplus.rs
    │   ├── evt_protocol_withdraw_surplus.rs
    │   ├── evt_swap.rs
    │   ├── evt_update_pool_creator.rs
    │   ├── evt_virtual_pool_metadata.rs
    │   ├── evt_withdraw_leftover.rs
    │   ├── evt_withdraw_migration_fee.rs
    │   └── mod.rs            # Module declarations
    ├── instructions/     # Instruction definitions and account structures
    │   ├── claim_creator_trading_fee.rs
    │   ├── claim_protocol_fee.rs
    │   ├── claim_trading_fee.rs
    │   ├── close_claim_fee_operator.rs
    │   ├── create_claim_fee_operator.rs
    │   ├── create_config.rs
    │   ├── create_locker.rs
    │   ├── create_partner_metadata.rs
    │   ├── create_virtual_pool_metadata.rs
    │   ├── creator_withdraw_surplus.rs
    │   ├── initialize_virtual_pool_with_spl_token.rs
    │   ├── initialize_virtual_pool_with_token2022.rs
    │   ├── migrate_meteora_damm.rs
    │   ├── migrate_meteora_damm_claim_lp_token.rs
    │   ├── migrate_meteora_damm_lock_lp_token.rs
    │   ├── migration_damm_v2.rs
    │   ├── migration_damm_v2_create_metadata.rs
    │   ├── migration_meteora_damm_create_metadata.rs
    │   ├── mod.rs            # Module declarations
    │   ├── partner_withdraw_surplus.rs
    │   ├── protocol_withdraw_surplus.rs
    │   ├── swap.rs
    │   ├── transfer_pool_creator.rs
    │   ├── withdraw_leftover.rs
    │   └── withdraw_migration_fee.rs
    ├── lib.rs            # Program ID declaration and module exports
    ├── parsers/     # Account and instruction parsing functions
    │   ├── accounts.rs
    │   ├── instructions.rs
    │   └── mod.rs            # Module declarations
    └── types/     # Custom type definitions
        ├── base_fee_config.rs
        ├── base_fee_parameters.rs
        ├── claim_fee_operator.rs
        ├── config.rs
        ├── config_parameters.rs
        ├── create_partner_metadata_parameters.rs
        ├── create_virtual_pool_metadata_parameters.rs
        ├── dynamic_fee_config.rs
        ├── dynamic_fee_parameters.rs
        ├── evt_claim_creator_trading_fee.rs
        ├── evt_claim_protocol_fee.rs
        ├── evt_claim_trading_fee.rs
        ├── evt_close_claim_fee_operator.rs
        ├── evt_create_claim_fee_operator.rs
        ├── evt_create_config.rs
        ├── evt_create_damm_v2_migration_metadata.rs
        ├── evt_create_meteora_migration_metadata.rs
        ├── evt_creator_withdraw_surplus.rs
        ├── evt_curve_complete.rs
        ├── evt_initialize_pool.rs
        ├── evt_partner_metadata.rs
        ├── evt_partner_withdraw_migration_fee.rs
        ├── evt_partner_withdraw_surplus.rs
        ├── evt_protocol_withdraw_surplus.rs
        ├── evt_swap.rs
        ├── evt_update_pool_creator.rs
        ├── evt_virtual_pool_metadata.rs
        ├── evt_withdraw_leftover.rs
        ├── evt_withdraw_migration_fee.rs
        ├── initialize_pool_parameters.rs
        ├── liquidity_distribution_config.rs
        ├── liquidity_distribution_parameters.rs
        ├── lock_escrow.rs
        ├── locked_vesting_config.rs
        ├── locked_vesting_params.rs
        ├── meteora_damm_migration_metadata.rs
        ├── meteora_damm_v2_metadata.rs
        ├── migration_fee.rs
        ├── mod.rs            # Module declarations
        ├── partner_metadata.rs
        ├── pool_config.rs
        ├── pool_fee_parameters.rs
        ├── pool_fees.rs
        ├── pool_fees_config.rs
        ├── pool_metrics.rs
        ├── swap_parameters.rs
        ├── swap_result.rs
        ├── token_supply_params.rs
        ├── virtual_pool.rs
        ├── virtual_pool_metadata.rs
        └── volatility_tracker.rs

```

## Usage

### Basic Import

```rust
use sol_meteora_dbc_interface::*;
```

### Program ID

The program ID is declared as a constant:

```rust
use sol_meteora_dbc_interface::ID;
// or
use solana_program::declare_id;
// ID is accessible as sol_meteora_dbc_interface::ID
```


### Instructions

```rust
use sol_meteora_dbc_interface::*;

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
use sol_meteora_dbc_interface::*;

// Deserialize account data
let account_data = SomeAccount::try_from_slice(&account_info.data.borrow())?;
```

### Parsing

```rust
use sol_meteora_dbc_interface::*;

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
solores idls/meteora_dbc.json --output-dir ./test_batch_headers/sol_meteora_dbc_interface --output-crate-name sol_meteora_dbc_interface
```
