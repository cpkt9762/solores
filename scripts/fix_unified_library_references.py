#!/usr/bin/env python3
"""
统一库引用错误修正脚本

自动扫描并修正统一库中的 crate::module 引用错误，
将其替换为正确的 crate::protocol::module 路径。
"""

import os
import re
import glob
from pathlib import Path

def find_protocol_for_file(file_path: str) -> str:
    """根据文件路径确定协议名称"""
    # 例如：src/moonshot/events/buy_event.rs → moonshot
    parts = Path(file_path).parts
    if len(parts) >= 2 and parts[0] == 'src':
        return parts[1]
    return None

def fix_crate_references(file_path: str, protocol_name: str) -> tuple[bool, int]:
    """修正文件中的 crate:: 引用"""
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_content = content
    fixes_count = 0
    
    # 修正模式列表
    patterns = [
        # errors 模块引用
        (r'use crate::errors::', f'use crate::{protocol_name}::errors::'),
        (r'crate::errors::', f'crate::{protocol_name}::errors::'),
        
        # 其他模块引用
        (r'use crate::accounts::', f'use crate::{protocol_name}::accounts::'),
        (r'crate::accounts::', f'crate::{protocol_name}::accounts::'),
        (r'use crate::types::', f'use crate::{protocol_name}::types::'),
        (r'crate::types::', f'crate::{protocol_name}::types::'),
        (r'use crate::events::', f'use crate::{protocol_name}::events::'),
        (r'crate::events::', f'crate::{protocol_name}::events::'),
        (r'use crate::instructions::', f'use crate::{protocol_name}::instructions::'),
        (r'crate::instructions::', f'crate::{protocol_name}::instructions::'),
        (r'use crate::parsers::', f'use crate::{protocol_name}::parsers::'),
        (r'crate::parsers::', f'crate::{protocol_name}::parsers::'),
        
        # ID 引用
        (r'crate::ID', f'crate::{protocol_name}::ID'),
    ]
    
    # 应用所有修正模式
    for pattern, replacement in patterns:
        new_content = re.sub(pattern, replacement, content)
        if new_content != content:
            matches = len(re.findall(pattern, content))
            print(f"  修正 {matches} 处: {pattern} → {replacement}")
            fixes_count += matches
            content = new_content
    
    # 如果有修改则写回文件
    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        return True, fixes_count
    
    return False, 0

def is_raydium_protocol(protocol_name: str) -> bool:
    """检测是否为 Raydium 协议"""
    return protocol_name == "raydium"

def fix_raydium_swap_instructions(file_path: str, protocol_name: str) -> int:
    """修复 Raydium SwapBaseIn/SwapBaseOut 指令的账户数量检查"""
    if not is_raydium_protocol(protocol_name):
        return 0
    
    if not ("swap_base_in" in file_path.lower() or "swap_base_out" in file_path.lower() or "parsers/instructions" in file_path):
        return 0
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        fixes_count = 0
        
        # 修复 SwapBaseIn 长度检查：支持 17 或 18 账户
        pattern1 = r'if accounts\.len\(\) < crate::[^:]+::instructions::SWAPBASEIN_IX_ACCOUNTS_LEN'
        replacement1 = 'if accounts.len() != 17 && accounts.len() != 18'
        new_content = re.sub(pattern1, replacement1, content)
        if new_content != content:
            print(f"  修正 SwapBaseIn 账户长度检查")
            fixes_count += 1
            content = new_content
        
        # 修复 SwapBaseOut 长度检查：支持 17 或 18 账户  
        pattern2 = r'if accounts\.len\(\) < crate::[^:]+::instructions::SWAPBASEOUT_IX_ACCOUNTS_LEN'
        replacement2 = 'if accounts.len() != 17 && accounts.len() != 18'
        new_content = re.sub(pattern2, replacement2, content)
        if new_content != content:
            print(f"  修正 SwapBaseOut 账户长度检查")
            fixes_count += 1
            content = new_content
            
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
        
        return fixes_count
    except Exception as e:
        print(f"  Raydium SwapBase 修复错误: {e}")
        return 0

def fix_raydium_account_keys(file_path: str, protocol_name: str) -> int:
    """修复 Raydium SwapBaseIn/SwapBaseOut Keys 结构"""
    if not is_raydium_protocol(protocol_name):
        return 0
    
    if not ("swap_base_in" in file_path.lower() or "swap_base_out" in file_path.lower()):
        return 0
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        fixes_count = 0
        
        # 1. 将 amm_target_orders 字段改为可选
        pattern1 = r'pub amm_target_orders: solana_pubkey::Pubkey,'
        replacement1 = 'pub amm_target_orders: Option<solana_pubkey::Pubkey>,'
        new_content = re.sub(pattern1, replacement1, content)
        if new_content != content:
            print(f"  修正 amm_target_orders 为 Option 类型")
            fixes_count += 1
            content = new_content
            
        # 2. 修改 Copy 为 Clone（因为 Option 不支持 Copy）
        pattern2 = r'#\[derive\(Copy, Clone, Debug, Default\)\]'
        replacement2 = '#[derive(Clone, Debug, Default)]'
        new_content = re.sub(pattern2, replacement2, content)
        if new_content != content:
            print(f"  移除 Copy trait，保留 Clone")
            fixes_count += 1
            content = new_content

        # 3. 修复 From<&[Pubkey]> 实现中的 amm_target_orders 赋值
        pattern3 = r'amm_target_orders: pubkeys\[4\],'
        replacement3 = 'amm_target_orders: Some(pubkeys[4]),'
        new_content = re.sub(pattern3, replacement3, content)
        if new_content != content:
            print(f"  修正 From impl 中的 amm_target_orders 赋值")
            fixes_count += 1
            content = new_content

        # 4. 修复 to_vec() 方法中的 Option 处理
        pattern4 = r'self\.amm_target_orders,'
        replacement4 = 'self.amm_target_orders.unwrap_or_default(),'
        new_content = re.sub(pattern4, replacement4, content)
        if new_content != content:
            print(f"  修正 to_vec 中的 Option 展开")
            fixes_count += 1
            content = new_content

        # 5. 修复 serde 属性以支持 Option<Pubkey>
        pattern5 = r'#\[cfg_attr\(feature = "serde", serde\(with = "serde_with::As::<serde_with::DisplayFromStr>"\)\)\]\s+pub amm_target_orders: Option<solana_pubkey::Pubkey>,'
        replacement5 = '#[cfg_attr(feature = "serde", serde(with = "serde_with::As::<std::option::Option<serde_with::DisplayFromStr>>"))]\n    pub amm_target_orders: Option<solana_pubkey::Pubkey>,'
        new_content = re.sub(pattern5, replacement5, content)
        if new_content != content:
            print(f"  修正 serde 属性以支持 Option<Pubkey>")
            fixes_count += 1
            content = new_content

        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
        
        return fixes_count
    except Exception as e:
        print(f"  Raydium 账户结构修复错误: {e}")
        return 0

def fix_raydium_array_initialization(file_path: str, protocol_name: str) -> int:
    """修复 Raydium 数组初始化问题"""
    if not is_raydium_protocol(protocol_name):
        return 0
    
    if not "target_orders" in file_path.lower():
        return 0
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        fixes_count = 0
        
        # 修复数组初始化
        patterns = [
            (r'buy_orders: \[Default::default\(\); 50\],', 'buy_orders: core::array::from_fn(|_| Default::default()),'),
            (r'sell_orders: \[Default::default\(\); 50\],', 'sell_orders: core::array::from_fn(|_| Default::default()),'),
        ]
        
        for pattern, replacement in patterns:
            new_content = re.sub(pattern, replacement, content)
            if new_content != content:
                print(f"  修正数组初始化: {pattern}")
                fixes_count += 1
                content = new_content
                
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
        
        return fixes_count
    except Exception as e:
        print(f"  Raydium 数组初始化修复错误: {e}")
        return 0

def apply_raydium_special_fixes(file_path: str, protocol_name: str) -> int:
    """应用所有 Raydium 特殊修复"""
    if not is_raydium_protocol(protocol_name):
        return 0
    
    total_fixes = 0
    total_fixes += fix_raydium_swap_instructions(file_path, protocol_name)
    total_fixes += fix_raydium_account_keys(file_path, protocol_name)
    total_fixes += fix_raydium_array_initialization(file_path, protocol_name)
    
    return total_fixes

def main():
    """主函数"""
    import sys

    print("🔧 开始修正统一库引用错误...")

    # 支持命令行参数指定统一库路径
    if len(sys.argv) > 1:
        # 使用命令行参数指定的路径
        specified_path = sys.argv[1]
        if os.path.isdir(specified_path):
            unified_lib_dirs = [specified_path]
            print(f"📂 使用指定路径: {specified_path}")
        else:
            print(f"❌ 指定路径不存在: {specified_path}")
            return
    else:
        # 默认查找 batch_output 目录
        unified_lib_dirs = glob.glob("batch_output/*/")
        if not unified_lib_dirs:
            print("❌ 未找到统一库输出目录")
            print("💡 提示: 可以使用命令行参数指定路径：python3 script.py <统一库路径>")
            return

    for unified_dir in unified_lib_dirs:
        print(f"\n📁 处理统一库: {unified_dir}")

        # 扫描所有 .rs 文件
        rs_files = glob.glob(os.path.join(unified_dir, "src/**/*.rs"), recursive=True)

        total_files_modified = 0
        total_fixes = 0
        raydium_files_modified = 0
        raydium_total_fixes = 0

        for rs_file in rs_files:
            # 跳过 lib.rs
            if rs_file.endswith('/lib.rs') or rs_file.endswith('\\lib.rs'):
                continue

            protocol_name = find_protocol_for_file(os.path.relpath(rs_file, unified_dir))
            if not protocol_name:
                continue

            # 先修复通用引用
            modified, fixes = fix_crate_references(rs_file, protocol_name)

            # 如果是 Raydium 协议，应用特殊修复
            if is_raydium_protocol(protocol_name):
                raydium_fixes = apply_raydium_special_fixes(rs_file, protocol_name)
                if raydium_fixes > 0:
                    raydium_files_modified += 1
                    raydium_total_fixes += raydium_fixes
                    fixes += raydium_fixes
                    modified = True
                    print(f"✅ 修正文件: {os.path.relpath(rs_file, unified_dir)} ({fixes} 处修改，含 {raydium_fixes} 处 Raydium 特殊修复)")
                elif modified:
                    print(f"✅ 修正文件: {os.path.relpath(rs_file, unified_dir)} ({fixes} 处修改)")
                    total_files_modified += 1
                    total_fixes += fixes
            elif modified:
                print(f"✅ 修正文件: {os.path.relpath(rs_file, unified_dir)} ({fixes} 处修改)")
                total_files_modified += 1
                total_fixes += fixes

        print(f"\n📊 统计:")
        print(f"   通用修复: 修改了 {total_files_modified} 个文件，共 {total_fixes} 处引用")
        if raydium_total_fixes > 0:
            print(f"   Raydium 特殊修复: 修改了 {raydium_files_modified} 个文件，共 {raydium_total_fixes} 处修复")
        print(f"   总计: 修改了 {total_files_modified + raydium_files_modified} 个文件，共 {total_fixes + raydium_total_fixes} 处修复")

    print(f"\n🎉 统一库引用错误修正完成！")

if __name__ == "__main__":
    main()