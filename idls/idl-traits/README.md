# idl-traits - IDL 解析器 Trait 库

为 Solana IDL 生成的解析器提供统一的 trait 接口和自动派生宏。

## 🎯 核心功能

- **统一 trait 接口** - Parser, ProgramParser trait 定义
- **事件解析支持** - ProgramParser 扩展了 `try_parse_any_event` 方法
- **零配置派生宏** - `#[derive(InstructionParser)]`, `#[derive(AccountParser)]`
- **默认空实现** - 所有方法都有合理的默认值

## 🚀 快速开始

### 在 IDL 生成的接口库中使用

```rust
// Cargo.toml
[dependencies]
idl-traits = { path = "../idl-traits" }

// 在生成的接口库中
use idl_traits::*;

// 指令解析器 - 自动支持事件解析扩展
#[derive(InstructionParser, Debug, Clone, Copy)]
pub struct MyInstructionParser;

// 账户解析器  
#[derive(AccountParser, Debug, Clone, Copy)]
pub struct MyAccountParser;
```

### 使用解析器

```rust
// 指令解析
let instruction_parser = MyInstructionParser;
let parsed = instruction_parser.parse(&instruction_update)?;

// 事件解析 (如果需要)
if let Some(event) = instruction_parser.try_parse_any_event(&cpi_log_data) {
    // 处理解析到的事件
    if let Some(trade_event) = event.try_as::<TradeEvent>() {
        println!("交易事件: {:?}", trade_event);
    }
}

// 账户解析
let account_parser = MyAccountParser;
let account = account_parser.parse(&account_update)?;
```

## 📚 API 文档

### Parser trait

基础解析器接口，定义所有解析器的通用方法。

### ProgramParser trait

程序级解析器接口，扩展了事件解析功能：

- `program_id()` - 获取程序ID
- `try_parse_any_event()` - 解析事件数据 (默认返回 None)

### 派生宏

- `#[derive(InstructionParser)]` - 自动实现指令解析器
- `#[derive(AccountParser)]` - 自动实现账户解析器

## 🔧 要求

使用派生宏的 crate 必须提供：

1. **程序ID**: `pub const ID: Pubkey = ...;`
2. **解析函数**: 
   - `parsers::instructions::parse_instruction()` (用于 InstructionParser)
   - `parsers::accounts::try_unpack_account()` (用于 AccountParser)

## 🎯 设计理念

- **向后兼容** - 现有代码无需修改
- **按需启用** - 事件解析功能可选
- **零配置** - 派生宏自动生成所有必需代码
- **类型安全** - 编译时保证接口正确性

---

**idl-traits v0.1.0** - 让 IDL 解析器开发更简单！