# sol_openbook_interface

Generated Rust interface for the **openbook** Solana program.

- **Program Name**: openbook
- **Program Version**: 0.0.0
- **Program ID**: `srmqPiDkJokFGWxoUyxFUZZiHFfvYAz4bE8vHwPAYDC`

## Directory Structure

```
sol_openbook_interface/ 
├── Cargo.toml
├── idl.json              # Original IDL file
├── README.md             # This file
└── src/
    ├── accounts/     # Program account structures
    │   ├── event.rs
    │   ├── eventqueueheader.rs
    │   ├── marketstate.rs
    │   ├── marketstatev2.rs
    │   ├── mod.rs            # Module declarations
    │   ├── openorders.rs
    │   ├── request.rs
    │   └── requestqueueheader.rs
    ├── errors.rs            # Program error definitions
    ├── events/     # Event structures and discriminators
    │   └── mod.rs            # Module declarations
    ├── instructions/     # Instruction definitions and account structures
    │   ├── cancel_order.rs
    │   ├── cancel_order_by_client_id.rs
    │   ├── cancel_order_by_client_id_v2.rs
    │   ├── cancel_order_v2.rs
    │   ├── cancel_orders_by_client_ids.rs
    │   ├── close_open_orders.rs
    │   ├── consume_events.rs
    │   ├── consume_events_permissioned.rs
    │   ├── disable_market.rs
    │   ├── init_open_orders.rs
    │   ├── initialize_market.rs
    │   ├── match_orders.rs
    │   ├── mod.rs            # Module declarations
    │   ├── new_order.rs
    │   ├── new_order_v2.rs
    │   ├── new_order_v3.rs
    │   ├── prune.rs
    │   ├── replace_order_by_client_id.rs
    │   ├── replace_orders_by_client_ids.rs
    │   ├── send_take.rs
    │   ├── settle_funds.rs
    │   └── sweep_fees.rs
    ├── lib.rs            # Program ID declaration and module exports
    ├── parsers/     # Account and instruction parsing functions
    │   ├── accounts.rs
    │   ├── instructions.rs
    │   └── mod.rs            # Module declarations
    └── types/     # Custom type definitions
        ├── cancel_order_instruction_v2.rs
        ├── event_view.rs
        ├── fee_tier.rs
        ├── initialize_market_instruction.rs
        ├── mod.rs            # Module declarations
        ├── new_order_instruction_v1.rs
        ├── new_order_instruction_v2.rs
        ├── new_order_instruction_v3.rs
        ├── order_type.rs
        ├── request_view.rs
        ├── self_trade_behavior.rs
        ├── send_take_instruction.rs
        └── side.rs

```

## Usage

### Basic Import

```rust
use sol_openbook_interface::*;
```

### Program ID

The program ID is declared as a constant:

```rust
use sol_openbook_interface::ID;
// or
use solana_program::declare_id;
// ID is accessible as sol_openbook_interface::ID
```


### Instructions

```rust
use sol_openbook_interface::*;

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
use sol_openbook_interface::*;

// Deserialize account data
let account_data = SomeAccount::try_from_slice(&account_info.data.borrow())?;
```

### Parsing

```rust
use sol_openbook_interface::*;

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
solores idls/open-book.json --output-dir ./test_batch_headers/sol_openbook_interface --output-crate-name sol_openbook_interface
```
