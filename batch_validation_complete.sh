#!/bin/bash

if [ $# -eq 0 ]; then
    echo "用法: $0 <batch_output_directory>"
    exit 1
fi

BATCH_DIR="$1"

if [ ! -d "$BATCH_DIR" ]; then
    echo "错误: 目录 $BATCH_DIR 不存在"
    exit 1
fi

echo "🔍 开始批量验证编译 $BATCH_DIR"
echo "========================================="

total=0
success=0
failed=0

# 遍历所有接口目录
for interface_dir in "$BATCH_DIR"/*/; do
    if [ -d "$interface_dir" ]; then
        interface_name=$(basename "$interface_dir")
        echo "📦 测试 $interface_name..."
        
        cd "$interface_dir" || continue
        
        # 寻找实际的 Cargo.toml 文件
        cargo_dir=""
        if [ -f "Cargo.toml" ]; then
            cargo_dir="."
        else
            # 查找子目录中的 Cargo.toml
            for subdir in */; do
                if [ -f "$subdir/Cargo.toml" ]; then
                    cargo_dir="$subdir"
                    break
                fi
            done
        fi
        
        if [ -n "$cargo_dir" ]; then
            cd "$cargo_dir"
            total=$((total + 1))
            
            if cargo check --quiet 2>/dev/null; then
                echo "  ✅ $interface_name 编译成功"
                success=$((success + 1))
            else
                echo "  ❌ $interface_name 编译失败"
                failed=$((failed + 1))
                # 显示详细错误
                echo "     详细错误信息:"
                cargo check 2>&1 | head -10 | sed 's/^/     /'
            fi
            cd - >/dev/null
        else
            echo "  ⚠️  $interface_name 找不到 Cargo.toml，跳过"
        fi
        
        # 返回到批量目录
        cd - >/dev/null
    fi
done

echo "========================================="
echo "📊 验证结果："
echo "   总数: $total"
echo "   成功: $success"
echo "   失败: $failed"

if [ $total -gt 0 ]; then
    success_rate=$((success * 100 / total))
    echo "   成功率: $success_rate%"
    
    if [ $success_rate -eq 100 ]; then
        echo "🎉 所有接口编译成功！"
        exit 0
    else
        echo "⚠️  有 $failed 个接口编译失败"
        exit 1
    fi
else
    echo "❌ 没有找到可验证的接口"
    exit 1
fi
