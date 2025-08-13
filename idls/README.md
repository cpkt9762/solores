# Solana IDL Collection

这个目录包含Solana生态系统中重要程序的Interface Description Language (IDL)文件，已经过标准化处理并添加了完整的元数据信息。

## 📁 目录结构

### 🏛️ **主要协议程序** - 根目录

#### 💰 DeFi协议 (6个)
| 文件名 | 程序地址 | 描述 | 类型 |
|--------|----------|------|------|
| `raydium.json` | `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8` | Raydium AMM和流动性提供 | NonAnchor |
| `dlmm.json` | `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo` | Meteora DLMM动态流动性市场做市商 | Anchor |
| `whirlpool.json` | `whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc` | Orca Whirlpool集中流动性AMM | Anchor |
| `stable_swap.json` | `swapNyd8XiQwJ6ianp9snpu4brUqFxadzvHebnAXjJZ` | 稳定币交换的低滑点AMM | Anchor |
| `lifinity.json` | `EewxydAPCCVuNEyrVN68PuSYdQ7wKn27V9Gjeoi8dy3S` | Lifinity主动做市商 | Anchor |
| `pump_amm.json` | `pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA` | Pump AMM自动做市协议 | Anchor |
| `meteora_dbc.json` | `dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN` | Meteora动态联合曲线 | Anchor |
| `saros.json` | `SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ` | Saros跨链AMM | NonAnchor |

#### 📈 交易协议 (2个)
| 文件名 | 程序地址 | 描述 | 类型 |
|--------|----------|------|------|
| `phoenix.json` | `PhoeNiXZ8ByJGLkxNfZRnkUfjvmuYqLR89jjFHGqdXY` | Phoenix高性能中央限价订单簿 | Shank |
| `serum.json` | `9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin` | Serum去中心化交易所 | Serum-fork |

#### 🚀 启动平台 (4个)
| 文件名 | 程序地址 | 描述 | 类型 |
|--------|----------|------|------|
| `boop.json` | `boop8hVGQGqehUK2iVEMEnMrL5RbjywRzHKBmBE7ry4` | Boop代币启动器 | Anchor |
| `moonshot.json` | `MoonCVVNZFSYkqNXP6bxHLPL6QQJiMagDL3qcqUQTrG` | Moonshot代币启动平台 | Anchor |
| `pump-fun-idl.json` | `6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P` | PumpFun代币启动器 | Anchor |
| `raydium_launchpad.json` | `LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj` | Raydium启动平台 | Anchor |

#### 🏗️ 基础设施 (1个)
| 文件名 | 程序地址 | 描述 | 类型 |
|--------|----------|------|------|
| `squads_multisig_program.json` | `SMPLecH534NA9acpos4G6x7uf3LWbCAwZQE9e8ZekMu` | Squads多签钱包 | Anchor |

### 📦 **SPL程序** - `/spl/` 目录

SPL程序是Solana生态系统的核心构建块，提供标准化的Token和其他功能实现。

| 文件名 | 程序地址 | 描述 |
|--------|----------|------|
| `token.json` | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` | SPL Token程序 - 标准Token实现 |
| `token-2022.json` | `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` | SPL Token-2022程序 - 下一代Token程序 |
| `associated-token-account.json` | `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL` | 关联Token账户程序 |
| `associated-token-program.json` | `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL` | 关联Token程序(备选接口) |
| `memo.json` | `MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr` | SPL Memo程序 - 链上备忘录 |
| `name-service.json` | `namesLPneVptA9Z5rqUDD9tMTWEJwofgaYwp8cawRkX` | SPL域名服务程序 |
| `shared-memory.json` | `shmem4EWT2sPdVGvTZCzXXRAURL9G5vpPxNwSeKhHUL` | SPL共享内存程序 |
| `token-lending.json` | `LendZqTs7gn5CTSJU1jWKhKuVpjFgom45nnwPb2AMTi` | SPL Token借贷程序 |
| `token-swap.json` | `SwaPpA9LAaLfeLi3a68M4DjnLqgtticKg6CnyNwgAC8` | SPL Token交换程序 |

### 🏛️ **原生程序** - `/native/` 目录

这些是Solana区块链的核心系统程序，提供基础的区块链功能。

| 文件名 | 程序地址 | 描述 |
|--------|----------|------|
| `system.json` | `11111111111111111111111111111112` | 系统程序 - 账户创建和基础操作 |
| `config.json` | `Config1111111111111111111111111111111111111` | 配置程序 - 通用配置管理 |
| `stake.json` | `Stake11111111111111111111111111111111111111` | 质押程序 - PoS共识机制 |
| `vote.json` | `Vote111111111111111111111111111111111111111` | 投票程序 - 验证者投票机制 |
| `compute-budget.json` | `ComputeBudget111111111111111111111111111111` | 计算预算程序 - 交易费用管理 |

## 🔧 元数据标准

所有IDL文件都包含标准化的元数据信息：

```json
{
  "metadata": {
    "address": "程序的链上地址",
    "origin": "solana-program-library | solana-native | shank",
    "description": "程序功能的详细描述",
    "repository": "源代码仓库URL",
    "documentation": "文档URL",
    "category": "spl | native",
    "instruction_selector_type": "index | single-instruction"
  }
}
```

## 🎯 使用方式

### 使用Solores生成Rust客户端库

#### 生成单个SPL程序接口：
```bash
$SOLORES_BIN idls/spl/token.json -o spl_token_interface --generate-parser
```

#### 批量生成SPL程序接口：
```bash
$SOLORES_BIN idls/spl/ --batch --generate-parser \
  --batch-output-dir ./spl_interfaces/ \
  --workspace --workspace-name solana_spl_interfaces
```

#### 批量生成Native程序接口：
```bash
$SOLORES_BIN idls/native/ --batch --generate-parser \
  --batch-output-dir ./native_interfaces/ \
  --workspace --workspace-name solana_native_interfaces
```

### 程序类型说明

- **SPL程序**: 使用NonAnchor模板系统，通常有1字节或无discriminator
- **Native程序**: 使用NonAnchor模板系统，原生Solana程序格式
- **特殊情况**: memo、shared-memory、compute-budget是单指令程序

## ⚠️ 特殊注意事项

1. **单指令程序**: memo、shared-memory、compute-budget等程序的instructions数组为空，使用`"instruction_selector_type": "single-instruction"`
2. **重复接口**: associated-token-account和associated-token-program是同一程序的不同接口版本
3. **地址验证**: 所有程序地址都是Solana官方确认的标准地址

## 📖 相关资源

- [SPL程序文档](https://spl.solana.com/)
- [Solana程序文档](https://docs.solana.com/developing/runtime-facilities/programs)
- [Solores项目](https://github.com/solana-labs/solores)

## 🔄 更新历史

- **2025-08-12**: 添加了完整的程序地址和标准化元数据信息
- **初始版本**: 基础IDL文件集合