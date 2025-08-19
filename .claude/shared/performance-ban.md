# 性能测试绝对禁令

## 禁止范围
- 执行时间统计
- 性能基准测试  
- 吞吐量测量
- 延迟测试
- 内存使用统计

## 检测触发器

| 违规信号 | 纠正动作 |
|---------|---------|
| "std::time::Instant" | 立即删除 |
| "#[bench]" | 立即删除 |
| "criterion" | 立即删除 |
| "assert!(duration <" | 立即删除 |
| "QPS/TPS" | 立即删除 |

## 项目焦点
专注功能正确性和数据准确性

## 唯一例外
仅允许调试期间临时性能测试，完成后必须移除

## 正确测试方式
```rust
// ✅ 功能测试
#[rstest]
fn test_functionality() {
    let result = process_data(&data);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), expected);
}

// ❌ 性能测试
// let start = Instant::now();
// assert!(start.elapsed() < Duration::from_millis(100));
```