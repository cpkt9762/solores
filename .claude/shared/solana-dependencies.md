# Solana 依赖配置规范

## 最新版本标准 (2.4.x)

### 核心依赖版本
```toml
# 链上程序核心依赖
solana-pubkey = { version = "2.4.0", features = ["borsh", "curve25519", "serde"] }
solana-account-info = "2.4.0"
solana-instruction = "2.3.0"
solana-program-error = "2.3.0"
solana-cpi = "2.3.0"
solana-entrypoint = "2.3.0"

# SDK和应用层依赖
solana-sdk = "2.1.1"
solana-client = "2.1.1"
solana-rpc-client = "2.1.1"
solana-account-decoder = "2.1.1"
solana-transaction-status = "2.1.1"
```

## 分层依赖策略

### 链上程序层 (contract/)
```toml
[dependencies]
solana-pubkey = { version = "2.4.0", default-features = false, features = ["borsh"] }
solana-account-info = { version = "2.4.0", default-features = false }
solana-instruction = { version = "2.3.0", default-features = false }
error-stack = "0.4"
thiserror = "1.0"  # IDL兼容
borsh = "1.5"
```

### 共享库层 (shared-libs/)
```toml
[dependencies]
solana-pubkey = "2.4.0"
solana-instruction = "2.3.0"
error-stack = "0.4"
# 基于功能选择性添加其他依赖
```

### SDK层 (dex-sdk/, contract-sdk/)
```toml
[dependencies]
solana-pubkey = "2.4.0"
solana-instruction = "2.3.0"
solana-decode-error = "2.3.0"
error-stack = "0.4"
```

### 应用层 (main-app/, *-engine/)
```toml
[dependencies]
solana-sdk = "2.1.1"
solana-client = "2.1.1"
solana-rpc-client = "2.1.1"
error-stack = "0.4"
tokio = { version = "1.41", features = ["full"] }
```

## 特性配置指南

### 最小化特性集（链上程序）
```toml
solana-pubkey = { 
    version = "2.4.0", 
    default-features = false,
    features = ["borsh"]  # 仅必需特性
}
```

### 标准特性集（SDK/共享库）
```toml
solana-pubkey = { 
    version = "2.4.0",
    features = ["borsh", "serde"]
}
```

### 完整特性集（应用层）
```toml
solana-pubkey = { 
    version = "2.4.0",
    features = ["borsh", "curve25519", "serde", "std"]
}
```

## 版本兼容性矩阵

| 模块层级 | solana-* crates | 版本范围 | 特性策略 |
|---------|----------------|---------|---------|
| 链上程序 | pubkey, account-info, instruction | 2.3.0-2.4.0 | 最小化 |
| 共享库 | pubkey, instruction | 2.3.0-2.4.0 | 选择性 |
| SDK | pubkey, instruction, decode-error | 2.3.0-2.4.0 | 标准 |
| 应用 | sdk, client, rpc-client | 2.1.1 | 完整 |

## IDL接口库特殊要求

### Solores生成的接口库
```toml
[dependencies]
# IDL接口库使用最新版本以支持所有特性
solana-pubkey = { version = "2.4.0", features = ["borsh", "curve25519", "serde"] }
solana-instruction = "2.3.0"
solana-account-info = "2.4.0"
thiserror = "1.0"  # 必需，用于错误类型生成
borsh = "1.5"      # 必需，用于序列化
```

## 依赖检查命令

```bash
# 检查版本一致性
cargo tree --workspace --duplicates

# 验证特性配置
cargo tree -f "{p} {f}" | grep solana

# 检查过时依赖
cargo outdated --workspace

# 安全审计
cargo audit
```

## 📦 版本查询和管理策略

### 最新版本优先原则
- **遇到依赖版本问题时，优先使用最新稳定版本**
- **自动化版本查询**：使用以下工具获取版本信息
- **立即更新策略**：发现版本问题时立即更新并验证

### 版本信息查询工具

#### docs.rs 查询
- **用途**：获取最新的版本信息
- **访问**：https://docs.rs/crate-name 查看最新版本
- **适用**：所有 Rust crates 的最新版本查询

#### rust-docs MCP 查询
- **用途**：其他依赖相关信息查询
- **工具**：
  - `mcp__rust-docs__get_dependencies` - 分析依赖关系
  - `mcp__rust-docs__structure` - 了解 crate 结构
  - `mcp__rust-docs__list_crate_versions` - 查看可用版本
- **适用**：依赖分析和结构理解

### 版本更新流程
1. **问题识别**：发现依赖版本冲突或过时
2. **版本查询**：通过 docs.rs 获取最新版本
3. **兼容性检查**：使用 rust-docs MCP 分析依赖影响
4. **立即更新**：更新 Cargo.toml 版本约束
5. **编译验证**：确保更新后正常编译和运行

## 迁移指南

### 从 2.1.x 升级到 2.4.x
1. 更新 `solana-pubkey` 到 2.4.0
2. 更新 `solana-instruction` 到 2.3.0
3. 添加必要的特性标志
4. 运行测试验证兼容性

### 特性迁移
- `no-entrypoint` → 使用条件编译
- `program` → 拆分为细粒度依赖
- `full` → 明确指定需要的特性

## 常见问题

### Q: 为什么应用层使用 2.1.1 而链上程序使用 2.4.0？
A: 应用层需要稳定的 RPC 客户端，2.1.1 经过充分测试。链上程序使用最新版本获得更好的性能和特性。

### Q: 如何处理版本冲突？
A: 使用 workspace 统一管理版本，或使用特性标志隔离不同版本。

### Q: 是否可以混用不同版本？
A: 可以，但需要注意 API 兼容性。建议在同一层级内保持版本一致。