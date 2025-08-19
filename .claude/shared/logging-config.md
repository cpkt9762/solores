# 文件导向日志策略

## 核心要求

### MANDATORY配置
- console_output: false（禁用控制台）
- 重定向到文件
- 显示完整文件路径
- 自动清理旧日志

### 标准配置模式
```rust
// 测试日志配置
fn setup_test_logging() {
    std::fs::create_dir_all("tests/logs").ok();
    
    // 清理旧日志
    if Path::new("tests/logs/test.log").exists() {
        let _ = std::fs::remove_file("tests/logs/test.log");
    }
    
    let file_appender = tracing_appender::rolling::never("tests/logs", "test.log");
    
    tracing_subscriber::registry()
        .with(fmt::layer()
            .with_writer(file_appender)
            .with_ansi(false)
            .with_target(true)
            .with_file(true)           // 显示文件名
            .with_line_number(true))   // 显示行号
        .try_init();
        
    eprintln!("📝 日志输出到: {}/tests/logs/test.log", env::current_dir().unwrap().display());
}
```

## 日志文件路径
- 测试日志: tests/logs/test.log
- 应用日志: logs/app.log
- 开发日志: logs/dev.log

## 违规检测

| 违规信号 | 纠正动作 |
|---------|---------|
| "console_output: true" | 改为false |
| "println!" 非路径提示 | 改为tracing日志 |
| 缺少路径显示 | 添加路径提示 |

## instrument使用要求

### 函数级监控
```rust
#[tracing::instrument(level = "info", skip(self))]
async fn process(&self, params: &Params) -> Result<Output> {
    // 自动创建span，支持文件导向日志
}
```

### 异步监控
```rust
let result = async_operation()
    .instrument(tracing::info_span!("operation"))
    .await?;
```

### 监控约束
- 所有公共API函数：#[instrument]
- 复杂异步操作：手动span
- 关键业务逻辑：debug/info级别监控
- 跳过大型数据：#[instrument(skip(large_data))]