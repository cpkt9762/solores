#!/usr/bin/env uv run
# /// script
# requires-python = ">=3.8"
# dependencies = ["colorama>=0.4.6"]
# ///

"""
Raydium 接口修复脚本 - MiniJinja 适配版本

专门针对新的 MiniJinja 模板系统生成的 sol_raydium_interface 进行修复
支持17和18账户场景的 SwapBaseIn/SwapBaseOut 指令

用法:
    ./scripts/fix_raydium_interface_minijinja.py --interface-dir path/to/sol_raydium_interface
"""

import os
import sys
import re
import argparse
import subprocess
import pathlib
from typing import Optional, List
from dataclasses import dataclass
from colorama import init, Fore, Style

# 初始化colorama
init(autoreset=True)

@dataclass
class FixResult:
    """修复结果"""
    success: bool
    message: str
    details: Optional[List[str]] = None

class Colors:
    """颜色定义"""
    SUCCESS = Fore.GREEN
    ERROR = Fore.RED
    WARNING = Fore.YELLOW
    INFO = Fore.BLUE
    HEADER = Fore.CYAN + Style.BRIGHT

def log_success(msg: str):
    print(f"{Colors.SUCCESS}✅ {msg}{Style.RESET_ALL}")

def log_error(msg: str):
    print(f"{Colors.ERROR}❌ {msg}{Style.RESET_ALL}")

def log_warning(msg: str):
    print(f"{Colors.WARNING}⚠️  {msg}{Style.RESET_ALL}")

def log_info(msg: str):
    print(f"{Colors.INFO}ℹ️  {msg}{Style.RESET_ALL}")

def log_header(msg: str):
    print(f"\n{Colors.HEADER}🔧 {msg}{Style.RESET_ALL}")

class RaydiumInterfaceFixerMiniJinja:
    """Raydium接口修复器 - MiniJinja版本"""
    
    def __init__(self, interface_dir: str):
        """初始化修复器"""
        self.interface_dir = pathlib.Path(interface_dir)
        self.instructions_file = self.interface_dir / "src/parsers/instructions.rs"
        self.swap_base_in_file = self.interface_dir / "src/instructions/swap_base_in.rs"
        self.swap_base_out_file = self.interface_dir / "src/instructions/swap_base_out.rs"
        
    def check_files_exist(self) -> FixResult:
        """检查必需文件是否存在"""
        log_info("📁 检查必需文件...")
        
        missing_files = []
        for file_path in [self.instructions_file, self.swap_base_in_file, self.swap_base_out_file]:
            if not file_path.exists():
                missing_files.append(str(file_path))
        
        if missing_files:
            return FixResult(False, f"缺少必需文件: {', '.join(missing_files)}")
        
        log_success("所有必需文件存在")
        return FixResult(True, "文件检查通过")
    
    def fix_instructions_parser(self) -> FixResult:
        """修复指令解析器长度检查"""
        log_header("修复指令解析器长度检查")
        
        try:
            content = self.instructions_file.read_text(encoding='utf-8')
            original_content = content
            
            # 只修复 SwapBaseIn 和 SwapBaseOut，跳过 Initialize
            
            # 1. 修复 SwapBaseIn 长度检查：支持 17 或 18 账户
            # 更新模式以匹配新的常量格式
            pattern1 = r'(if instruction_type == 9 \{[\s\S]*?)if accounts\.len\(\) < crate::instructions::SWAPBASEIN_IX_ACCOUNTS_LEN([\s\S]*?SwapBaseIn[\s\S]*?\})'
            replacement1 = r'\1if accounts.len() != 17 && accounts.len() != 18\2'
            content = re.sub(pattern1, replacement1, content)
            
            # 2. 修复 SwapBaseOut 长度检查：支持 17 或 18 账户
            # 更新模式以匹配新的常量格式
            pattern2 = r'(if instruction_type == 11 \{[\s\S]*?)if accounts\.len\(\) < crate::instructions::SWAPBASEOUT_IX_ACCOUNTS_LEN([\s\S]*?SwapBaseOut[\s\S]*?\})'
            replacement2 = r'\1if accounts.len() != 17 && accounts.len() != 18\2'
            content = re.sub(pattern2, replacement2, content)
            
            # 3. 账户传递方式已经是 accounts，无需修复
            # (现在模板已经直接使用 accounts，所以这个步骤不再需要)
            
            # 检查是否有更改
            if content == original_content:
                log_warning("指令解析器文件没有找到需要修复的内容")
                return FixResult(True, "指令解析器无需修复")
            
            # 写回文件
            self.instructions_file.write_text(content, encoding='utf-8')
            log_success("指令解析器修复完成")
            
            return FixResult(True, "指令解析器修复成功")
            
        except Exception as e:
            return FixResult(False, f"修复指令解析器失败: {e}")
    
    def fix_swap_base_in_keys(self) -> FixResult:
        """修复SwapBaseInKeys结构"""
        log_header("修复SwapBaseInKeys结构")
        
        try:
            content = self.swap_base_in_file.read_text(encoding='utf-8')
            original_content = content
            
            # 1. 将 amm_target_orders 字段改为可选
            pattern1 = r'pub amm_target_orders: solana_pubkey::Pubkey,'
            replacement1 = 'pub amm_target_orders: Option<solana_pubkey::Pubkey>,'
            content = re.sub(pattern1, replacement1, content)
            
            # 2. 修改 Copy 为 Clone（因为 Option 不支持 Copy）
            pattern2 = r'#\[derive\(Copy, Clone, Debug, Default\)\]'
            replacement2 = '#[derive(Clone, Debug, Default)]'
            content = re.sub(pattern2, replacement2, content)
            
            # 3. 替换 From<&[solana_pubkey::Pubkey]> 实现为支持动态账户数量
            from_impl_pattern = r'impl From<&\[solana_pubkey::Pubkey\]> for SwapBaseInKeys \{[\s\S]*?\n\}\n'
            from_impl_replacement = '''impl From<&[solana_pubkey::Pubkey]> for SwapBaseInKeys {
    fn from(pubkeys: &[solana_pubkey::Pubkey]) -> Self {
        let has_target_orders = pubkeys.len() >= 18;
        
        if has_target_orders {
            // 18账户情况：包含amm_target_orders
            Self {
                token_program: pubkeys[0],
                amm: pubkeys[1],
                amm_authority: pubkeys[2],
                amm_open_orders: pubkeys[3],
                amm_target_orders: Some(pubkeys[4]),
                pool_coin_token_account: pubkeys[5],
                pool_pc_token_account: pubkeys[6],
                serum_program: pubkeys[7],
                serum_market: pubkeys[8],
                serum_bids: pubkeys[9],
                serum_asks: pubkeys[10],
                serum_event_queue: pubkeys[11],
                serum_coin_vault_account: pubkeys[12],
                serum_pc_vault_account: pubkeys[13],
                serum_vault_signer: pubkeys[14],
                user_source_token_account: pubkeys[15],
                user_destination_token_account: pubkeys[16],
                user_source_owner: pubkeys[17],
            }
        } else {
            // 17账户情况：没有amm_target_orders
            Self {
                token_program: pubkeys[0],
                amm: pubkeys[1],
                amm_authority: pubkeys[2],
                amm_open_orders: pubkeys[3],
                amm_target_orders: None,
                pool_coin_token_account: pubkeys[4],
                pool_pc_token_account: pubkeys[5],
                serum_program: pubkeys[6],
                serum_market: pubkeys[7],
                serum_bids: pubkeys[8],
                serum_asks: pubkeys[9],
                serum_event_queue: pubkeys[10],
                serum_coin_vault_account: pubkeys[11],
                serum_pc_vault_account: pubkeys[12],
                serum_vault_signer: pubkeys[13],
                user_source_token_account: pubkeys[14],
                user_destination_token_account: pubkeys[15],
                user_source_owner: pubkeys[16],
            }
        }
    }
}
'''
            content = re.sub(from_impl_pattern, from_impl_replacement, content, flags=re.MULTILINE)
            
            # 4. 替换to_vec方法实现（处理Option<solana_pubkey::Pubkey>）
            # 使用精确匹配：匹配完整的to_vec方法，包括std::vec!宏的多行结构
            to_vec_pattern = r'/// Convert Keys to Vec<Pubkey>\s*\n\s*pub fn to_vec\(&self\) -> std::vec::Vec<solana_pubkey::Pubkey> \{\s*\n\s*std::vec!\[\s*(?:[^\]]*\n)*[^\]]*\]\s*\n\s*\}'
            to_vec_replacement = '''/// Convert Keys to Vec<Pubkey>
    pub fn to_vec(&self) -> std::vec::Vec<solana_pubkey::Pubkey> {
        let mut vec = vec![
            self.token_program,
            self.amm,
            self.amm_authority,
            self.amm_open_orders,
        ];
        
        // 条件性添加amm_target_orders
        if let Some(amm_target_orders) = self.amm_target_orders {
            vec.push(amm_target_orders);
        }
        
        vec.extend_from_slice(&[
            self.pool_coin_token_account,
            self.pool_pc_token_account,
            self.serum_program,
            self.serum_market,
            self.serum_bids,
            self.serum_asks,
            self.serum_event_queue,
            self.serum_coin_vault_account,
            self.serum_pc_vault_account,
            self.serum_vault_signer,
            self.user_source_token_account,
            self.user_destination_token_account,
            self.user_source_owner,
        ]);
        
        vec
    }'''
            content = re.sub(to_vec_pattern, to_vec_replacement, content, flags=re.MULTILINE)
            
            # 检查是否有更改
            if content == original_content:
                log_warning("SwapBaseInKeys文件没有找到需要修复的内容")
                return FixResult(True, "SwapBaseInKeys无需修复")
            
            # 写回文件
            self.swap_base_in_file.write_text(content, encoding='utf-8')
            log_success("SwapBaseInKeys修复完成")
            
            return FixResult(True, "SwapBaseInKeys修复成功")
            
        except Exception as e:
            return FixResult(False, f"修复SwapBaseInKeys失败: {e}")
    
    def fix_array_initializations(self) -> FixResult:
        """修复数组初始化问题（移除Copy trait依赖）"""
        log_header("修复数组初始化问题")
        
        try:
            # 修复 target_orders.rs 中的数组初始化
            target_orders_file = self.interface_dir / "src/accounts/target_orders.rs"
            if target_orders_file.exists():
                content = target_orders_file.read_text(encoding='utf-8')
                original_content = content
                
                # 将 [Default::default(); N] 替换为 core::array::from_fn
                pattern1 = r'buy_orders: \[Default::default\(\); 50\],'
                replacement1 = 'buy_orders: core::array::from_fn(|_| Default::default()),'
                content = re.sub(pattern1, replacement1, content)
                
                pattern2 = r'sell_orders: \[Default::default\(\); 50\],'
                replacement2 = 'sell_orders: core::array::from_fn(|_| Default::default()),'
                content = re.sub(pattern2, replacement2, content)
                
                if content != original_content:
                    target_orders_file.write_text(content, encoding='utf-8')
                    log_success("target_orders.rs 数组初始化修复")
            
            # 修复 withdraw_queue.rs 中的大数组 Default 问题
            withdraw_queue_file = self.interface_dir / "src/types/withdraw_queue.rs"
            if withdraw_queue_file.exists():
                content = withdraw_queue_file.read_text(encoding='utf-8')
                original_content = content
                
                # 移除 Default derive，添加自定义实现
                pattern = r'#\[derive\(borsh::BorshDeserialize, borsh::BorshSerialize, Clone, Debug, Default\)\]'
                replacement = '#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Clone, Debug)]'
                content = re.sub(pattern, replacement, content)
                
                # 添加自定义 Default 实现
                if 'impl Default for WithdrawQueue' not in content:
                    custom_default = '''
impl Default for WithdrawQueue {
    fn default() -> Self {
        Self {
            buf: core::array::from_fn(|_| Default::default()),
        }
    }
}
'''
                    content += custom_default
                
                if content != original_content:
                    withdraw_queue_file.write_text(content, encoding='utf-8')
                    log_success("withdraw_queue.rs 自定义Default实现")
            
            return FixResult(True, "数组初始化修复成功")
            
        except Exception as e:
            return FixResult(False, f"修复数组初始化失败: {e}")
    
    def fix_option_pubkey_serde(self) -> FixResult:
        """修复Option<Pubkey>的serde序列化问题"""
        log_header("修复Option<Pubkey>的serde序列化问题")
        
        try:
            files_to_fix = [
                self.swap_base_in_file,
                self.swap_base_out_file
            ]
            
            total_fixes = 0
            
            for file_path in files_to_fix:
                if not file_path.exists():
                    continue
                    
                content = file_path.read_text(encoding='utf-8')
                original_content = content
                
                # 修复Option<Pubkey>字段的错误serde属性
                # 错误模式：serde(with = "serde_with::As::<serde_with::DisplayFromStr>") 用于 Option<Pubkey>
                # 正确模式：serde(with = "serde_with::As::<std::option::Option<serde_with::DisplayFromStr>>") 用于 Option<Pubkey>
                
                # 简化：直接查找和替换所有错误的serde属性模式
                wrong_serde_text = 'serde(with = "serde_with::As::<serde_with::DisplayFromStr>")'
                correct_serde_text = 'serde(with = "serde_with::As::<std::option::Option<serde_with::DisplayFromStr>>")'
                
                # 只对Option<Pubkey>字段进行替换
                # 分步处理：找到Option<Pubkey>字段，然后向上查找其serde属性
                lines = content.split('\n')
                for i, line in enumerate(lines):
                    # 找到Option<Pubkey>字段
                    if 'pub ' in line and ': Option<solana_pubkey::Pubkey>' in line:
                        # 向上查找对应的serde属性（通常在前面几行）
                        for j in range(max(0, i-5), i):
                            if wrong_serde_text in lines[j]:
                                lines[j] = lines[j].replace(wrong_serde_text, correct_serde_text)
                                break
                
                content = '\n'.join(lines)
                
                # 检查是否有更改
                if content != original_content:
                    file_path.write_text(content, encoding='utf-8')
                    total_fixes += 1
                    log_success(f"{file_path.name} Option<Pubkey> serde属性修复完成")
            
            if total_fixes == 0:
                log_warning("没有找到需要修复的Option<Pubkey> serde属性")
                return FixResult(True, "Option<Pubkey> serde序列化无需修复")
            
            return FixResult(True, f"成功修复 {total_fixes} 个文件的Option<Pubkey> serde属性")
            
        except Exception as e:
            return FixResult(False, f"修复Option<Pubkey> serde序列化失败: {e}")
    
    def fix_swap_base_out_keys(self) -> FixResult:
        """修复SwapBaseOutKeys结构（相同模式）"""
        log_header("修复SwapBaseOutKeys结构")
        
        try:
            content = self.swap_base_out_file.read_text(encoding='utf-8')
            original_content = content
            
            # 应用与SwapBaseInKeys相同的修复模式
            # 1. 将 amm_target_orders 字段改为可选
            pattern1 = r'pub amm_target_orders: solana_pubkey::Pubkey,'
            replacement1 = 'pub amm_target_orders: Option<solana_pubkey::Pubkey>,'
            content = re.sub(pattern1, replacement1, content)
            
            # 2. 修改 Copy 为 Clone
            pattern2 = r'#\[derive\(Copy, Clone, Debug, Default\)\]'
            replacement2 = '#[derive(Clone, Debug, Default)]'
            content = re.sub(pattern2, replacement2, content)
            
            # 3. 替换 From<&[solana_pubkey::Pubkey]> 实现
            from_impl_pattern = r'impl From<&\[solana_pubkey::Pubkey\]> for SwapBaseOutKeys \{[\s\S]*?\n\}\n'
            from_impl_replacement = '''impl From<&[solana_pubkey::Pubkey]> for SwapBaseOutKeys {
    fn from(pubkeys: &[solana_pubkey::Pubkey]) -> Self {
        let has_target_orders = pubkeys.len() >= 18;
        
        if has_target_orders {
            // 18账户情况：包含amm_target_orders
            Self {
                token_program: pubkeys[0],
                amm: pubkeys[1],
                amm_authority: pubkeys[2],
                amm_open_orders: pubkeys[3],
                amm_target_orders: Some(pubkeys[4]),
                pool_coin_token_account: pubkeys[5],
                pool_pc_token_account: pubkeys[6],
                serum_program: pubkeys[7],
                serum_market: pubkeys[8],
                serum_bids: pubkeys[9],
                serum_asks: pubkeys[10],
                serum_event_queue: pubkeys[11],
                serum_coin_vault_account: pubkeys[12],
                serum_pc_vault_account: pubkeys[13],
                serum_vault_signer: pubkeys[14],
                user_source_token_account: pubkeys[15],
                user_destination_token_account: pubkeys[16],
                user_source_owner: pubkeys[17],
            }
        } else {
            // 17账户情况：没有amm_target_orders
            Self {
                token_program: pubkeys[0],
                amm: pubkeys[1],
                amm_authority: pubkeys[2],
                amm_open_orders: pubkeys[3],
                amm_target_orders: None,
                pool_coin_token_account: pubkeys[4],
                pool_pc_token_account: pubkeys[5],
                serum_program: pubkeys[6],
                serum_market: pubkeys[7],
                serum_bids: pubkeys[8],
                serum_asks: pubkeys[9],
                serum_event_queue: pubkeys[10],
                serum_coin_vault_account: pubkeys[11],
                serum_pc_vault_account: pubkeys[12],
                serum_vault_signer: pubkeys[13],
                user_source_token_account: pubkeys[14],
                user_destination_token_account: pubkeys[15],
                user_source_owner: pubkeys[16],
            }
        }
    }
}
'''
            content = re.sub(from_impl_pattern, from_impl_replacement, content, flags=re.MULTILINE)
            
            # 4. 替换to_vec方法实现（处理Option<solana_pubkey::Pubkey>）
            # 使用精确匹配：匹配完整的to_vec方法，包括std::vec!宏的多行结构
            to_vec_pattern = r'/// Convert Keys to Vec<Pubkey>\s*\n\s*pub fn to_vec\(&self\) -> std::vec::Vec<solana_pubkey::Pubkey> \{\s*\n\s*std::vec!\[\s*(?:[^\]]*\n)*[^\]]*\]\s*\n\s*\}'
            to_vec_replacement = '''/// Convert Keys to Vec<Pubkey>
    pub fn to_vec(&self) -> std::vec::Vec<solana_pubkey::Pubkey> {
        let mut vec = vec![
            self.token_program,
            self.amm,
            self.amm_authority,
            self.amm_open_orders,
        ];
        
        // 条件性添加amm_target_orders
        if let Some(amm_target_orders) = self.amm_target_orders {
            vec.push(amm_target_orders);
        }
        
        vec.extend_from_slice(&[
            self.pool_coin_token_account,
            self.pool_pc_token_account,
            self.serum_program,
            self.serum_market,
            self.serum_bids,
            self.serum_asks,
            self.serum_event_queue,
            self.serum_coin_vault_account,
            self.serum_pc_vault_account,
            self.serum_vault_signer,
            self.user_source_token_account,
            self.user_destination_token_account,
            self.user_source_owner,
        ]);
        
        vec
    }'''
            content = re.sub(to_vec_pattern, to_vec_replacement, content, flags=re.MULTILINE)
            
            # 检查是否有更改
            if content == original_content:
                log_warning("SwapBaseOutKeys文件没有找到需要修复的内容")
                return FixResult(True, "SwapBaseOutKeys无需修复")
            
            # 写回文件
            self.swap_base_out_file.write_text(content, encoding='utf-8')
            log_success("SwapBaseOutKeys修复完成")
            
            return FixResult(True, "SwapBaseOutKeys修复成功")
            
        except Exception as e:
            return FixResult(False, f"修复SwapBaseOutKeys失败: {e}")
    
    def validate_fixes(self) -> FixResult:
        """验证修复是否应用"""
        log_header("验证修复应用")
        
        try:
            # 检查指令解析器修复
            instructions_content = self.instructions_file.read_text(encoding='utf-8')
            
            checks = []
            
            # 检查长度检查修复 - 只验证SwapBaseIn和SwapBaseOut，跳过Initialize
            swap_base_checks = instructions_content.count("accounts.len() != 17 && accounts.len() != 18")
            if swap_base_checks >= 2:  # SwapBaseIn + SwapBaseOut
                checks.append("✅ SwapBaseIn/Out 长度检查修复已应用")
            else:
                checks.append("❌ SwapBaseIn/Out 长度检查修复失败")
            
            # 验证Initialize保持原有逻辑（现在使用常量）
            if "if instruction_type == 0" in instructions_content:
                initialize_section = re.search(r'if instruction_type == 0 \{[\s\S]*?return Ok\(Self::Initialize[\s\S]*?\}\);', instructions_content)
                if initialize_section and "crate::instructions::INITIALIZE_IX_ACCOUNTS_LEN" in initialize_section.group():
                    checks.append("✅ Initialize 使用常量进行账户检查（已更新至新格式）")
                else:
                    checks.append("❌ Initialize 账户检查逻辑异常")
            
            # 检查账户传递方式（现在直接使用accounts）
            if "SwapBaseInKeys::from(accounts)" in instructions_content and "SwapBaseOutKeys::from(accounts)" in instructions_content:
                checks.append("✅ 账户传递已使用直接传递方式")
            else:
                checks.append("❌ 账户传递方式异常")
            
            # 检查是否使用了新的常量格式
            constant_usage_count = instructions_content.count("_IX_ACCOUNTS_LEN")
            if constant_usage_count > 10:  # 应该有很多指令使用常量
                checks.append(f"✅ 新常量格式已被广泛使用 ({constant_usage_count} 个引用)")
            else:
                checks.append(f"⚠️  常量格式使用较少 ({constant_usage_count} 个引用)")
            
            # 检查SwapBaseInKeys修复
            swap_in_content = self.swap_base_in_file.read_text(encoding='utf-8')
            if "pub amm_target_orders: Option<solana_pubkey::Pubkey>" in swap_in_content:
                checks.append("✅ SwapBaseInKeys 可选字段修复已应用")
            else:
                checks.append("❌ SwapBaseInKeys 可选字段修复失败")
            
            # 检查SwapBaseOutKeys修复
            swap_out_content = self.swap_base_out_file.read_text(encoding='utf-8')
            if "pub amm_target_orders: Option<solana_pubkey::Pubkey>" in swap_out_content:
                checks.append("✅ SwapBaseOutKeys 可选字段修复已应用")
            else:
                checks.append("❌ SwapBaseOutKeys 可选字段修复失败")
            
            # 检查Option<Pubkey> serde修复
            serde_fixes_found = 0
            for file_path in [self.swap_base_in_file, self.swap_base_out_file]:
                if file_path.exists():
                    content = file_path.read_text(encoding='utf-8')
                    # 检查是否有正确的Option<Pubkey> serde模式
                    if 'serde(with = "serde_with::As::<std::option::Option<serde_with::DisplayFromStr>>")' in content:
                        serde_fixes_found += 1
            
            if serde_fixes_found > 0:
                checks.append(f"✅ Option<Pubkey> serde序列化修复已应用 ({serde_fixes_found} 个文件)")
            else:
                # 检查是否仍有错误的模式
                wrong_serde_pattern = 0
                for file_path in [self.swap_base_in_file, self.swap_base_out_file]:
                    if file_path.exists():
                        content = file_path.read_text(encoding='utf-8')
                        # 查找错误的serde模式用于Option<Pubkey>
                        if re.search(r'serde\(with = "serde_with::As::<serde_with::DisplayFromStr>"\)\s*\]\s*pub\s+\w+: Option<solana_pubkey::Pubkey>', content):
                            wrong_serde_pattern += 1
                
                if wrong_serde_pattern > 0:
                    checks.append("❌ Option<Pubkey> serde序列化仍有错误模式")
                else:
                    checks.append("✅ Option<Pubkey> serde序列化无需修复")
            
            # 打印检查结果
            for check in checks:
                if check.startswith("✅"):
                    log_success(check[2:])  # 去掉emoji前缀
                else:
                    log_error(check[2:])
            
            failed_checks = [c for c in checks if c.startswith("❌")]
            if failed_checks:
                return FixResult(False, f"{len(failed_checks)} 个验证检查失败")
            
            return FixResult(True, "所有修复验证通过")
            
        except Exception as e:
            return FixResult(False, f"验证修复失败: {e}")
    
    def run_compilation_test(self) -> FixResult:
        """运行编译测试"""
        log_header("运行编译测试")
        
        try:
            os.chdir(self.interface_dir)
            
            # 1. 基础编译测试
            result = subprocess.run(
                ["cargo", "check", "--quiet"],
                capture_output=True,
                text=True
            )
            
            if result.returncode != 0:
                log_error(f"基础编译测试失败: {result.stderr}")
                return FixResult(False, f"基础编译失败: {result.stderr}")
                
            log_success("基础编译测试通过")
            
            # 2. serde特性编译测试
            log_info("运行serde特性编译测试...")
            serde_result = subprocess.run(
                ["cargo", "check", "--features", "serde", "--quiet"],
                capture_output=True,
                text=True
            )
            
            if serde_result.returncode != 0:
                log_error(f"serde特性编译失败: {serde_result.stderr}")
                return FixResult(False, f"serde编译失败: {serde_result.stderr}")
                
            log_success("serde特性编译测试通过")
            return FixResult(True, "编译测试成功（包括serde特性）")
                
        except Exception as e:
            return FixResult(False, f"运行编译测试失败: {e}")
    
    def run_fixes(self) -> bool:
        """执行所有修复"""
        log_header("🚀 开始 Raydium 接口修复 (MiniJinja版本)...")
        
        # 检查文件存在
        result = self.check_files_exist()
        if not result.success:
            log_error(result.message)
            return False
        
        success = True
        try:
            # 执行修复步骤
            steps = [
                ("指令解析器修复", self.fix_instructions_parser),
                ("SwapBaseInKeys修复", self.fix_swap_base_in_keys), 
                ("SwapBaseOutKeys修复", self.fix_swap_base_out_keys),
                ("Option<Pubkey>序列化修复", self.fix_option_pubkey_serde),
                ("数组初始化修复", self.fix_array_initializations),
                ("验证修复", self.validate_fixes),
                ("编译测试", self.run_compilation_test),
            ]
            
            for step_name, step_func in steps:
                result = step_func()
                if not result.success:
                    log_error(f"{step_name}失败: {result.message}")
                    success = False
                    break
            
            if success:
                log_header("🎉 Raydium 接口修复成功完成！")
                print()
                print("📋 修复摘要:")
                print("  ✅ SwapBaseIn 现在支持 17 和 18 账户场景")
                print("  ✅ SwapBaseOut 现在支持 17 和 18 账户场景") 
                print("  ✅ Initialize 使用常量进行账户检查（适配新的常量格式）")
                print("  ✅ amm_target_orders 字段在17账户时为 None，18账户时为 Some")
                print("  ✅ Option<Pubkey> serde序列化属性已修复")
                print("  ✅ 所有修复已通过编译测试（包括serde特性）")
                print("  ✅ 已适配新的常量引用格式（INSTRUCTION_NAME_IX_ACCOUNTS_LEN）")
                print()
            
            return success
            
        except KeyboardInterrupt:
            log_warning("用户中断")
            return False
        except Exception as e:
            log_error(f"修复过程中出现未预期错误: {e}")
            return False

def main():
    """主函数"""
    parser = argparse.ArgumentParser(description="Raydium 接口修复脚本 - MiniJinja版")
    parser.add_argument("--interface-dir", required=True, 
                       help="sol_raydium_interface 目录路径")
    
    args = parser.parse_args()
    
    # 检查目录是否存在
    interface_dir = pathlib.Path(args.interface_dir)
    if not interface_dir.exists():
        log_error(f"接口目录不存在: {interface_dir}")
        sys.exit(1)
    
    # 创建修复器并运行
    fixer = RaydiumInterfaceFixerMiniJinja(args.interface_dir)
    success = fixer.run_fixes()
    
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()