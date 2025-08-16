# CLAUDE.md - Solores 项目指南

## 🎯 项目概述

Solores - Solana IDL 到 Rust 接口生成器。支持 Anchor/NonAnchor/SPL/Native 全格式，100%编译成功率。

### 架构
```
IDL → 格式检测 → 模板生成 → Rust代码
```
- 二元架构: Anchor vs NonAnchor
- 模板系统: `templates/anchor/` 和 `templates/non_anchor/`
- 自动后处理: Raydium修复、未使用变量处理

## 🚀 快速开始

### 使用Makefile (推荐)
```bash
make test              # 测试关键IDL
make test-one IDL=xxx  # 测试单个IDL
make batch             # 批量生成所有IDL
make clean             # 清理测试文件
make help              # 查看所有命令
```

### 直接使用
```bash
export SOLORES_BIN="./scripts/solores-wrapper.py"
$SOLORES_BIN idls/xxx.json -o output_dir --generate-parser
```

## 📁 目录结构规范

### 输出目录约定
```
test_output/           # 临时测试（.gitignore）
├── {feature}_test/    # 功能测试
├── serde_verify_*/    # serde验证
├── test_makefile/     # Makefile测试
└── verify_*/          # 其他验证

batch_output/          # 批量输出（.gitignore）
└── sol_xxx_interface/ # 生成的接口

生产输出：使用绝对路径指定目标位置
```

### 命名规范
- 测试: `test_output/{功能}_{idl名}/`
- 验证: `test_output/verify_{特性}/`
- 批量: `batch_output/` 或指定路径
- **避免**: 随意命名、深层嵌套、混合用途

## 🎯 核心功能

### 支持的特性
- ✅ **Serde支持**: `--features serde` 生成JSON序列化
- ✅ **解析器生成**: `--generate-parser` 生成指令/账户解析器
- ✅ **批量处理**: `--batch` 批量生成所有IDL
- ✅ **Workspace**: `--workspace` 生成workspace项目

### 关键技术
- **HashMap支持**: 嵌套HashMap类型完整支持
- **动态账户**: Raydium 17/18账户自动修复
- **智能优化**: 未使用变量自动处理
- **类型系统**: Option/Vec/Array完整支持

### 生成的接口
```rust
// IxData: 指令数据结构
pub struct XxxIxData { ... }
impl XxxIxData {
    pub fn new(...) -> Self
    pub fn from_bytes(&[u8]) -> Result<Self>
    pub fn try_to_vec() -> Result<Vec<u8>>
    #[cfg(feature = "serde")]
    pub fn to_json() -> String
}

// Keys: 账户结构
pub struct XxxKeys { ... }
impl XxxKeys {
    pub fn to_vec() -> Vec<Pubkey>
    #[cfg(feature = "serde")]
    pub fn to_json() -> String
}
```

## 🚫 开发原则

### 生成器优先
**绝对禁止手动修改生成的代码**。所有问题必须在生成器层面解决。

### 修复流程
1. 定位生成器中的问题
2. 修改模板或解析器
3. 重新生成并验证

### 代码规范
- 使用Makefile进行测试
- 遵循目录结构规范
- 清理临时测试文件
- 不提交test_output/

## ✅ 验证状态

**已验证协议**: 20+ 主流协议（Raydium、Phoenix、OpenBook、Whirlpool、SPL Token等）
**成功率**: 100%编译成功，零错误零警告
**Serde支持**: 全部IDL格式完整支持