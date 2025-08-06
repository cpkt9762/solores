# sol_squads_interface

Generated Rust interface for the **squads** Solana program.

- **Program Name**: squads
- **Program Version**: 2.0.0
- **Program ID**: `SMPLecH534NA9acpos4G6x7uf3LWbCAwZQE9e8ZekMu`

## Directory Structure

```
sol_squads_interface/ 
├── Cargo.toml
├── idl.json              # Original IDL file
├── README.md             # This file
└── src/
    ├── accounts/     # Program account structures
    │   ├── batch.rs
    │   ├── configtransaction.rs
    │   ├── mod.rs            # Module declarations
    │   ├── multisig.rs
    │   ├── programconfig.rs
    │   ├── proposal.rs
    │   ├── spendinglimit.rs
    │   ├── vaultbatchtransaction.rs
    │   └── vaulttransaction.rs
    ├── errors.rs            # Program error definitions
    ├── instructions/     # Instruction definitions and account structures
    │   ├── batch_accounts_close.rs
    │   ├── batch_add_transaction.rs
    │   ├── batch_create.rs
    │   ├── batch_execute_transaction.rs
    │   ├── config_transaction_accounts_close.rs
    │   ├── config_transaction_create.rs
    │   ├── config_transaction_execute.rs
    │   ├── mod.rs            # Module declarations
    │   ├── multisig_add_member.rs
    │   ├── multisig_add_spending_limit.rs
    │   ├── multisig_change_threshold.rs
    │   ├── multisig_create.rs
    │   ├── multisig_create_v2.rs
    │   ├── multisig_remove_member.rs
    │   ├── multisig_remove_spending_limit.rs
    │   ├── multisig_set_config_authority.rs
    │   ├── multisig_set_rent_collector.rs
    │   ├── multisig_set_time_lock.rs
    │   ├── program_config_init.rs
    │   ├── program_config_set_authority.rs
    │   ├── program_config_set_multisig_creation_fee.rs
    │   ├── program_config_set_treasury.rs
    │   ├── proposal_activate.rs
    │   ├── proposal_approve.rs
    │   ├── proposal_cancel.rs
    │   ├── proposal_create.rs
    │   ├── proposal_reject.rs
    │   ├── spending_limit_use.rs
    │   ├── vault_batch_transaction_account_close.rs
    │   ├── vault_transaction_accounts_close.rs
    │   ├── vault_transaction_create.rs
    │   └── vault_transaction_execute.rs
    ├── lib.rs            # Program ID declaration and module exports
    ├── parsers/     # Account and instruction parsing functions
    │   ├── accounts.rs
    │   ├── instructions.rs
    │   └── mod.rs            # Module declarations
    └── types/     # Custom type definitions
        ├── batch_add_transaction_args.rs
        ├── batch_create_args.rs
        ├── compiled_instruction.rs
        ├── config_action.rs
        ├── config_transaction_create_args.rs
        ├── member.rs
        ├── message_address_table_lookup.rs
        ├── mod.rs            # Module declarations
        ├── multisig_add_member_args.rs
        ├── multisig_add_spending_limit_args.rs
        ├── multisig_change_threshold_args.rs
        ├── multisig_compiled_instruction.rs
        ├── multisig_create_args.rs
        ├── multisig_create_args_v2.rs
        ├── multisig_message_address_table_lookup.rs
        ├── multisig_remove_member_args.rs
        ├── multisig_remove_spending_limit_args.rs
        ├── multisig_set_config_authority_args.rs
        ├── multisig_set_rent_collector_args.rs
        ├── multisig_set_time_lock_args.rs
        ├── period.rs
        ├── permission.rs
        ├── permissions.rs
        ├── program_config_init_args.rs
        ├── program_config_set_authority_args.rs
        ├── program_config_set_multisig_creation_fee_args.rs
        ├── program_config_set_treasury_args.rs
        ├── proposal_create_args.rs
        ├── proposal_status.rs
        ├── proposal_vote_args.rs
        ├── spending_limit_use_args.rs
        ├── transaction_message.rs
        ├── vault_transaction_create_args.rs
        ├── vault_transaction_message.rs
        └── vote.rs

```

## Usage

### Basic Import

```rust
use sol_squads_interface::*;
```

### Program ID

The program ID is declared as a constant:

```rust
use sol_squads_interface::ID;
// or
use solana_program::declare_id;
// ID is accessible as sol_squads_interface::ID
```


### Instructions

```rust
use sol_squads_interface::*;

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
use sol_squads_interface::*;

// Deserialize account data
let account_data = SomeAccount::try_from_slice(&account_info.data.borrow())?;
```

### Parsing

```rust
use sol_squads_interface::*;

// Parse account data
let parsed_account = try_unpack_account(&account_data)?;

// Parse instruction data  
let parsed_instruction = parse_instruction(&instruction_data)?;
```

## Modules

- **instructions/**: Contains instruction definitions, account structures, and helper functions for invoking program instructions.
- **accounts/**: Program account structures with Borsh serialization support.
- **types/**: Custom type definitions used by the program, including enums and structs.
- **errors.rs**: Program-specific error definitions with conversion to `ProgramError`.
- **parsers/**: Utility functions for parsing account data and instruction data based on discriminators.

## Generated with

This crate was generated using [solores](https://github.com/cpkt9762/solores) - a Solana IDL to Rust client code generator.

```bash
solores idls/squads_multisig_program.json --output-dir ./test_batch_headers/sol_squads_interface --output-crate-name sol_squads_interface
```
