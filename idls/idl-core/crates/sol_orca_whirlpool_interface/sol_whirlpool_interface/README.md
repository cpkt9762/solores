# sol_whirlpool_interface

Generated Rust interface for the **whirlpool** Solana program.

- **Program Name**: whirlpool
- **Program Version**: 0.3.0
- **Program ID**: `whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc`

## Directory Structure

```
sol_whirlpool_interface/ 
├── Cargo.toml
├── idl.json              # Original IDL file
├── README.md             # This file
└── src/
    ├── accounts/     # Program account structures
    │   ├── fee_tier.rs
    │   ├── mod.rs            # Module declarations
    │   ├── position.rs
    │   ├── position_bundle.rs
    │   ├── tick_array.rs
    │   ├── token_badge.rs
    │   ├── whirlpool.rs
    │   ├── whirlpools_config.rs
    │   └── whirlpools_config_extension.rs
    ├── errors.rs            # Program error definitions
    ├── instructions/     # Instruction definitions and account structures
    │   ├── close_bundled_position.rs
    │   ├── close_position.rs
    │   ├── collect_fees.rs
    │   ├── collect_fees_v_2.rs
    │   ├── collect_protocol_fees.rs
    │   ├── collect_protocol_fees_v_2.rs
    │   ├── collect_reward.rs
    │   ├── collect_reward_v_2.rs
    │   ├── decrease_liquidity.rs
    │   ├── decrease_liquidity_v_2.rs
    │   ├── delete_position_bundle.rs
    │   ├── delete_token_badge.rs
    │   ├── increase_liquidity.rs
    │   ├── increase_liquidity_v_2.rs
    │   ├── initialize_config.rs
    │   ├── initialize_config_extension.rs
    │   ├── initialize_fee_tier.rs
    │   ├── initialize_pool.rs
    │   ├── initialize_pool_v_2.rs
    │   ├── initialize_position_bundle.rs
    │   ├── initialize_position_bundle_with_metadata.rs
    │   ├── initialize_reward.rs
    │   ├── initialize_reward_v_2.rs
    │   ├── initialize_tick_array.rs
    │   ├── initialize_token_badge.rs
    │   ├── mod.rs            # Module declarations
    │   ├── open_bundled_position.rs
    │   ├── open_position.rs
    │   ├── open_position_with_metadata.rs
    │   ├── set_collect_protocol_fees_authority.rs
    │   ├── set_config_extension_authority.rs
    │   ├── set_default_fee_rate.rs
    │   ├── set_default_protocol_fee_rate.rs
    │   ├── set_fee_authority.rs
    │   ├── set_fee_rate.rs
    │   ├── set_protocol_fee_rate.rs
    │   ├── set_reward_authority.rs
    │   ├── set_reward_authority_by_super_authority.rs
    │   ├── set_reward_emissions.rs
    │   ├── set_reward_emissions_super_authority.rs
    │   ├── set_reward_emissions_v_2.rs
    │   ├── set_token_badge_authority.rs
    │   ├── swap.rs
    │   ├── swap_v_2.rs
    │   ├── two_hop_swap.rs
    │   ├── two_hop_swap_v_2.rs
    │   └── update_fees_and_rewards.rs
    ├── lib.rs            # Program ID declaration and module exports
    ├── parsers/     # Account and instruction parsing functions
    │   ├── accounts.rs
    │   ├── instructions.rs
    │   └── mod.rs            # Module declarations
    └── types/     # Custom type definitions
        ├── accounts_type.rs
        ├── curr_index.rs
        ├── direction.rs
        ├── mod.rs            # Module declarations
        ├── open_position_bumps.rs
        ├── open_position_with_metadata_bumps.rs
        ├── position_reward_info.rs
        ├── remaining_accounts_info.rs
        ├── remaining_accounts_slice.rs
        ├── tick.rs
        ├── tick_label.rs
        ├── whirlpool_bumps.rs
        └── whirlpool_reward_info.rs

```

## Usage

### Basic Import

```rust
use sol_whirlpool_interface::*;
```

### Program ID

The program ID is declared as a constant:

```rust
use sol_whirlpool_interface::ID;
// or
use solana_program::declare_id;
// ID is accessible as sol_whirlpool_interface::ID
```


### Instructions

```rust
use sol_whirlpool_interface::*;

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
use sol_whirlpool_interface::*;

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
solores idls/whirlpool.json --output-dir idls/idl-core/crates/sol_orca_whirlpool_interface/sol_whirlpool_interface --output-crate-name sol_whirlpool_interface
```
