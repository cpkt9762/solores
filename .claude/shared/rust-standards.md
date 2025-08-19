# Rust编程标准规范

## 核心原则

### MANDATORY要求
- 零错误容忍：禁止unwrap/expect
- error-stack优先：现代错误处理
- rstest强制：所有测试使用rstest
- 真实数据：测试使用真实数据

### 参数命名约定（项目特定）
```rust
// 定义时加下划线，使用时去掉
pub fn create_transaction(_account: Pubkey, _amount: u64) -> Result<Transaction> {
    Transaction::new(account, amount) // 使用时去掉_
}
```

### 结构体注释要求
```rust
/// 数据结构
#[derive(Debug, Clone)]
pub struct Data {
    /// 账户地址
    pub account: Pubkey,
    /// 金额 (lamports)
    pub amount: u64,
}
```

## 测试规范

### rstest强制要求
```rust
#[rstest]
#[case("真实地址", 1000000)]
fn test_function(#[case] addr: &str, #[case] amount: u64) {
    // 测试逻辑
}
```

### 禁止模式
- #[test]属性使用
- 虚构测试数据
- 性能相关断言

## 性能优化原则

### 零拷贝优先
- 优先使用引用和借用
- 使用 `&[u8]` 而非 `Vec<u8>`
- 使用 `&str` 而非 `String`
- 考虑使用 `Cow<'_, T>` 处理可选拷贝

### 避免 clone
- 仅在必要时使用 clone
- 优先重构代码逻辑避免 clone
- 使用 Arc/Rc 共享大数据
- 注释说明每个 clone 的必要性

### unsafe 使用规范
- 允许在性能关键路径使用 unsafe
- 必须包含 `// SAFETY:` 注释说明
- 优先使用安全抽象封装 unsafe 代码
- 所有 unsafe 代码需要额外测试覆盖
- UnsafeCell 允许用于内部可变性需求
- UnsafeCell 必须封装在安全的抽象中
- 需要文档说明为什么需要内部可变性

### 示例模式
```rust
// ✅ 零拷贝处理
pub fn process_data(data: &[u8]) -> Result<&str> {
    std::str::from_utf8(data)
        .change_context(Error::InvalidUtf8)
}

// ✅ 必要的 clone 带注释
let cloned = original.clone(); // 必需：异步任务需要独立所有权

// ✅ 安全的 unsafe 使用
// SAFETY: 已验证 index < len，不会越界
unsafe { data.get_unchecked(index) }

// ✅ UnsafeCell 用于内部可变性
use std::cell::UnsafeCell;
/// 缓存实现，需要内部可变性以支持 &self 方法中的更新
struct Cache {
    // SAFETY: 单线程访问，手动保证借用规则
    data: UnsafeCell<HashMap<String, Value>>,
}
```

## 依赖配置

详见 [@solana-dependencies.md](./solana-dependencies.md)

## 检测触发器

| 违规信号 | 纠正动作 |
|---------|---------|
| ".unwrap()" | 改为?操作符 |
| "#[test]" | 改为#[rstest] |
| 缺少字段注释 | 添加///注释 |
| 虚构测试数据 | 使用真实数据 |
| "无注释的clone()" | 添加必要性说明 |
| "unsafe无SAFETY注释" | 添加安全性说明 |
| "Vec<u8>参数传递" | 考虑改为&[u8] |
| "String参数传递" | 考虑改为&str |
| "UnsafeCell无封装" | 必须提供安全抽象 |
| "UnsafeCell无文档" | 添加内部可变性说明 |