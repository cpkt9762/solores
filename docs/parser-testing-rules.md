# Solores Parser测试规则文档

## 📋 概述

Solores项目为Solana程序生成Parser模块，用于解析账户数据和指令数据。根据IDL合约类型的不同，采用两套不同的测试规则：

- **Anchor模式**: 基于8字节SHA256 discriminator的识别机制
- **非Anchor模式**: 基于数据长度的识别机制

本文档详细说明这两种模式的测试规则、验证标准和最佳实践。

## 🎯 测试架构

### 识别机制判断

```rust
fn is_anchor_contract(&self) -> bool {
    // 检查是否有指令包含discriminator字段
    if let Some(instructions) = &self.instructions {
        return instructions.iter().any(|ix| ix.has_discriminator());
    }
    false
}
```

### 测试生成器统一接口

所有Parser测试都遵循统一的生成器接口：

```rust
pub trait IdlCodegenModule {
    fn generate_tests(&self) -> TokenStream;
}
```

## 🔧 Anchor模式测试规则

### 1. 识别特征
- IDL中指令包含`discriminator`字段
- 使用8字节SHA256哈希作为discriminator
- 账户结构体包含`discriminator: [u8; 8]`字段

### 2. 测试数据生成策略

#### 使用结构体构造器（推荐）
```rust
let expected_first_field = 42u64;
let test_account = GlobalConfig {
    epoch: expected_first_field,
    ..Default::default()
};
let test_data = test_account.try_to_vec().unwrap();
```

#### Discriminator计算
```rust
let discm = <[u8; 8]>::try_from(
    &Sha256::digest(format!("account:{}", account_name.to_pascal_case()).as_bytes()).as_slice()[..8],
).unwrap();
```

### 3. 测试验证内容

#### 一致性测试 (`test_{account}_consistency`)
```rust
#[test]
fn test_global_config_consistency() {
    let expected_first_field = 42u64;
    let test_account = GlobalConfig {
        epoch: expected_first_field,
        ..Default::default()
    };
    let test_data = test_account.try_to_vec().unwrap();
    
    match try_unpack_account(&test_data) {
        Ok(RaydiumLaunchpadAccount::GlobalConfig(account)) => {
            // ✅ 第一个字段数据一致性验证
            assert_eq!(
                account.epoch, expected_first_field,
                "First field value should match expected value"
            );
            
            // ✅ Discriminator字段验证
            assert_eq!(
                account.discriminator,
                [149u8, 8u8, 156u8, 202u8, 160u8, 252u8, 176u8, 217u8],
                "Discriminator field should match expected value"
            );
        }
        Ok(_) => panic!("Discriminator matched wrong account type"),
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                !error_msg.contains("Unknown account discriminator"),
                "Should recognize discriminator for {}",
                stringify!(GlobalConfig)
            );
        }
    }
}
```

#### 识别测试 (`test_discriminator_recognition`)
```rust
#[test]
fn test_discriminator_recognition() {
    let test_account = GlobalConfig::default();
    let test_data = test_account.try_to_vec().unwrap();
    
    // ✅ 验证最小长度 (8字节discriminator + 账户数据)
    assert!(
        test_data.len() >= 8,
        "Test data should have at least 8 bytes for discriminator"
    );
    
    // ✅ 测试discriminator识别能力
    let result = try_unpack_account(&test_data);
    match result {
        Ok(_) => {
            // 成功 - discriminator被识别且账户被解析
        },
        Err(e) => {
            // 允许解析错误，但不允许discriminator识别错误
            let error_msg = e.to_string();
            assert!(
                !error_msg.contains("Unknown account discriminator"), 
                "Should recognize discriminator for account {}", 
                stringify!(GlobalConfig)
            );
        }
    }
}
```

### 4. 边界条件测试

#### 无效Discriminator测试
```rust
#[test]
fn test_invalid_discriminator() {
    let invalid_data = vec![255u8; 40]; // 无效discriminator + 数据
    let result = try_unpack_account(&invalid_data);
    assert!(result.is_err(), "Should fail with invalid discriminator");
}
```

#### 数据长度不足测试
```rust
#[test]
fn test_insufficient_data() {
    let short_data = vec![1u8; 4]; // 少于8字节
    let result = try_unpack_account(&short_data);
    assert!(result.is_err(), "Should fail with insufficient data");
}
```

## ⚡ 非Anchor模式测试规则

### 1. 识别特征
- IDL中指令不包含`discriminator`字段
- 使用数据长度进行账户类型识别
- 账户结构体包含`LEN`常量

### 2. 测试数据生成策略

#### 使用Default构造器
```rust
let test_account = AccountStruct::default();
let test_data = test_account.try_to_vec().unwrap();
```

#### 长度常量定义
```rust
const ACCOUNT_STRUCT_LEN: usize = std::mem::size_of::<AccountStruct>();
```

### 3. 测试验证内容

#### 一致性测试
```rust
#[test]
fn test_account_struct_consistency() {
    let test_account = AccountStruct::default();
    let test_data = test_account.try_to_vec().unwrap();
    
    match try_unpack_account(&test_data) {
        Ok(ProgramAccount::AccountStruct(account)) => {
            // ✅ 成功解析账户
            // 注意：非Anchor模式通常不验证具体字段值
        }
        Ok(_) => panic!("Length matched wrong account type"),
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                !error_msg.contains("Invalid Account data length"),
                "Should recognize account length for {}", 
                stringify!(AccountStruct)
            );
        }
    }
}
```

#### 长度识别测试
```rust
#[test]
fn test_length_recognition() {
    let test_account = AccountStruct::default();
    let test_data = test_account.try_to_vec().unwrap();
    
    // ✅ 验证账户数据大小匹配预期长度
    assert_eq!(
        test_data.len(), 
        AccountStruct::LEN, 
        "Test data should match account struct size"
    );
    
    // ✅ 测试基于长度的识别能力
    let result = try_unpack_account(&test_data);
    match result {
        Ok(_) => {
            // 成功 - 长度被识别且账户被解析
        },
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                !error_msg.contains("Invalid Account data length"), 
                "Should recognize account length for {}", 
                stringify!(AccountStruct)
            );
        }
    }
}
```

## 📊 测试覆盖对比表

| 测试类型 | Anchor模式 | 非Anchor模式 | 验证重点 |
|---------|-----------|-------------|---------|
| **一致性测试** | ✅ 字段值 + Discriminator | ✅ 基本解析 | 序列化/反序列化一致性 |
| **识别测试** | ✅ 8字节Discriminator | ✅ 数据长度 | 账户类型识别能力 |
| **字段验证** | ✅ 第一个字段对比 | ❌ 通常跳过 | 数据完整性 |
| **边界测试** | ✅ 无效Discriminator | ✅ 无效长度 | 错误处理能力 |
| **长度测试** | ✅ 最小8字节 | ✅ 精确匹配 | 数据格式验证 |

## 🔍 测试质量标准

### 1. 测试命名规范
- 一致性测试: `test_{account_name}_consistency`
- 识别测试: `test_discriminator_recognition` / `test_length_recognition`
- 边界测试: `test_invalid_discriminator` / `test_invalid_length`
- 数据测试: `test_insufficient_data`

### 2. 断言消息标准化

#### Anchor模式断言消息
```rust
assert_eq!(account.field, expected_value, "First field value should match expected value");
assert_eq!(account.discriminator, expected_discriminator, "Discriminator field should match expected value");
assert!(!error_msg.contains("Unknown account discriminator"), "Should recognize discriminator for {}", stringify!(AccountName));
```

#### 非Anchor模式断言消息
```rust
assert_eq!(test_data.len(), AccountStruct::LEN, "Test data should match account struct size");
assert!(!error_msg.contains("Invalid Account data length"), "Should recognize account length for {}", stringify!(AccountName));
```

### 3. 错误处理要求
- ✅ 必须测试识别错误情况
- ✅ 必须验证错误消息内容
- ✅ 允许解析错误但不允许识别错误
- ✅ 使用`stringify!`宏生成类型名称

## 🛠️ 生成器实现细节

### 1. 测试数据生成逻辑

#### Anchor模式类型匹配
```rust
let (test_value, test_value_bytes) = match &first_field.r#type {
    TypedefFieldType::PrimitiveOrPubkey(type_str) => {
        match type_str.as_str() {
            "u64" => (quote! { 42u64 }, quote! { 42u64.to_le_bytes().to_vec() }),
            "u32" => (quote! { 42u32 }, quote! { 42u32.to_le_bytes().to_vec() }),
            "u8" => (quote! { 42u8 }, quote! { vec![42u8] }),
            "publicKey" | "Pubkey" => (
                quote! { solana_program::pubkey::Pubkey::new_from_array([1u8; 32]) },
                quote! { [1u8; 32].to_vec() }
            ),
            "string" => (
                quote! { "test".to_string() },
                quote! { {
                    let s = "test";
                    let mut bytes = (s.len() as u32).to_le_bytes().to_vec();
                    bytes.extend_from_slice(s.as_bytes());
                    bytes
                } }
            ),
            "bool" => (quote! { true }, quote! { vec![1u8] }),
            _ => (quote! { Default::default() }, quote! { vec![0u8; 32] })
        }
    },
    TypedefFieldType::array(array_type) => {
        let array_size = proc_macro2::Literal::usize_unsuffixed(array_type.1 as usize);
        match &*array_type.0 {
            TypedefFieldType::PrimitiveOrPubkey(elem_type) => {
                match elem_type.as_str() {
                    "u64" => (
                        quote! { [42u64; #array_size] },
                        quote! { [42u64; #array_size].iter().flat_map(|&x| x.to_le_bytes()).collect::<Vec<u8>>() }
                    ),
                    "u8" => (
                        quote! { [42u8; #array_size] },
                        quote! { [42u8; #array_size].to_vec() }
                    ),
                    _ => (quote! { Default::default() }, quote! { vec![0u8; 32] })
                }
            },
            _ => (quote! { Default::default() }, quote! { vec![0u8; 32] })
        }
    },
    _ => (quote! { Default::default() }, quote! { vec![0u8; 32] })
};
```

### 2. 结构体定义查找

#### 类型定义查找逻辑
```rust
let struct_def = if acc.0.r#type.is_some() {
    acc.0.r#type.as_ref()
} else {
    // 尝试从types数组中查找匹配的结构体定义
    self.idl.types.as_deref().unwrap_or(&[]).iter()
        .find(|t| t.name == acc.0.name)
        .and_then(|t| t.r#type.as_ref())
};
```

## 🚨 故障排除指南

### 1. 常见编译错误

#### 变量作用域错误
```
error[E0425]: cannot find value `is_anchor` in this scope
```
**解决方案**: 移除条件检查，直接验证discriminator字段

#### 数组Default实现错误
```
error[E0277]: the trait `Default` is not implemented for `[u8; 256]`
```
**解决方案**: 使用显式数组初始化
```rust
padding: [0u8; 256]  // 而不是 Default::default()
```

#### Panic字符串生成错误
```
error: expected string literal, found `ix.name`
```
**解决方案**: 使用`stringify!`宏
```rust
panic!("Should successfully parse instruction {}", stringify!(InstructionName));
```

### 2. 测试失败诊断

#### Discriminator不匹配
```
assertion failed: discriminator field should match expected value
```
**检查项**:
- SHA256计算是否正确
- 账户名称PascalCase转换
- Default实现是否包含正确discriminator

#### 字段值不匹配
```
assertion failed: first field value should match expected value
```
**检查项**:
- 结构体构造器语法
- 字段名snake_case转换
- 类型定义查找逻辑

#### 长度不匹配（非Anchor模式）
```
assertion failed: test data should match account struct size
```
**检查项**:
- `std::mem::size_of`计算
- 结构体字段对齐
- Borsh序列化长度

### 3. 最佳实践建议

#### 测试维护
- 🔄 定期运行完整测试套件
- 📝 更新测试时保持文档同步
- 🧪 为新增字段类型添加测试覆盖
- 🔍 使用`cargo test parsers::accounts::tests -- --nocapture`查看详细输出

#### 代码质量
- ✅ 确保所有测试都有意义的断言消息
- ✅ 使用统一的错误处理模式  
- ✅ 避免硬编码魔法数字，使用常量
- ✅ 保持测试代码与生产代码的一致性

## 📈 未来改进方向

### 1. 测试覆盖增强
- [ ] 添加更多数据类型的测试用例
- [ ] 支持嵌套结构体的字段验证
- [ ] 增加性能基准测试

### 2. 错误处理优化
- [ ] 提供更详细的错误诊断信息
- [ ] 添加测试数据生成失败的回退机制
- [ ] 实现测试结果的结构化报告

### 3. 工具集成
- [ ] 集成到CI/CD流水线
- [ ] 提供测试覆盖率报告
- [ ] 支持并行测试执行

---

## 📚 相关文档

- [CLAUDE.md](../CLAUDE.md) - 项目框架和架构文档
- [README.md](../README.md) - 项目使用说明
- [examples/](../examples/) - 各种IDL格式的示例项目

---

*最后更新: 2025-08-05*
*版本: v0.8.0*