# sol_stable_swap_interface

Generated Rust interface for the **stable_swap** Solana program.

- **Program Name**: stable_swap
- **Program Version**: 1.5.0
- **Program ID**: `swapNyd8XiQwJ6ianp9snpu4brUqFxadzvHebnAXjJZ`

## Directory Structure

```
sol_stable_swap_interface/ 
├── Cargo.toml
├── idl.json              # Original IDL file
├── README.md             # This file
└── src/
    ├── accounts/     # Program account structures
    │   ├── mod.rs            # Module declarations
    │   ├── pool.rs
    │   ├── strategy.rs
    │   └── vault.rs
    ├── events/     # Event structures and discriminators
    │   ├── mod.rs            # Module declarations
    │   ├── pool_balance_updated_event.rs
    │   └── pool_updated_event.rs
    ├── instructions/     # Instruction definitions and account structures
    │   ├── accept_owner.rs
    │   ├── approve_strategy.rs
    │   ├── change_amp_factor.rs
    │   ├── change_max_supply.rs
    │   ├── change_swap_fee.rs
    │   ├── change_swap_fee_privileged.rs
    │   ├── close_strategy.rs
    │   ├── create_strategy.rs
    │   ├── deposit.rs
    │   ├── exec_strategy.rs
    │   ├── initialize.rs
    │   ├── mod.rs            # Module declarations
    │   ├── pause.rs
    │   ├── reject_owner.rs
    │   ├── shutdown.rs
    │   ├── swap.rs
    │   ├── swap_v_2.rs
    │   ├── transfer_owner.rs
    │   ├── unpause.rs
    │   └── withdraw.rs
    ├── lib.rs            # Program ID declaration and module exports
    ├── parsers/     # Account and instruction parsing functions
    │   ├── accounts.rs
    │   ├── instructions.rs
    │   └── mod.rs            # Module declarations
    └── types/     # Custom type definitions
        ├── mod.rs            # Module declarations
        ├── pool_balance_updated_data.rs
        ├── pool_token.rs
        └── pool_updated_data.rs

```

## Usage

### Basic Import

```rust
use sol_stable_swap_interface::*;
```

### Program ID

The program ID is declared as a constant:

```rust
use sol_stable_swap_interface::ID;
// or
use solana_program::declare_id;
// ID is accessible as sol_stable_swap_interface::ID
```


### Instructions

```rust
use sol_stable_swap_interface::*;

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
use sol_stable_swap_interface::*;

// Deserialize account data
let account_data = SomeAccount::try_from_slice(&account_info.data.borrow())?;
```

## Modules

- **instructions/**: Contains instruction definitions, account structures, and helper functions for invoking program instructions.
- **accounts/**: Program account structures with Borsh serialization support.
- **types/**: Custom type definitions used by the program, including enums and structs.
- **events/**: Event structures emitted by the program with discriminators for parsing.

## Generated with

This crate was generated using [solores](https://github.com/cpkt9762/solores) - a Solana IDL to Rust client code generator.

```bash
solores idls/idl-core/pending_dex_idls/stable_swap.json --output-dir idls/idl-core/crates/sol_stable_swap_interface --output-crate-name sol_stable_swap_interface
```
