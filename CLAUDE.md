# CLAUDE.md - Solores 项目指南

## 🎯 项目概述

Solores - Solana IDL 到 Rust 接口生成器。支持 Anchor/NonAnchor/SPL/Native 全格式，100%编译成功率。

### 架构

```
IDL → 格式检测 → 模板生成 → Rust代码
```

- 二元架构: Anchor vs NonAnchor
- 模板系统: `templates/anchor/` 和 `templates/non_anchor/`
- 自动后处理: Raydium 修复、未使用变量处理

## 🚀 快速开始

### 使用 Makefile (推荐)

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

- ✅ **Serde 支持**: `--features serde` 生成 JSON 序列化
- ✅ **解析器生成**: `--generate-parser` 生成指令/账户解析器
- ✅ **批量处理**: `--batch` 批量生成所有 IDL
- ✅ **Workspace**: `--workspace` 生成 workspace 项目

### 关键技术

- **HashMap 支持**: 嵌套 HashMap 类型完整支持
- **动态账户**: Raydium 17/18 账户自动修复
- **智能优化**: 未使用变量自动处理
- **类型系统**: Option/Vec/Array 完整支持

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

## 🔥 强制规范引用体系（不可违反）

### 全局 Rust 开发规范

@~/.claude/langs/rust/README.md
@~/.claude/langs/rust/core/testing.md  
@~/.claude/langs/rust/core/error-handling.md
@~/.claude/langs/rust/frameworks/solana.md
@~/.claude/langs/rust/tools/solana-dependencies.md

### 项目特定强制约束

@.claude/shared/compilation-rules.md
@.claude/shared/performance-ban.md
@.claude/shared/global-rules.md
@.claude/shared/error-handling.md
@.claude/shared/logging-config.md
@.claude/shared/rust-standards.md
@.claude/shared/solana-dependencies.md

## ⚡ 编译优先强制工作流程

### 强制执行顺序

1. 代码修改/生成
2. **MANDATORY**: `cargo check` (必须通过)
3. **ONLY IF** 编译通过: `cargo test`
4. **ONLY IF** 测试通过: 质量验证

### 违规检测触发器

| 违规行为                    | 强制纠正                  |
| --------------------------- | ------------------------- |
| "测试失败但有编译错误"      | 立即停止测试 - 先修复编译 |
| "跳过 cargo check 直接测试" | 立即停止 - 强制编译检查   |
| "删除模块解决编译错误"      | 立即停止 - 修复而非删除   |

## 🤖 Sub-agent 委托策略

### 任务类型映射

| 任务类型 | 委托给 | 触发关键词 |
|---------|--------|-----------|
| **通用 Rust 开发** | rust-dev | "编写", "实现", "优化", "重构", "测试" |
| **Solores 项目** | solores | "IDL", "模板", "MiniJinja", "生成器" |
| **编译错误修复** | rust-fix | "编译错误", "cargo check 失败", "类型错误" |

### 委托决策流程

1. **识别任务类型**: 根据关键词和上下文判断
2. **选择合适 agent**: 按照任务映射表选择
3. **明确委托**: 使用标准格式调用
4. **验收结果**: 检查完成质量

### 委托调用标准格式

- **格式**: "使用 [agent-name] 来[具体任务]"
- **示例**: 
  - "使用 rust-dev 来实现新功能"
  - "使用 solores 来修改 MiniJinja 模板"
  - "使用 rust-fix 来修复编译错误"

### 主代理验收职责

#### 编译验证
- 运行 `cargo check` 确认编译通过
- 检查无严重警告
- 允许无害警告（unused_imports, dead_code）

#### 文件完整性
- 验证文件存在性
- 检查目录结构完整
- 确认关键文件非空

#### 功能验证
- 运行相关测试
- 验证功能符合需求
- 确认无 TODO/unimplemented

### 多 Agent 协作优势

- **专门化**: 每个 agent 专注特定领域
- **效率高**: 减少上下文加载，避免超限
- **灵活性**: 可根据任务选择最合适的 agent
- **可扩展**: 易于添加新的专门 agent

### SOLORES_BIN 强制使用规范

- **MANDATORY**: 所有 Solores 测试必须使用`$SOLORES_BIN`而非直接路径
- **REQUIRED**: 确保`SOLORES_BIN="./scripts/solores-wrapper.py"`正确设置
- **FORBIDDEN**: 直接使用`./target/release/solores`或`cargo run --bin solores`
- **BENEFIT**: 包装器自动检查二进制文件新鲜度，避免使用过期版本

### 标准命令格式约束

- **正确**: `$SOLORES_BIN idls/xxx.json -o output --generate-parser`
- **正确**: `SOLORES_USE_MINIJINJA=true $SOLORES_BIN idls/xxx.json -o output`
- **错误**: `./target/release/solores idls/xxx.json -o output`
- **错误**: `cargo run --bin solores -- idls/xxx.json -o output`

### 违规检测触发器

| 违规信号                           | 强制纠正                |
| ---------------------------------- | ----------------------- |
| "直接使用./target/release/solores" | 停止 - 改用$SOLORES_BIN |
| "cargo run --bin solores"          | 停止 - 使用标准环境变量 |
| "SOLORES_BIN 未设置"               | 停止 - 设置环境变量     |

## 🚫 开发原则

### 生成器优先

**绝对禁止手动修改生成的代码**。所有问题必须在生成器层面解决。

### 修复流程

1. 定位生成器中的问题
2. 修改模板或解析器
3. 重新生成并验证

### 代码规范

- 使用 Makefile 进行测试
- 遵循目录结构规范
- 清理临时测试文件
- 不提交 test_output/

## ✅ 验证状态

**已验证协议**: 20+ 主流协议（Raydium、Phoenix、OpenBook、Whirlpool、SPL Token 等）
**成功率**: 100%编译成功，零错误零警告
**Serde 支持**: 全部 IDL 格式完整支持
