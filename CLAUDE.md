# CLAUDE.md - Solores 项目开发指南

## 🎯 项目概述

Solores 是一个 Solana IDL 到 Rust 客户端/CPI 接口生成器，支持从多种 IDL 格式自动生成高质量的 Rust 代码。

### 核心特点
- **自动化代码生成**: 从 IDL 文件自动生成完整的 Rust 接口代码
- **多格式支持**: 支持 Anchor、Shank、Bincode、Native 等多种 IDL 格式
- **100%编译成功率**: 经过 20+ 主要协议验证，零错误零警告

## 🏗️ 架构概述

### 二元模板架构
```
IDL文件 → 格式检测 → 模板工厂 → 代码生成 → Rust项目
```

**核心模块**:
- `idl_format/`: Anchor vs NonAnchor 二元解析架构
- `templates/`: 统一模板生成系统
- `templates/common/`: 共享组件库(导入管理、文档生成等)

### IDL 格式支持状态

| 合约类型 | 支持状态 | discriminator | 特殊处理 |
|----------|----------|---------------|----------|
| **Anchor** | ✅ 完全支持 | 8字节 | discriminator长度检测 |
| **NonAnchor** | ✅ 完全支持 | 1字节或无 | 长度识别/字段分析 |
| **SPL Programs** | ✅ 完全支持 | 变长 | 自动格式检测 |
| **Native Programs** | ✅ 完全支持 | 4字节索引 | 系统变量替换 |

## 🔄 代码生成流程

### 完整生成流程
1. **IDL解析** → AnchorIdl/NonAnchorIdl 结构体
2. **格式检测** → TemplateFactory::detect_contract_mode()
3. **字段分析** → FieldAllocationAnalyzer 优化类型分配
4. **模板生成** → 多文件架构生成
5. **代码优化** → 智能导入管理、未使用变量处理

### 生成的模块结构
- **instructions/** - 指令模块(IxData + Keys)
- **accounts/** - 账户结构体
- **types/** - 自定义类型(支持HashMap等复杂类型)
- **events/** - 事件结构体  
- **errors.rs** - 错误枚举
- **parsers/** - 指令/账户解析器(可选)

## 🛠️ 构建和使用

### 环境变量配置
```bash
export SOLORES_BIN="/path/to/solores/scripts/solores-wrapper.py"
```

### 基础使用
```bash
# 基本生成
$SOLORES_BIN path/to/idl.json

# 生成解析器(推荐)
$SOLORES_BIN path/to/idl.json --generate-parser

# 批量处理
$SOLORES_BIN idls/ --batch --generate-parser --batch-output-dir ./output/
```

## 🔧 开发工具生态

### UV智能包装器 (`scripts/solores-wrapper.py`)
- 自动构建检测和重新编译
- 彩色进度显示和错误处理
- Raydium接口自动修复集成

### 接口修复工具 (`scripts/fix_raydium_interface.py`)
- 专门修复Raydium 17/18账户动态场景
- Option<Pubkey>字段修复
- 动态AccountMeta生成

### 验证工具 (`scripts/validate_module_functions.py`)
- 跨模块函数一致性验证
- 批量项目验证支持
- 详细报告和统计

## 🎯 技术突破

### HashMap类型支持
- 完整支持嵌套HashMap类型: `{"hashMap": ["string", "string"]}`
- 自动类型转换和默认值生成
- 跨模板系统的一致性处理

### 系统程序完善
- SystemError完整枚举(9个错误类型)
- NonceState/NonceData完整定义
- 系统变量自动替换: `$(SysVarRentPubkey)` → `rent`

### 智能代码优化
- 未使用变量智能重命名(下划线前缀)
- 无效类型名称自动清理: `'&'astr'` → `Refastr`
- format!宏字符串插值修复

## 📁 标准路径规范

### 路径约定
- **IDL输入**: `idls/{name}.json`
- **测试输出**: `test_output/{purpose}_{name}/`
- **生产输出**: `batch_output/{crate_name}/`

### 命令格式
```bash
# 标准格式
RUST_LOG=info $SOLORES_BIN idls/{FILE}.json -o test_output/{PURPOSE} --generate-parser

# 批量生成
$SOLORES_BIN idls/ --batch --generate-parser --batch-output-dir ./batch_output/
```

## 🚫 生成器问题处理原则

**核心原则**: 必须从生成器修复问题，绝对禁止手动修改生成的代码。

**问题修复流程**:
1. 在生成器代码中定位根本原因
2. 在模板或解析器中实施修复  
3. 验证修复的完整性和一致性
4. 重新生成验证解决方案

## 📊 验证状态

### 成功验证的协议 (20+)
**DEX/AMM**: Raydium, Phoenix, OpenBook, Whirlpool, Saros, Lifinity  
**DeFi**: Squads, Meteora, Stable Swap  
**SPL**: Token, Token-2022  
**Native**: System Program, Compute Budget  
**其他**: Pump.fun, Moonshot, Boop, Serum

**批量生成成功率**: 100% (零错误, 零警告)