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

## 🔥 强制规范引用体系（不可违反）

### 全局Rust开发规范
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
| 违规行为 | 强制纠正 |
|---------|---------|
| "测试失败但有编译错误" | 立即停止测试 - 先修复编译 |
| "跳过cargo check直接测试" | 立即停止 - 强制编译检查 |
| "删除模块解决编译错误" | 立即停止 - 修复而非删除 |

## 🤖 主代理+rust-expert协作约束

### 任务委托强制规则
- **MANDATORY**: 所有Rust代码编写/修改必须委托给rust-expert
- **MANDATORY**: 所有复杂Rust任务必须通过rust-expert执行
- **FORBIDDEN**: 主代理直接进行复杂的Rust代码操作

### 强制委托的任务类型
- **代码编写**: 新功能实现、模块创建
- **代码修改**: Bug修复、重构、优化
- **代码生成**: 模板生成、IDL转换
- **编译修复**: 编译错误修复、依赖更新
- **架构设计**: 模块设计、接口设计

### 主代理验收职责（详细规范）

#### 编译状态验收
- **MUST**: 运行 `cargo check` 验证编译通过
- **MUST**: 确认零编译错误（严格要求）
- **ALLOWED**: 无害编译警告（unused_imports, dead_code等）
- **FORBIDDEN**: 严重警告（unused_must_use, deprecated等）
- **FAILURE**: 发现编译错误立即要求rust-expert修复

#### 文件完整性验收
- **MUST**: 使用 `LS` 验证所有声明文件存在
- **MUST**: 使用 `Read` 检查关键文件内容非空且有效
- **MUST**: 验证目录结构符合标准（7目录+4根文件）
- **MUST**: 确认文件数量与IDL复杂度匹配

#### 代码质量验收
- **MUST**: 使用 `Grep` 搜索占位符和临时代码
- **MUST**: 检查无TODO注释或unimplemented!宏
- **MUST**: 验证所有函数都有完整实现
- **MUST**: 确认无空的impl块或函数体

#### 功能验收测试
- **MUST**: 运行基础编译测试验证可用性
- **MUST**: 验证生成的接口API完整性  
- **ALLOWED**: 运行cargo test进行功能验证（编译通过后）
- **REQUIRED**: 确认功能与任务需求100%匹配

### 协作工作流程
1. **主代理**: 分析任务，委托给rust-expert
2. **rust-expert**: 执行具体的代码实现
3. **主代理**: 执行详细验收检查
4. **主代理**: 确认任务100%完成或要求rust-expert修复

### 验收失败强制处理

#### 发现问题时的响应
- **IMMEDIATE**: 立即拒绝验收，标记任务为未完成
- **REQUIRED**: 明确列出所有发现的问题
- **MANDATORY**: 要求rust-expert修复所有问题后重新提交
- **FORBIDDEN**: 接受部分完成或"基本可用"的状态

#### 警告分类处理
- **允许的无害警告**: unused_imports, dead_code, non_snake_case
- **禁止的严重警告**: unused_must_use, deprecated, missing_docs
- **处理原则**: 编译错误必须修复，严重警告必须修复，无害警告可接受

### 验证检查权限
- **ALLOWED**: 主代理运行cargo check/cargo test进行验证
- **ALLOWED**: 主代理检查文件存在性和完整性
- **ALLOWED**: 主代理验证生成结果的目录结构
- **FORBIDDEN**: 主代理直接修改rust-expert生成的代码

### 委托调用标准格式
- **标准格式**: "使用rust-expert来[具体任务]"
- **示例**: "使用rust-expert来修复这些编译错误"
- **示例**: "让rust-expert来实现MiniJinja模板重构"

### SOLORES_BIN强制使用规范
- **MANDATORY**: 所有Solores测试必须使用`$SOLORES_BIN`而非直接路径
- **REQUIRED**: 确保`SOLORES_BIN="./scripts/solores-wrapper.py"`正确设置
- **FORBIDDEN**: 直接使用`./target/release/solores`或`cargo run --bin solores`
- **BENEFIT**: 包装器自动检查二进制文件新鲜度，避免使用过期版本

### 标准命令格式约束
- **正确**: `$SOLORES_BIN idls/xxx.json -o output --generate-parser`
- **正确**: `SOLORES_USE_MINIJINJA=true $SOLORES_BIN idls/xxx.json -o output`
- **错误**: `./target/release/solores idls/xxx.json -o output`
- **错误**: `cargo run --bin solores -- idls/xxx.json -o output`

### 违规检测触发器
| 违规信号 | 强制纠正 |
|---------|---------|
| "直接使用./target/release/solores" | 停止 - 改用$SOLORES_BIN |
| "cargo run --bin solores" | 停止 - 使用标准环境变量 |
| "SOLORES_BIN未设置" | 停止 - 设置环境变量 |

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