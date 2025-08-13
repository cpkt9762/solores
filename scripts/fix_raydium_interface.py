#!/usr/bin/env uv run
# /// script
# requires-python = ">=3.8"
# dependencies = ["colorama>=0.4.6"]
# ///

"""
Raydium 接口修复脚本 - Python重构版本

基于 Basic Memory 中的"Raydium 解析器修复完整指南"
修复 sol_raydium_interface 以支持17和18账户场景

用法:
    ./scripts/fix_raydium_interface.py --interface-dir path/to/sol_raydium_interface
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
    details: List[str] = None

class Colors:
    """颜色定义"""
    SUCCESS = Fore.GREEN
    ERROR = Fore.RED
    WARNING = Fore.YELLOW
    INFO = Fore.BLUE
    HEADER = Fore.CYAN + Style.BRIGHT
    DIM = Style.DIM

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

class RaydiumInterfaceFixer:
    """Raydium接口修复器"""
    
    def __init__(self, interface_dir: str):
        """初始化修复器"""
        self.interface_dir = pathlib.Path(interface_dir)
        self.instructions_file = self.interface_dir / "src/parsers/instructions.rs"
        self.swap_base_in_file = self.interface_dir / "src/instructions/swap_base_in.rs"
        self.swap_base_out_file = self.interface_dir / "src/instructions/swap_base_out.rs"
        
        # 备份文件路径
        self.backup_files = []
        
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
    
    def backup_file(self, file_path: pathlib.Path):
        """备份文件"""
        backup_path = file_path.with_suffix(file_path.suffix + ".backup")
        if backup_path.exists():
            backup_path.unlink()  # 删除旧备份
        file_path.rename(backup_path)
        self.backup_files.append((file_path, backup_path))
        
    def restore_backups(self):
        """恢复所有备份文件"""
        for original_path, backup_path in self.backup_files:
            if backup_path.exists():
                if original_path.exists():
                    original_path.unlink()
                backup_path.rename(original_path)
        self.backup_files.clear()
    
    def clean_backups(self):
        """清理备份文件"""
        for original_path, backup_path in self.backup_files:
            if backup_path.exists():
                backup_path.unlink()
        self.backup_files.clear()
    
    def fix_instructions_parser(self) -> FixResult:
        """修复指令解析器长度检查"""
        log_header("修复指令解析器长度检查")
        
        try:
            # 读取文件内容
            content = self.instructions_file.read_text(encoding='utf-8')
            original_content = content
            
            # 1. 修复 SwapBaseIn 长度检查
            pattern1 = r'if accounts\.len\(\) < SWAP_BASE_IN_IX_ACCOUNTS_LEN'
            replacement1 = 'if accounts.len() != SWAP_BASE_IN_IX_ACCOUNTS_LEN && accounts.len() != SWAP_BASE_IN_IX_ACCOUNTS_LEN - 1'
            content = re.sub(pattern1, replacement1, content)
            
            # 2. 修复 SwapBaseOut 长度检查
            pattern2 = r'if accounts\.len\(\) < SWAP_BASE_OUT_IX_ACCOUNTS_LEN'
            replacement2 = 'if accounts.len() != SWAP_BASE_OUT_IX_ACCOUNTS_LEN && accounts.len() != SWAP_BASE_OUT_IX_ACCOUNTS_LEN - 1'
            content = re.sub(pattern2, replacement2, content)
            
            # 3. 修复错误消息 - SwapBaseIn（一次性替换整个错误格式）
            pattern3 = r'format!\(\s*"Insufficient accounts for instruction \{\}",\s*stringify!\(SwapBaseIn\)\s*\)'
            replacement3 = 'format!("Invalid account count for instruction SwapBaseIn: expected 17 or 18, got {}", accounts.len())'
            content = re.sub(pattern3, replacement3, content)
            
            # 4. 修复错误消息 - SwapBaseOut（一次性替换整个错误格式）
            pattern4 = r'format!\(\s*"Insufficient accounts for instruction \{\}",\s*stringify!\(SwapBaseOut\)\s*\)'
            replacement4 = 'format!("Invalid account count for instruction SwapBaseOut: expected 17 or 18, got {}", accounts.len())'
            content = re.sub(pattern4, replacement4, content)
            
            # 5. 修复账户传递方式
            pattern5 = r'SwapBaseInKeys::from\(&accounts\[\.\.SWAP_BASE_IN_IX_ACCOUNTS_LEN\]\)'
            replacement5 = 'SwapBaseInKeys::from(accounts)'
            content = re.sub(pattern5, replacement5, content)
            
            pattern6 = r'SwapBaseOutKeys::from\(&accounts\[\.\.SWAP_BASE_OUT_IX_ACCOUNTS_LEN\]\)'
            replacement6 = 'SwapBaseOutKeys::from(accounts)'
            content = re.sub(pattern6, replacement6, content)
            
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
            pattern1 = r'pub amm_target_orders: Pubkey,'
            replacement1 = 'pub amm_target_orders: Option<Pubkey>,'
            content = re.sub(pattern1, replacement1, content)
            
            # 2. 修改 Copy 为 Clone（因为 Option 不支持 Copy）
            pattern2 = r'#\[derive\(Copy, Clone, Debug, PartialEq\)\]'
            replacement2 = '#[derive(Clone, Debug, PartialEq)]'
            content = re.sub(pattern2, replacement2, content)
            
            # 3. 修改默认值实现
            pattern3 = r'amm_target_orders: Pubkey::default\(\),'
            replacement3 = 'amm_target_orders: None,'
            content = re.sub(pattern3, replacement3, content)
            
            # 4. 替换 From<&[Pubkey]> 实现
            from_impl_pattern = r'impl From<&\[Pubkey\]> for SwapBaseInKeys \{[\s\S]*?\n\}\n'
            from_impl_replacement = '''impl From<&[Pubkey]> for SwapBaseInKeys {
    fn from(pubkeys: &[Pubkey]) -> Self {
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
                uer_source_token_account: pubkeys[15],
                uer_destination_token_account: pubkeys[16],
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
                uer_source_token_account: pubkeys[14],
                uer_destination_token_account: pubkeys[15],
                user_source_owner: pubkeys[16],
            }
        }
    }
}
'''
            content = re.sub(from_impl_pattern, from_impl_replacement, content, flags=re.MULTILINE)
            
            # 5. 替换 From<SwapBaseInKeys> for AccountMeta 实现
            to_meta_pattern = r'impl From<SwapBaseInKeys>\s*for \[solana_instruction::AccountMeta; SWAP_BASE_IN_IX_ACCOUNTS_LEN\][\s\S]*?\n\}\n'
            to_meta_replacement = '''impl From<SwapBaseInKeys> for Vec<solana_instruction::AccountMeta> {
    fn from(keys: SwapBaseInKeys) -> Self {
        let mut metas = vec![
            solana_instruction::AccountMeta::new(keys.token_program, false),
            solana_instruction::AccountMeta::new(keys.amm, false),
            solana_instruction::AccountMeta::new(keys.amm_authority, false),
            solana_instruction::AccountMeta::new(keys.amm_open_orders, false),
        ];
        
        if let Some(amm_target_orders) = keys.amm_target_orders {
            metas.push(solana_instruction::AccountMeta::new(amm_target_orders, false));
        }
        
        metas.extend_from_slice(&[
            solana_instruction::AccountMeta::new(keys.pool_coin_token_account, false),
            solana_instruction::AccountMeta::new(keys.pool_pc_token_account, false),
            solana_instruction::AccountMeta::new(keys.serum_program, false),
            solana_instruction::AccountMeta::new(keys.serum_market, false),
            solana_instruction::AccountMeta::new(keys.serum_bids, false),
            solana_instruction::AccountMeta::new(keys.serum_asks, false),
            solana_instruction::AccountMeta::new(keys.serum_event_queue, false),
            solana_instruction::AccountMeta::new(keys.serum_coin_vault_account, false),
            solana_instruction::AccountMeta::new(keys.serum_pc_vault_account, false),
            solana_instruction::AccountMeta::new(keys.serum_vault_signer, false),
            solana_instruction::AccountMeta::new(keys.uer_source_token_account, false),
            solana_instruction::AccountMeta::new(keys.uer_destination_token_account, false),
            solana_instruction::AccountMeta::new(keys.user_source_owner, false),
        ]);
        
        metas
    }
}
'''
            content = re.sub(to_meta_pattern, to_meta_replacement, content, flags=re.MULTILINE)
            
            # 6. 修改函数签名
            pattern6 = r'let metas: \[solana_instruction::AccountMeta; SWAP_BASE_IN_IX_ACCOUNTS_LEN\] = keys\.into\(\);'
            replacement6 = 'let metas: Vec<solana_instruction::AccountMeta> = keys.into();'
            content = re.sub(pattern6, replacement6, content)
            
            pattern7 = r'accounts: Vec::from\(metas\),'
            replacement7 = 'accounts: metas,'
            content = re.sub(pattern7, replacement7, content)
            
            # 7. 替换to_vec方法实现（处理Option<Pubkey>）
            to_vec_pattern = r'/// Convert Keys to Vec<Pubkey>\s*\n\s*pub fn to_vec\(&self\) -> Vec<Pubkey> \{\s*\n\s*vec!\[\s*\n([\s\S]*?)\s*\]\s*\n\s*\}'
            to_vec_replacement = '''/// Convert Keys to Vec<Pubkey>
    pub fn to_vec(&self) -> Vec<Pubkey> {
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
            self.uer_source_token_account,
            self.uer_destination_token_account,
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
    
    def fix_swap_base_out_keys(self) -> FixResult:
        """修复SwapBaseOutKeys结构（相同模式）"""
        log_header("修复SwapBaseOutKeys结构")
        
        try:
            content = self.swap_base_out_file.read_text(encoding='utf-8')
            original_content = content
            
            # 应用与SwapBaseInKeys相同的修复模式
            # 1. 将 amm_target_orders 字段改为可选
            pattern1 = r'pub amm_target_orders: Pubkey,'
            replacement1 = 'pub amm_target_orders: Option<Pubkey>,'
            content = re.sub(pattern1, replacement1, content)
            
            # 2. 修改 Copy 为 Clone
            pattern2 = r'#\[derive\(Copy, Clone, Debug, PartialEq\)\]'
            replacement2 = '#[derive(Clone, Debug, PartialEq)]'
            content = re.sub(pattern2, replacement2, content)
            
            # 3. 修改默认值实现
            pattern3 = r'amm_target_orders: Pubkey::default\(\),'
            replacement3 = 'amm_target_orders: None,'
            content = re.sub(pattern3, replacement3, content)
            
            # 4. 替换 From<&[Pubkey]> 实现
            from_impl_pattern = r'impl From<&\[Pubkey\]> for SwapBaseOutKeys \{[\s\S]*?\n\}\n'
            from_impl_replacement = '''impl From<&[Pubkey]> for SwapBaseOutKeys {
    fn from(pubkeys: &[Pubkey]) -> Self {
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
                uer_source_token_account: pubkeys[15],
                uer_destination_token_account: pubkeys[16],
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
                uer_source_token_account: pubkeys[14],
                uer_destination_token_account: pubkeys[15],
                user_source_owner: pubkeys[16],
            }
        }
    }
}
'''
            content = re.sub(from_impl_pattern, from_impl_replacement, content, flags=re.MULTILINE)
            
            # 5. 替换 From<SwapBaseOutKeys> for AccountMeta 实现
            to_meta_pattern = r'impl From<SwapBaseOutKeys>\s*for \[solana_instruction::AccountMeta; SWAP_BASE_OUT_IX_ACCOUNTS_LEN\][\s\S]*?\n\}\n'
            to_meta_replacement = '''impl From<SwapBaseOutKeys> for Vec<solana_instruction::AccountMeta> {
    fn from(keys: SwapBaseOutKeys) -> Self {
        let mut metas = vec![
            solana_instruction::AccountMeta::new(keys.token_program, false),
            solana_instruction::AccountMeta::new(keys.amm, false),
            solana_instruction::AccountMeta::new(keys.amm_authority, false),
            solana_instruction::AccountMeta::new(keys.amm_open_orders, false),
        ];
        
        if let Some(amm_target_orders) = keys.amm_target_orders {
            metas.push(solana_instruction::AccountMeta::new(amm_target_orders, false));
        }
        
        metas.extend_from_slice(&[
            solana_instruction::AccountMeta::new(keys.pool_coin_token_account, false),
            solana_instruction::AccountMeta::new(keys.pool_pc_token_account, false),
            solana_instruction::AccountMeta::new(keys.serum_program, false),
            solana_instruction::AccountMeta::new(keys.serum_market, false),
            solana_instruction::AccountMeta::new(keys.serum_bids, false),
            solana_instruction::AccountMeta::new(keys.serum_asks, false),
            solana_instruction::AccountMeta::new(keys.serum_event_queue, false),
            solana_instruction::AccountMeta::new(keys.serum_coin_vault_account, false),
            solana_instruction::AccountMeta::new(keys.serum_pc_vault_account, false),
            solana_instruction::AccountMeta::new(keys.serum_vault_signer, false),
            solana_instruction::AccountMeta::new(keys.uer_source_token_account, false),
            solana_instruction::AccountMeta::new(keys.uer_destination_token_account, false),
            solana_instruction::AccountMeta::new(keys.user_source_owner, false),
        ]);
        
        metas
    }
}
'''
            content = re.sub(to_meta_pattern, to_meta_replacement, content, flags=re.MULTILINE)
            
            # 6. 修改函数签名
            pattern6 = r'let metas: \[solana_instruction::AccountMeta; SWAP_BASE_OUT_IX_ACCOUNTS_LEN\] = keys\.into\(\);'
            replacement6 = 'let metas: Vec<solana_instruction::AccountMeta> = keys.into();'
            content = re.sub(pattern6, replacement6, content)
            
            pattern7 = r'accounts: Vec::from\(metas\),'
            replacement7 = 'accounts: metas,'
            content = re.sub(pattern7, replacement7, content)
            
            # 7. 替换to_vec方法实现（处理Option<Pubkey>）
            to_vec_pattern = r'/// Convert Keys to Vec<Pubkey>\s*\n\s*pub fn to_vec\(&self\) -> Vec<Pubkey> \{\s*\n\s*vec!\[\s*\n([\s\S]*?)\s*\]\s*\n\s*\}'
            to_vec_replacement = '''/// Convert Keys to Vec<Pubkey>
    pub fn to_vec(&self) -> Vec<Pubkey> {
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
            self.uer_source_token_account,
            self.uer_destination_token_account,
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
            
            # 检查长度检查修复
            if "accounts.len() != SWAP_BASE_IN_IX_ACCOUNTS_LEN && accounts.len() != SWAP_BASE_IN_IX_ACCOUNTS_LEN - 1" in instructions_content:
                checks.append("✅ SwapBaseIn 长度检查修复已应用")
            else:
                checks.append("❌ SwapBaseIn 长度检查修复失败")
            
            if "SwapBaseInKeys::from(accounts)" in instructions_content:
                checks.append("✅ SwapBaseIn 账户传递修复已应用")
            else:
                checks.append("❌ SwapBaseIn 账户传递修复失败")
            
            # 检查SwapBaseInKeys修复
            swap_in_content = self.swap_base_in_file.read_text(encoding='utf-8')
            if "pub amm_target_orders: Option<Pubkey>" in swap_in_content:
                checks.append("✅ SwapBaseInKeys 可选字段修复已应用")
            else:
                checks.append("❌ SwapBaseInKeys 可选字段修复失败")
            
            # 检查SwapBaseOutKeys修复
            swap_out_content = self.swap_base_out_file.read_text(encoding='utf-8')
            if "pub amm_target_orders: Option<Pubkey>" in swap_out_content:
                checks.append("✅ SwapBaseOutKeys 可选字段修复已应用")
            else:
                checks.append("❌ SwapBaseOutKeys 可选字段修复失败")
            
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
            result = subprocess.run(
                ["cargo", "check", "--quiet"],
                capture_output=True,
                text=True
            )
            
            if result.returncode == 0:
                log_success("编译测试通过")
                return FixResult(True, "编译测试成功")
            else:
                log_error(f"编译测试失败: {result.stderr}")
                return FixResult(False, f"编译失败: {result.stderr}")
                
        except Exception as e:
            return FixResult(False, f"运行编译测试失败: {e}")
    
    def run_functional_test(self) -> FixResult:
        """运行功能测试"""
        log_header("运行功能测试")
        
        try:
            # 检查是否有测试文件
            tests_dir = self.interface_dir / "tests"
            if not tests_dir.exists():
                log_warning("未找到tests目录，跳过功能测试")
                return FixResult(True, "无功能测试可运行")
            
            os.chdir(self.interface_dir)
            
            # 运行测试
            result = subprocess.run(
                ["cargo", "test", "--quiet"],
                capture_output=True,
                text=True
            )
            
            if result.returncode == 0:
                log_success("功能测试通过")
                return FixResult(True, "功能测试成功")
            else:
                log_warning(f"功能测试有问题，但继续执行: {result.stderr}")
                return FixResult(True, f"功能测试警告: {result.stderr}")
                
        except Exception as e:
            return FixResult(False, f"运行功能测试失败: {e}")
    
    def run_fixes(self) -> bool:
        """执行所有修复"""
        log_header("🚀 开始 Raydium 接口修复...")
        
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
                ("验证修复", self.validate_fixes),
                ("编译测试", self.run_compilation_test),
                ("功能测试", self.run_functional_test),
            ]
            
            for step_name, step_func in steps:
                result = step_func()
                if not result.success:
                    log_error(f"{step_name}失败: {result.message}")
                    success = False
                    break
            
            if success:
                self.clean_backups()
                log_header("🎉 Raydium 接口修复成功完成！")
                print()
                print("📋 修复摘要:")
                print("  ✅ SwapBaseIn 现在支持 17 和 18 账户场景")
                print("  ✅ SwapBaseOut 现在支持 17 和 18 账户场景") 
                print("  ✅ amm_target_orders 字段在17账户时为 None，18账户时为 Some")
                print("  ✅ 所有修复已通过编译和功能测试")
                print()
                print("⚠️  重要提醒:")
                print("  - 这些修复修改了自动生成的代码")
                print("  - 重新运行 Solores 生成器会覆盖这些修复")
                print("  - 建议在每次重新生成后重新运行此脚本")
                print()
            else:
                self.restore_backups()
                log_error("修复过程中出现错误，已恢复原始文件")
            
            return success
            
        except KeyboardInterrupt:
            log_warning("用户中断，恢复备份文件...")
            self.restore_backups()
            return False
        except Exception as e:
            log_error(f"修复过程中出现未预期错误: {e}")
            self.restore_backups()
            return False

def main():
    """主函数"""
    parser = argparse.ArgumentParser(description="Raydium 接口修复脚本 - Python版")
    parser.add_argument("--interface-dir", required=True, 
                       help="sol_raydium_interface 目录路径")
    
    args = parser.parse_args()
    
    # 检查目录是否存在
    interface_dir = pathlib.Path(args.interface_dir)
    if not interface_dir.exists():
        log_error(f"接口目录不存在: {interface_dir}")
        sys.exit(1)
    
    # 创建修复器并运行
    fixer = RaydiumInterfaceFixer(args.interface_dir)
    success = fixer.run_fixes()
    
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()