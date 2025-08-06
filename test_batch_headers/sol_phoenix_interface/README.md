# sol_phoenix_interface

Generated Rust interface for the **phoenix** Solana program.

- **Program Name**: phoenix
- **Program Version**: 0.2.4
- **Program ID**: `PhoeNiXZ8ByJGLkxNfZRnkUfjvmuYqLR89jjFHGqdXY`

## Directory Structure

```
sol_phoenix_interface/ 
├── Cargo.toml
├── idl.json              # Original IDL file
├── README.md             # This file
└── src/
    ├── errors.rs            # Program error definitions
    ├── instructions/     # Instruction definitions and account structures
    │   ├── cancel_all_orders.rs
    │   ├── cancel_all_orders_with_free_funds.rs
    │   ├── cancel_multiple_orders_by_id.rs
    │   ├── cancel_multiple_orders_by_id_with_free_funds.rs
    │   ├── cancel_up_to.rs
    │   ├── cancel_up_to_with_free_funds.rs
    │   ├── change_fee_recipient.rs
    │   ├── change_market_status.rs
    │   ├── change_seat_status.rs
    │   ├── claim_authority.rs
    │   ├── collect_fees.rs
    │   ├── deposit_funds.rs
    │   ├── evict_seat.rs
    │   ├── force_cancel_orders.rs
    │   ├── initialize_market.rs
    │   ├── log.rs
    │   ├── mod.rs            # Module declarations
    │   ├── name_successor.rs
    │   ├── place_limit_order.rs
    │   ├── place_limit_order_with_free_funds.rs
    │   ├── place_multiple_post_only_orders.rs
    │   ├── place_multiple_post_only_orders_with_free_funds.rs
    │   ├── reduce_order.rs
    │   ├── reduce_order_with_free_funds.rs
    │   ├── request_seat.rs
    │   ├── request_seat_authorized.rs
    │   ├── swap.rs
    │   ├── swap_with_free_funds.rs
    │   └── withdraw_funds.rs
    ├── lib.rs            # Program ID declaration and module exports
    ├── parsers/     # Account and instruction parsing functions
    │   ├── instructions.rs
    │   └── mod.rs            # Module declarations
    └── types/     # Custom type definitions
        ├── audit_log_header.rs
        ├── cancel_multiple_orders_by_id_params.rs
        ├── cancel_order_params.rs
        ├── cancel_up_to_params.rs
        ├── condensed_order.rs
        ├── deposit_params.rs
        ├── evict_event.rs
        ├── expired_order_event.rs
        ├── failed_multiple_limit_order_behavior.rs
        ├── fee_event.rs
        ├── fifo_order_id.rs
        ├── fill_event.rs
        ├── fill_summary_event.rs
        ├── initialize_params.rs
        ├── market_header.rs
        ├── market_size_params.rs
        ├── market_status.rs
        ├── mod.rs            # Module declarations
        ├── multiple_order_packet.rs
        ├── order_packet.rs
        ├── phoenix_market_event.rs
        ├── place_event.rs
        ├── reduce_event.rs
        ├── reduce_order_params.rs
        ├── seat.rs
        ├── seat_approval_status.rs
        ├── self_trade_behavior.rs
        ├── side.rs
        ├── time_in_force_event.rs
        ├── token_params.rs
        └── withdraw_params.rs

```

## Usage

### Basic Import

```rust
use sol_phoenix_interface::*;
```

### Program ID

The program ID is declared as a constant:

```rust
use sol_phoenix_interface::ID;
// or
use solana_program::declare_id;
// ID is accessible as sol_phoenix_interface::ID
```


### Instructions

```rust
use sol_phoenix_interface::*;

// Create instruction
let instruction = some_instruction_ix(
    SomeInstructionKeys { /* account keys */ },
    SomeInstructionIxArgs { /* instruction args */ }
)?;

// Invoke instruction  
some_instruction_invoke(accounts, args)?;
```

### Parsing

```rust
use sol_phoenix_interface::*;

// Parse account data
let parsed_account = try_unpack_account(&account_data)?;

// Parse instruction data  
let parsed_instruction = parse_instruction(&instruction_data)?;
```

## Modules

- **instructions/**: Contains instruction definitions, account structures, and helper functions for invoking program instructions.
- **types/**: Custom type definitions used by the program, including enums and structs.
- **errors.rs**: Program-specific error definitions with conversion to `ProgramError`.
- **parsers/**: Utility functions for parsing account data and instruction data based on discriminators.

## Generated with

This crate was generated using [solores](https://github.com/cpkt9762/solores) - a Solana IDL to Rust client code generator.

```bash
solores idls/phoenix.json --output-dir ./test_batch_headers/sol_phoenix_interface --output-crate-name sol_phoenix_interface
```
