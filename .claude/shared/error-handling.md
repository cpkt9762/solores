# 错误处理规范（error-stack优先）

## 框架优先级

```toml
error-stack = "0.4"       # 优先使用
thiserror = "1.0"         # IDL兼容  
anyhow = "1.0"           # 避免新增
```

## 核心约束

### MANDATORY要求
- 使用error-stack框架
- 禁止unwrap/expect
- 使用?操作符传播错误
- 添加具体错误上下文

### 标准模式
```rust
use error_stack::{Context, Result, ResultExt};

let value = risky_operation()
    .change_context(ErrorType::Failed)
    .attach_printable("具体上下文")?;
```

### 禁止模式
- anyhow新代码使用
- unwrap/expect调用  
- 错误忽略

## 检测触发器

| 违规信号 | 纠正动作 |
|---------|---------|
| "anyhow::Result" 新代码 | 改为error_stack::Result |
| ".unwrap()" | 改为?操作符 |
| ".expect(" | 改为错误传播 |

## 分层策略

- **链上程序**: error-stack + thiserror
- **共享库**: error-stack优先
- **应用层**: error-stack主导