# sol_meteora_dlmm_interface

Generated Rust interface for the **meteora_dlmm** Solana program.

- **Program Name**: meteora_dlmm
- **Program Version**: 0.8.2
- **Program ID**: `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo`

## Directory Structure

```
sol_meteora_dlmm_interface/ 
├── Cargo.toml
├── idl.json              # Original IDL file
├── README.md             # This file
└── src/
    ├── accounts/     # Program account structures
    │   ├── binarray.rs
    │   ├── binarraybitmapextension.rs
    │   ├── lbpair.rs
    │   ├── mod.rs            # Module declarations
    │   ├── oracle.rs
    │   ├── position.rs
    │   ├── positionv2.rs
    │   └── presetparameter.rs
    ├── constants.rs            # Program constants
    ├── errors.rs            # Program error definitions
    ├── events/     # Event structures and discriminators
    │   ├── add_liquidity.rs
    │   ├── claim_fee.rs
    │   ├── claim_reward.rs
    │   ├── composition_fee.rs
    │   ├── fee_parameter_update.rs
    │   ├── fund_reward.rs
    │   ├── go_to_a_bin.rs
    │   ├── increase_observation.rs
    │   ├── initialize_reward.rs
    │   ├── lb_pair_create.rs
    │   ├── mod.rs            # Module declarations
    │   ├── position_close.rs
    │   ├── position_create.rs
    │   ├── remove_liquidity.rs
    │   ├── swap.rs
    │   ├── update_position_lock_release_point.rs
    │   ├── update_position_operator.rs
    │   ├── update_reward_duration.rs
    │   ├── update_reward_funder.rs
    │   └── withdraw_ineligible_reward.rs
    ├── instructions/     # Instruction definitions and account structures
    │   ├── add_liquidity.rs
    │   ├── add_liquidity_by_strategy.rs
    │   ├── add_liquidity_by_strategy_one_side.rs
    │   ├── add_liquidity_by_weight.rs
    │   ├── add_liquidity_one_side.rs
    │   ├── add_liquidity_one_side_precise.rs
    │   ├── claim_fee.rs
    │   ├── claim_reward.rs
    │   ├── close_position.rs
    │   ├── close_preset_parameter.rs
    │   ├── fund_reward.rs
    │   ├── go_to_a_bin.rs
    │   ├── increase_oracle_length.rs
    │   ├── initialize_bin_array.rs
    │   ├── initialize_bin_array_bitmap_extension.rs
    │   ├── initialize_customizable_permissionless_lb_pair.rs
    │   ├── initialize_lb_pair.rs
    │   ├── initialize_permission_lb_pair.rs
    │   ├── initialize_position.rs
    │   ├── initialize_position_by_operator.rs
    │   ├── initialize_position_pda.rs
    │   ├── initialize_preset_parameter.rs
    │   ├── initialize_reward.rs
    │   ├── migrate_bin_array.rs
    │   ├── migrate_position.rs
    │   ├── mod.rs            # Module declarations
    │   ├── remove_all_liquidity.rs
    │   ├── remove_liquidity.rs
    │   ├── remove_liquidity_by_range.rs
    │   ├── set_activation_point.rs
    │   ├── set_pre_activation_duration.rs
    │   ├── set_pre_activation_swap_address.rs
    │   ├── swap.rs
    │   ├── swap_exact_out.rs
    │   ├── swap_with_price_impact.rs
    │   ├── toggle_pair_status.rs
    │   ├── update_fee_parameters.rs
    │   ├── update_fees_and_rewards.rs
    │   ├── update_position_operator.rs
    │   ├── update_reward_duration.rs
    │   ├── update_reward_funder.rs
    │   ├── withdraw_ineligible_reward.rs
    │   └── withdraw_protocol_fee.rs
    ├── lib.rs            # Program ID declaration and module exports
    ├── parsers/     # Account and instruction parsing functions
    │   ├── accounts.rs
    │   ├── instructions.rs
    │   └── mod.rs            # Module declarations
    └── types/     # Custom type definitions
        ├── activation_type.rs
        ├── add_liquidity_single_side_precise_parameter.rs
        ├── bin.rs
        ├── bin_liquidity_distribution.rs
        ├── bin_liquidity_distribution_by_weight.rs
        ├── bin_liquidity_reduction.rs
        ├── compressed_bin_deposit_amount.rs
        ├── customizable_params.rs
        ├── fee_info.rs
        ├── fee_parameter.rs
        ├── init_permission_pair_ix.rs
        ├── init_preset_parameters_ix.rs
        ├── layout_version.rs
        ├── liquidity_one_side_parameter.rs
        ├── liquidity_parameter.rs
        ├── liquidity_parameter_by_strategy.rs
        ├── liquidity_parameter_by_strategy_one_side.rs
        ├── liquidity_parameter_by_weight.rs
        ├── mod.rs            # Module declarations
        ├── observation.rs
        ├── pair_status.rs
        ├── pair_type.rs
        ├── protocol_fee.rs
        ├── reward_info.rs
        ├── rounding.rs
        ├── static_parameters.rs
        ├── strategy_parameters.rs
        ├── strategy_type.rs
        ├── user_reward_info.rs
        └── variable_parameters.rs

```

## Usage

### Basic Import

```rust
use sol_meteora_dlmm_interface::*;
```

### Program ID

The program ID is declared as a constant:

```rust
use sol_meteora_dlmm_interface::ID;
// or
use solana_program::declare_id;
// ID is accessible as sol_meteora_dlmm_interface::ID
```


### Instructions

```rust
use sol_meteora_dlmm_interface::*;

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
use sol_meteora_dlmm_interface::*;

// Deserialize account data
let account_data = SomeAccount::try_from_slice(&account_info.data.borrow())?;
```

### Parsing

```rust
use sol_meteora_dlmm_interface::*;

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
- **constants.rs**: Program constants and configuration values.
- **errors.rs**: Program-specific error definitions with conversion to `ProgramError`.
- **parsers/**: Utility functions for parsing account data and instruction data based on discriminators.

## Generated with

This crate was generated using [solores](https://github.com/cpkt9762/solores) - a Solana IDL to Rust client code generator.

```bash
solores idls/dlmm.json --output-dir ./test_batch_headers/sol_meteora_dlmm_interface --output-crate-name sol_meteora_dlmm_interface
```
