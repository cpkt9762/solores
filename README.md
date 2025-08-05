# Solores - Solana IDL to Rust Interface Generator

[![Crates.io](https://img.shields.io/crates/v/solores.svg)](https://crates.io/crates/solores)
[![Documentation](https://docs.rs/solores/badge.svg)](https://docs.rs/solores)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A powerful and reliable Solana IDL to Rust client/CPI interface generator that achieves **100% compilation success rate** across diverse IDL formats.

## ✨ Key Features

- **🎯 100% Compilation Success**: Tested on 16+ major Solana protocols with zero errors, zero warnings
- **🚀 Multi-Format Support**: Seamlessly handles Anchor, Shank, and Bincode IDL formats
- **📦 Smart Code Generation**: Produces idiomatic Rust code with proper type mappings
- **🔍 Built-in Parser Generation**: Automatically generates instruction and account parsers
- **📝 Rich Documentation**: Preserves and maps IDL documentation to generated code
- **⚡ Production Ready**: Battle-tested with complex protocols like Whirlpool, Squads, Phoenix

## 🏆 Proven Reliability

Successfully generates fully compilable interfaces for:

- **DEX/AMM**: Raydium, Phoenix, OpenBook, Whirlpool, Saros
- **DeFi**: Squads Multisig, Boop, DLMM
- **Launchpads**: Pump.fun, Moonshot, Raydium Launchpad
- **Trading**: Serum DEX
- And many more...

## 📦 Installation

```bash
cargo install solores
```

Or build from source:

```bash
git clone https://github.com/your-username/solores
cd solores
cargo build --release
```

## 🚀 Quick Start

### Basic Usage

Generate a complete Rust interface from any Solana IDL:

```bash
# Generate from Anchor IDL
solores path/to/anchor_idl.json

# Specify output directory and package name
solores path/to/idl.json -o ./output -n my_program

# Generate with parser support (recommended)
solores path/to/idl.json --generate-parser
```

### Generated Package Structure

```
sol_program_interface/
├── Cargo.toml              # Configured dependencies
├── README.md               # Auto-generated documentation
├── idl.json               # Original IDL for reference
└── src/
    ├── lib.rs             # Module exports and program ID
    ├── instructions/      # Instruction builders and invokers
    │   ├── mod.rs
    │   └── *.rs           # One file per instruction
    ├── types/             # Custom types and structs
    │   ├── mod.rs
    │   └── *.rs           # One file per type
    ├── accounts/          # Account structures (Anchor)
    │   ├── mod.rs
    │   └── *.rs           # One file per account
    ├── events/            # Event definitions (Anchor)
    │   ├── mod.rs
    │   └── *.rs           # One file per event
    ├── errors.rs          # Error enums and conversions
    └── parsers/           # Optional parser module
        ├── mod.rs
        ├── instructions.rs # Instruction deserializer
        └── accounts.rs     # Account deserializer
```

## 💡 Usage Examples

### Client-Side Usage

```rust
use sol_raydium_interface::{BuyExactInKeys, BuyExactInIxArgs, buy_exact_in_ix};
use solana_program::pubkey::Pubkey;

async fn create_buy_instruction() -> Result<()> {
    let keys = BuyExactInKeys {
        payer: wallet_pubkey,
        authority: authority_pda,
        pool_state: pool_pubkey,
        // ... other accounts
    };
    
    let args = BuyExactInIxArgs {
        amount_in: 1_000_000,
        minimum_amount_out: 950_000,
    };
    
    let instruction = buy_exact_in_ix(keys, args)?;
    // Send instruction in transaction...
    Ok(())
}
```

### CPI (Cross-Program Invocation) Usage

```rust
use sol_raydium_interface::{BuyExactInAccounts, BuyExactInIxArgs, buy_exact_in_invoke_signed};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

pub fn process_buy(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let accounts = BuyExactInAccounts {
        payer: &accounts[0],
        authority: &accounts[1],
        pool_state: &accounts[2],
        // ... map other accounts
    };
    
    let args = BuyExactInIxArgs {
        amount_in: amount,
        minimum_amount_out: amount * 95 / 100, // 5% slippage
    };
    
    buy_exact_in_invoke_signed(accounts, args, &[&[b"authority", &[bump]]])
}
```

### Parser Usage (with `--generate-parser`)

```rust
use sol_raydium_interface::parsers::{parse_instruction, ProgramInstruction};

fn handle_instruction(data: &[u8]) -> Result<()> {
    match parse_instruction(data)? {
        ProgramInstruction::Initialize(args) => {
            println!("Initialize with: {:?}", args);
        }
        ProgramInstruction::BuyExactIn(args) => {
            println!("Buy exact in: {} tokens", args.amount_in);
        }
        // ... handle other instructions
    }
    Ok(())
}
```

## 🛠️ Advanced Features

### Parser Generation

Generate comprehensive parsers for instruction and account deserialization:

```bash
# Generate full interface with parsers
solores idl.json --generate-parser

# Generate only parsers (skip other modules)
solores idl.json --parser-only
```

The parser module includes:
- **Instruction Parser**: Deserializes instruction data with automatic discriminator detection
- **Account Parser**: Identifies and deserializes account types
- **Comprehensive Tests**: Auto-generated test suite with validation

### Zero-Copy Support

Enable zero-copy deserialization for specific types:

```bash
solores idl.json -z LargeDataStruct -z OrderBook
```

This adds `#[repr(C)]`, `Pod`, and `Zeroable` derives for efficient memory mapping.

### Custom Program ID

Override the program ID in the IDL:

```bash
solores idl.json -p "YourProgram1111111111111111111111111111111"
```

## 🎯 Type Mapping Intelligence

Solores intelligently handles complex type mappings:

- **SmallVec → Vec**: Automatically converts `SmallVec<T,N>` to `Vec<T>`
- **Field Name Conversion**: Smart camelCase to snake_case (preserves special cases like `X64`)
- **Array Types**: Proper handling of fixed-size arrays with type-safe indexing
- **Option/Vec Nesting**: Correctly handles deeply nested generic types
- **Discriminator Handling**: Supports both Anchor (8-byte) and native (1-byte) discriminators

## 📊 Comparison with Similar Tools

| Feature | Solores | anchor-gen | solita |
|---------|---------|------------|--------|
| Zero Dependencies on Anchor | ✅ | ❌ | ✅ |
| Human-Readable Output | ✅ | ❌ | ✅ |
| Parser Generation | ✅ | ❌ | ❌ |
| 100% Compilation Rate | ✅ | ❌ | N/A |
| SmallVec Support | ✅ | ❌ | ❌ |
| Multi-file Organization | ✅ | ❌ | ✅ |
| Rust Native | ✅ | ✅ | ❌ (TypeScript) |

## 🔧 CLI Options

```
solores [OPTIONS] <IDL_PATH>

Arguments:
  <IDL_PATH>  Path to the IDL JSON file

Options:
  -o, --output <DIR>          Output directory [default: ./]
  -n, --name <NAME>           Package name [default: derived from IDL]
  -p, --program-id <PUBKEY>   Override program ID
  -z, --zero-copy <TYPE>      Enable zero-copy for type (can be repeated)
  --generate-parser           Generate parser module for instructions and accounts
  --parser-only              Generate only parser module, skip other modules
  -h, --help                 Print help
  -V, --version              Print version
```

## 📚 Generated Module Documentation

Each generated module is fully documented with:
- Comprehensive doc comments from IDL
- Usage examples in module headers
- Type safety guarantees
- Discriminator constants
- Account length constants

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

MIT License - see [LICENSE](LICENSE) for details

## 🙏 Acknowledgments

> *"solita, light of my life, fire of my loins"*

Inspired by [solita](https://github.com/metaplex-foundation/solita) and the Solana ecosystem.

---

**Built with ❤️ for the Solana community**