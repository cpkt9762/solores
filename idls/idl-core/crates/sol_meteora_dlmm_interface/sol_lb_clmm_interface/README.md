# sol_lb_clmm_interface

Generated Rust interface for the **lb_clmm** Solana program.

- **Program Name**: lb_clmm
- **Program Version**: 0.10.0
- **Program ID**: `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo`

## Directory Structure

```
sol_lb_clmm_interface/ 
├── Cargo.toml
├── idl.json              # Original IDL file
├── README.md             # This file
└── src/
    ├── accounts/     # Program account structures
    │   ├── bin_array.rs
    │   ├── bin_array_bitmap_extension.rs
    │   ├── claim_fee_operator.rs
    │   ├── dummy_zc_account.rs
    │   ├── lb_pair.rs
    │   ├── mod.rs            # Module declarations
    │   ├── oracle.rs
    │   ├── position.rs
    │   ├── position_v_2.rs
    │   ├── preset_parameter.rs
    │   ├── preset_parameter_2.rs
    │   └── token_badge.rs
    ├── errors.rs            # Program error definitions
    ├── events/     # Event structures and discriminators
    │   ├── add_liquidity.rs
    │   ├── claim_fee.rs
    │   ├── claim_fee_2.rs
    │   ├── claim_reward.rs
    │   ├── claim_reward_2.rs
    │   ├── composition_fee.rs
    │   ├── decrease_position_length.rs
    │   ├── dynamic_fee_parameter_update.rs
    │   ├── fee_parameter_update.rs
    │   ├── fund_reward.rs
    │   ├── go_to_a_bin.rs
    │   ├── increase_observation.rs
    │   ├── increase_position_length.rs
    │   ├── initialize_reward.rs
    │   ├── lb_pair_create.rs
    │   ├── mod.rs            # Module declarations
    │   ├── position_close.rs
    │   ├── position_create.rs
    │   ├── rebalancing.rs
    │   ├── remove_liquidity.rs
    │   ├── swap.rs
    │   ├── update_position_lock_release_point.rs
    │   ├── update_position_operator.rs
    │   ├── update_reward_duration.rs
    │   ├── update_reward_funder.rs
    │   └── withdraw_ineligible_reward.rs
    ├── instructions/     # Instruction definitions and account structures
    │   ├── add_liquidity.rs
    │   ├── add_liquidity_2.rs
    │   ├── add_liquidity_by_strategy.rs
    │   ├── add_liquidity_by_strategy_2.rs
    │   ├── add_liquidity_by_strategy_one_side.rs
    │   ├── add_liquidity_by_weight.rs
    │   ├── add_liquidity_one_side.rs
    │   ├── add_liquidity_one_side_precise.rs
    │   ├── add_liquidity_one_side_precise_2.rs
    │   ├── claim_fee.rs
    │   ├── claim_fee_2.rs
    │   ├── claim_reward.rs
    │   ├── claim_reward_2.rs
    │   ├── close_claim_protocol_fee_operator.rs
    │   ├── close_position.rs
    │   ├── close_position_2.rs
    │   ├── close_position_if_empty.rs
    │   ├── close_preset_parameter.rs
    │   ├── close_preset_parameter_2.rs
    │   ├── create_claim_protocol_fee_operator.rs
    │   ├── decrease_position_length.rs
    │   ├── for_idl_type_generation_do_not_call.rs
    │   ├── fund_reward.rs
    │   ├── go_to_a_bin.rs
    │   ├── increase_oracle_length.rs
    │   ├── increase_position_length.rs
    │   ├── initialize_bin_array.rs
    │   ├── initialize_bin_array_bitmap_extension.rs
    │   ├── initialize_customizable_permissionless_lb_pair.rs
    │   ├── initialize_customizable_permissionless_lb_pair_2.rs
    │   ├── initialize_lb_pair.rs
    │   ├── initialize_lb_pair_2.rs
    │   ├── initialize_permission_lb_pair.rs
    │   ├── initialize_position.rs
    │   ├── initialize_position_by_operator.rs
    │   ├── initialize_position_pda.rs
    │   ├── initialize_preset_parameter.rs
    │   ├── initialize_preset_parameter_2.rs
    │   ├── initialize_reward.rs
    │   ├── initialize_token_badge.rs
    │   ├── migrate_bin_array.rs
    │   ├── migrate_position.rs
    │   ├── mod.rs            # Module declarations
    │   ├── rebalance_liquidity.rs
    │   ├── remove_all_liquidity.rs
    │   ├── remove_liquidity.rs
    │   ├── remove_liquidity_2.rs
    │   ├── remove_liquidity_by_range.rs
    │   ├── remove_liquidity_by_range_2.rs
    │   ├── set_activation_point.rs
    │   ├── set_pair_status.rs
    │   ├── set_pair_status_permissionless.rs
    │   ├── set_pre_activation_duration.rs
    │   ├── set_pre_activation_swap_address.rs
    │   ├── swap.rs
    │   ├── swap_2.rs
    │   ├── swap_exact_out.rs
    │   ├── swap_exact_out_2.rs
    │   ├── swap_with_price_impact.rs
    │   ├── swap_with_price_impact_2.rs
    │   ├── update_base_fee_parameters.rs
    │   ├── update_dynamic_fee_parameters.rs
    │   ├── update_fees_and_reward_2.rs
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
        ├── accounts_type.rs
        ├── activation_type.rs
        ├── add_liquidity_params.rs
        ├── add_liquidity_single_side_precise_parameter.rs
        ├── add_liquidity_single_side_precise_parameter_2.rs
        ├── base_fee_parameter.rs
        ├── bin.rs
        ├── bin_liquidity_distribution.rs
        ├── bin_liquidity_distribution_by_weight.rs
        ├── bin_liquidity_reduction.rs
        ├── compressed_bin_deposit_amount.rs
        ├── customizable_params.rs
        ├── dummy_ix.rs
        ├── dynamic_fee_parameter.rs
        ├── fee_info.rs
        ├── init_permission_pair_ix.rs
        ├── init_preset_parameters_2_ix.rs
        ├── init_preset_parameters_ix.rs
        ├── initialize_lb_pair_2_params.rs
        ├── liquidity_one_side_parameter.rs
        ├── liquidity_parameter.rs
        ├── liquidity_parameter_by_strategy.rs
        ├── liquidity_parameter_by_strategy_one_side.rs
        ├── liquidity_parameter_by_weight.rs
        ├── mod.rs            # Module declarations
        ├── pair_status.rs
        ├── pair_type.rs
        ├── position_bin_data.rs
        ├── protocol_fee.rs
        ├── rebalance_liquidity_params.rs
        ├── remaining_accounts_info.rs
        ├── remaining_accounts_slice.rs
        ├── remove_liquidity_params.rs
        ├── resize_side.rs
        ├── reward_info.rs
        ├── rounding.rs
        ├── static_parameters.rs
        ├── strategy_parameters.rs
        ├── strategy_type.rs
        ├── token_program_flags.rs
        ├── user_reward_info.rs
        └── variable_parameters.rs

```

## Usage

### Basic Import

```rust
use sol_lb_clmm_interface::*;
```

### Program ID

The program ID is declared as a constant:

```rust
use sol_lb_clmm_interface::ID;
// or
use solana_program::declare_id;
// ID is accessible as sol_lb_clmm_interface::ID
```


### Instructions

```rust
use sol_lb_clmm_interface::*;

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
use sol_lb_clmm_interface::*;

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
solores idls/dlmm.json --output-dir idls/idl-core/crates/sol_meteora_dlmm_interface/sol_lb_clmm_interface --output-crate-name sol_lb_clmm_interface
```
