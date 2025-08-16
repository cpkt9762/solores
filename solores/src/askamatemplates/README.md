# Askama 模板系统

这个目录包含 Solores 的 Askama 外部模板文件，用于生成 Solana 程序接口代码。

## 📁 目录结构

- `anchor/` - Anchor 合约模板 (8字节 discriminator)
- `non_anchor/` - 非 Anchor 合约模板 (1字节 discriminator)
- `common/` - 共用模板组件

## 🎯 模板文件说明

### Anchor 模板
- `lib.rs.askama` - 主库文件模板
- `instructions.rs.askama` - 指令模块模板
- `accounts.rs.askama` - 账户模块模板
- `events.rs.askama` - 事件模块模板
- `types.rs.askama` - 类型模块模板
- `parsers.rs.askama` - 解析器模块模板
- `mod.rs.askama` - 模块导出模板
- `cargo.toml.askama` - Cargo配置模板

### 公共组件
- `serde_helpers.askama` - Serde 序列化辅助函数
- `imports.askama` - 公共导入语句
- `test_utils.askama` - 测试工具函数

## 🔧 模板语法

使用 Jinja2 语法：
- `{{ variable }}` - 变量插值
- `{% if condition %}` - 条件判断
- `{% for item in list %}` - 循环遍历
- `{% include "common/file.askama" %}` - 包含其他模板

## 🎨 自定义过滤器

- `{{ name|snake_case }}` - 转换为 snake_case
- `{{ name|pascal_case }}` - 转换为 PascalCase
- `{{ name|rust_keywords }}` - 处理 Rust 关键字冲突

## 🚀 使用方式

通过环境变量启用 Askama 模板系统：
```bash
export SOLORES_USE_ASKAMA=true
$SOLORES_BIN idls/example.json -o output_dir --generate-parser
```