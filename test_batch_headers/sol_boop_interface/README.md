# sol_boop_interface

Generated Rust interface for the **boop** Solana program.

- **Program Name**: boop
- **Program Version**: 0.2.0
- **Program ID**: `boop8hVGQGqehUK2iVEMEnMrL5RbjywRzHKBmBE7ry4`

## Directory Structure

```
sol_boop_interface/ 
├── Cargo.toml
├── idl.json              # Original IDL file
├── README.md             # This file
└── src/
    ├── accounts/     # Program account structures
    │   ├── ammconfig.rs
    │   ├── bondingcurve.rs
    │   ├── config.rs
    │   ├── lockedcpliquiditystate.rs
    │   └── mod.rs            # Module declarations
    ├── errors.rs            # Program error definitions
    ├── events/     # Event structures and discriminators
    │   ├── authority_transfer_cancelled_event.rs
    │   ├── authority_transfer_completed_event.rs
    │   ├── authority_transfer_initiated_event.rs
    │   ├── bonding_curve_deployed_event.rs
    │   ├── bonding_curve_deployed_fallback_event.rs
    │   ├── bonding_curve_vault_closed_event.rs
    │   ├── config_updated_event.rs
    │   ├── liquidity_deposited_into_raydium_event.rs
    │   ├── mod.rs            # Module declarations
    │   ├── operators_added_event.rs
    │   ├── operators_removed_event.rs
    │   ├── paused_toggled_event.rs
    │   ├── raydium_liquidity_locked_event.rs
    │   ├── raydium_pool_created_event.rs
    │   ├── raydium_random_pool_created_event.rs
    │   ├── swap_sol_for_tokens_on_raydium_event.rs
    │   ├── swap_tokens_for_sol_on_raydium_event.rs
    │   ├── token_bought_event.rs
    │   ├── token_created_event.rs
    │   ├── token_created_fallback_event.rs
    │   ├── token_graduated_event.rs
    │   ├── token_sold_event.rs
    │   ├── trading_fees_collected_event.rs
    │   └── trading_fees_split_event.rs
    ├── instructions/     # Instruction definitions and account structures
    │   ├── add_operators.rs
    │   ├── buy_token.rs
    │   ├── cancel_authority_transfer.rs
    │   ├── close_bonding_curve_vault.rs
    │   ├── collect_meteora_trading_fees.rs
    │   ├── collect_trading_fees.rs
    │   ├── complete_authority_transfer.rs
    │   ├── create_meteora_pool.rs
    │   ├── create_raydium_pool.rs
    │   ├── create_raydium_random_pool.rs
    │   ├── create_token.rs
    │   ├── create_token_fallback.rs
    │   ├── deploy_bonding_curve.rs
    │   ├── deploy_bonding_curve_fallback.rs
    │   ├── deposit_into_raydium.rs
    │   ├── graduate.rs
    │   ├── initialize.rs
    │   ├── initiate_authority_transfer.rs
    │   ├── lock_raydium_liquidity.rs
    │   ├── mod.rs            # Module declarations
    │   ├── remove_operators.rs
    │   ├── sell_token.rs
    │   ├── split_trading_fees.rs
    │   ├── swap_sol_for_tokens_on_raydium.rs
    │   ├── swap_tokens_for_sol_on_raydium.rs
    │   ├── toggle_paused.rs
    │   └── update_config.rs
    ├── lib.rs            # Program ID declaration and module exports
    ├── parsers/     # Account and instruction parsing functions
    │   ├── accounts.rs
    │   ├── instructions.rs
    │   └── mod.rs            # Module declarations
    └── types/     # Custom type definitions
        ├── amm_config.rs
        ├── authority_transfer_cancelled_event.rs
        ├── authority_transfer_completed_event.rs
        ├── authority_transfer_initiated_event.rs
        ├── bonding_curve.rs
        ├── bonding_curve_deployed_event.rs
        ├── bonding_curve_deployed_fallback_event.rs
        ├── bonding_curve_status.rs
        ├── bonding_curve_vault_closed_event.rs
        ├── config.rs
        ├── config_updated_event.rs
        ├── liquidity_deposited_into_raydium_event.rs
        ├── locked_cp_liquidity_state.rs
        ├── mod.rs            # Module declarations
        ├── operators_added_event.rs
        ├── operators_removed_event.rs
        ├── paused_toggled_event.rs
        ├── raydium_liquidity_locked_event.rs
        ├── raydium_pool_created_event.rs
        ├── raydium_random_pool_created_event.rs
        ├── swap_sol_for_tokens_on_raydium_event.rs
        ├── swap_tokens_for_sol_on_raydium_event.rs
        ├── token_bought_event.rs
        ├── token_created_event.rs
        ├── token_created_fallback_event.rs
        ├── token_graduated_event.rs
        ├── token_sold_event.rs
        ├── trading_fees_collected_event.rs
        └── trading_fees_split_event.rs

```

## Usage

### Basic Import

```rust
use sol_boop_interface::*;
```

### Program ID

The program ID is declared as a constant:

```rust
use sol_boop_interface::ID;
// or
use solana_program::declare_id;
// ID is accessible as sol_boop_interface::ID
```


### Instructions

```rust
use sol_boop_interface::*;

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
use sol_boop_interface::*;

// Deserialize account data
let account_data = SomeAccount::try_from_slice(&account_info.data.borrow())?;
```

### Parsing

```rust
use sol_boop_interface::*;

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
solores idls/boop.json --output-dir ./test_batch_headers/sol_boop_interface --output-crate-name sol_boop_interface
```
