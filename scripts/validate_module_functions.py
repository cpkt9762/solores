#!/usr/bin/env uv run
# /// script
# requires-python = ">=3.8"
# dependencies = ["colorama>=0.4.6"]
# ///

"""
Solores 模块函数一致性检查脚本

检查生成的代码中每个模块的成员函数必须存在且必须一致，确保所有生成的代码具有统一的函数接口。
包括检查 Keys 结构体的 to_vec() 方法，用于将账户列表转换为 Vec<Pubkey>。

用法:
    python validate_module_functions.py --project path/to/generated/project
    python validate_module_functions.py --batch-dir path/to/batch/output --idl-dir path/to/idls
"""

import os
import sys
import re
import json
import argparse
import pathlib
from datetime import datetime
from typing import Dict, List, Set, Tuple, Optional, NamedTuple
from dataclasses import dataclass
from colorama import init, Fore, Style, Back

# 初始化colorama
init(autoreset=True)

class ValidationResult(NamedTuple):
    """验证结果"""
    passed: bool
    message: str
    details: List[str] = []

@dataclass
class FunctionSignature:
    """函数签名信息"""
    name: str
    return_type: str
    parameters: List[str]
    is_impl: bool = False
    struct_name: str = ""

@dataclass
class ModuleInfo:
    """模块信息"""
    name: str
    path: pathlib.Path
    exists: bool
    structs: Set[str]
    functions: Dict[str, FunctionSignature]
    constants: Set[str]

class Colors:
    """颜色定义"""
    SUCCESS = Fore.GREEN
    ERROR = Fore.RED
    WARNING = Fore.YELLOW
    INFO = Fore.BLUE
    HEADER = Fore.CYAN + Style.BRIGHT
    BOLD = Style.BRIGHT
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
    print(f"\n{Colors.HEADER}{'='*50}")
    print(f"{msg}")
    print(f"{'='*50}{Style.RESET_ALL}\n")

class FunctionSignatureParser:
    """改进的函数签名解析器"""
    
    def __init__(self):
        # 改进的正则表达式模式
        self.impl_pattern = re.compile(r'impl\s+(\w+)')
        self.impl_default_pattern = re.compile(r'impl\s+Default\s+for\s+(\w+)')
        self.impl_from_pattern = re.compile(r'impl\s+From<[^>]+>\s+for\s+(\w+)')
        self.function_pattern = re.compile(r'pub\s+fn\s+(\w+)\s*\([^)]*\)\s*(?:->\s*([^{]+?))?(?:\s*\{)', re.MULTILINE | re.DOTALL)
        self.const_pattern = re.compile(r'pub\s+const\s+(\w+):\s*([^=]+)=')
        self.struct_pattern = re.compile(r'pub\s+struct\s+(\w+)')
        self.enum_pattern = re.compile(r'pub\s+enum\s+(\w+)')
        
    def extract_functions_from_file(self, file_path: pathlib.Path) -> Tuple[Set[str], Dict[str, FunctionSignature], Set[str]]:
        """从文件中提取结构体、函数和常量"""
        if not file_path.exists():
            return set(), {}, set()
            
        try:
            content = file_path.read_text(encoding='utf-8')
            return self._parse_content(content)
        except Exception as e:
            log_error(f"解析文件失败 {file_path}: {e}")
            return set(), {}, set()
    
    def _parse_content(self, content: str) -> Tuple[Set[str], Dict[str, FunctionSignature], Set[str]]:
        """解析文件内容 - 改进版本"""
        structs = set()
        functions = {}
        constants = set()
        
        # 预处理：移除注释和清理多行
        cleaned_content = self._preprocess_content(content)
        
        # 按impl块分段解析
        impl_blocks = self._split_into_impl_blocks(cleaned_content)
        
        for block_type, block_content, struct_name in impl_blocks:
            if block_type == 'global':
                # 全局定义
                self._parse_global_definitions(block_content, structs, functions, constants)
            elif block_type == 'impl':
                # impl块内的函数
                self._parse_impl_block(block_content, struct_name, functions)
            elif block_type == 'impl_from':
                # From trait实现 - 等同于有from函数
                signature = FunctionSignature(
                    name='from',
                    return_type='Self',
                    parameters=['pubkeys: &[Pubkey]'],
                    is_impl=True,
                    struct_name=struct_name
                )
                key = f"{struct_name}::from"
                functions[key] = signature
            elif block_type == 'impl_default':
                # Default trait实现 - 等同于有default函数
                signature = FunctionSignature(
                    name='default',
                    return_type='Self',
                    parameters=[],
                    is_impl=True,
                    struct_name=struct_name
                )
                key = f"{struct_name}::default"
                functions[key] = signature
                
        return structs, functions, constants
    
    def _preprocess_content(self, content: str) -> str:
        """预处理内容：移除注释，处理多行函数定义"""
        lines = content.split('\n')
        cleaned_lines = []
        
        for line in lines:
            # 移除行注释
            if '//' in line:
                line = line[:line.index('//')]
            cleaned_lines.append(line)
            
        return '\n'.join(cleaned_lines)
    
    def _split_into_impl_blocks(self, content: str) -> List[Tuple[str, str, str]]:
        """将内容分割为impl块"""
        blocks = []
        lines = content.split('\n')
        current_block = []
        current_type = 'global'
        current_struct = ''
        brace_depth = 0
        
        for line in lines:
            line = line.strip()
            if not line:
                continue
                
            # 检查impl块开始 - 优先级顺序很重要
            impl_default_match = self.impl_default_pattern.search(line)
            impl_from_match = self.impl_from_pattern.search(line)
            impl_match = self.impl_pattern.search(line)
            
            if impl_from_match:
                # 完成当前块
                if current_block:
                    blocks.append((current_type, '\n'.join(current_block), current_struct))
                    current_block = []
                
                current_type = 'impl_from'
                current_struct = impl_from_match.group(1)
                brace_depth = 0
                
            elif impl_default_match:
                # 完成当前块
                if current_block:
                    blocks.append((current_type, '\n'.join(current_block), current_struct))
                    current_block = []
                
                current_type = 'impl_default'
                current_struct = impl_default_match.group(1)
                brace_depth = 0
                
            elif impl_match:
                # 完成当前块
                if current_block:
                    blocks.append((current_type, '\n'.join(current_block), current_struct))
                    current_block = []
                
                current_type = 'impl'
                current_struct = impl_match.group(1)
                brace_depth = 0
                
            # 统计大括号深度
            brace_depth += line.count('{') - line.count('}')
            current_block.append(line)
            
            # impl块结束
            if current_type in ['impl', 'impl_from', 'impl_default'] and brace_depth == 0 and '}' in line:
                blocks.append((current_type, '\n'.join(current_block), current_struct))
                current_block = []
                current_type = 'global'
                current_struct = ''
        
        # 添加最后一个块
        if current_block:
            blocks.append((current_type, '\n'.join(current_block), current_struct))
            
        return blocks
    
    def _parse_global_definitions(self, content: str, structs: Set[str], functions: Dict[str, FunctionSignature], constants: Set[str]):
        """解析全局定义"""
        # 首先处理整个内容的函数（支持跨多行）
        func_matches = self.function_pattern.findall(content)
        for func_name, return_type in func_matches:
            if not return_type:
                return_type = "()"
            return_type = return_type.strip()
            
            signature = FunctionSignature(
                name=func_name,
                return_type=return_type,
                parameters=[],
                is_impl=False,
                struct_name=""
            )
            functions[func_name] = signature
        
        # 然后按行处理其他定义
        lines = content.split('\n')
        
        for line in lines:
            line = line.strip()
            
            # 结构体定义
            struct_match = self.struct_pattern.search(line)
            if struct_match:
                structs.add(struct_match.group(1))
                continue
                
            # 枚举定义
            enum_match = self.enum_pattern.search(line)
            if enum_match:
                structs.add(enum_match.group(1))
                continue
            
            # 常量定义
            const_match = self.const_pattern.search(line)
            if const_match:
                constants.add(const_match.group(1))
                continue
                
    def _parse_impl_block(self, content: str, struct_name: str, functions: Dict[str, FunctionSignature]):
        """解析impl块内的函数"""
        # 将整个impl块作为一个整体处理多行函数
        func_matches = self.function_pattern.findall(content)
        
        for func_name, return_type in func_matches:
            if not return_type:
                return_type = "()"
            return_type = return_type.strip()
            
            # 特殊处理一些已知的函数签名
            if func_name == 'from_bytes':
                return_type = 'Result<Self, std::io::Error>'
            elif func_name == 'try_to_vec':
                return_type = 'std::io::Result<Vec<u8>>'
            elif func_name == 'default':
                return_type = 'Self'
            
            signature = FunctionSignature(
                name=func_name,
                return_type=return_type,
                parameters=[],
                is_impl=True,
                struct_name=struct_name
            )
            
            key = f"{struct_name}::{func_name}"
            functions[key] = signature

class InstructionsFunctionValidator:
    """Instructions模块函数验证器"""
    
    def __init__(self, module_info: ModuleInfo):
        self.module_info = module_info
        
    def validate(self) -> ValidationResult:
        """验证Instructions模块"""
        if not self.module_info.exists:
            return ValidationResult(False, "Instructions模块不存在")
            
        errors = []
        warnings = []
        
        # 检查IxData结构体函数 (指令数据结构体)
        ixdata_structs = [s for s in self.module_info.structs if s.endswith('IxData')]
        for struct_name in ixdata_structs:
            # 检查必需函数
            required_functions = [
                ('try_to_vec', 'std::io::Result<Vec<u8>>'),
                ('from_bytes', 'Result<Self, std::io::Error>'),
                ('default', 'Self')  # Default trait
            ]
            
            for func_name, expected_return in required_functions:
                key = f"{struct_name}::{func_name}"
                if key not in self.module_info.functions:
                    errors.append(f"{struct_name} 缺少函数 {func_name}()")
                else:
                    func = self.module_info.functions[key]
                    if not self._check_return_type_compatible(func.return_type, expected_return):
                        warnings.append(f"{struct_name}::{func_name} 返回类型不一致: 期望 {expected_return}, 实际 {func.return_type}")
        
        # 检查Keys结构体函数 (账户密钥结构体) - 需要from函数和to_vec方法
        keys_structs = [s for s in self.module_info.structs if s.endswith('Keys')]
        for struct_name in keys_structs:
            # 检查From trait实现
            key = f"{struct_name}::from"
            if key not in self.module_info.functions:
                errors.append(f"{struct_name} 缺少From trait实现")
            
            # 检查to_vec方法
            to_vec_key = f"{struct_name}::to_vec"
            if to_vec_key not in self.module_info.functions:
                errors.append(f"{struct_name} 缺少to_vec()方法")
            else:
                # 验证返回类型
                func = self.module_info.functions[to_vec_key]
                if not self._check_to_vec_return_type(func.return_type):
                    warnings.append(f"{struct_name}::to_vec 返回类型不正确: 期望 Vec<Pubkey>, 实际 {func.return_type}")
        
        # 检查discriminator常量
        discm_constants = [c for c in self.module_info.constants if c.endswith('_IX_DISCM')]
        if not discm_constants:
            warnings.append("未找到指令discriminator常量")
        
        # 检查账户长度常量
        len_constants = [c for c in self.module_info.constants if c.endswith('_IX_ACCOUNTS_LEN')]
        if not len_constants:
            warnings.append("未找到账户长度常量")
            
        success = len(errors) == 0
        message = f"Instructions模块验证: {'通过' if success else '失败'}"
        
        details = []
        if errors:
            details.extend([f"错误: {e}" for e in errors])
        if warnings:
            details.extend([f"警告: {w}" for w in warnings])
            
        return ValidationResult(success, message, details)
    
    def _check_return_type_compatible(self, actual: str, expected: str) -> bool:
        """检查返回类型是否兼容"""
        # 简化的类型兼容检查
        actual = actual.strip()
        expected = expected.strip()
        
        # 处理Self类型
        if 'Self' in expected:
            return 'Self' in actual or actual.endswith('Error>')
            
        return actual == expected or actual.replace(' ', '') == expected.replace(' ', '')
    
    def _check_to_vec_return_type(self, return_type: str) -> bool:
        """检查to_vec返回类型是否为Vec<Pubkey>"""
        normalized = return_type.strip().replace(' ', '')
        return normalized == 'Vec<Pubkey>' or normalized == 'Vec<solana_pubkey::Pubkey>'

class AccountsFunctionValidator:
    """Accounts模块函数验证器"""
    
    def __init__(self, module_info: ModuleInfo, has_accounts: bool):
        self.module_info = module_info
        self.has_accounts = has_accounts
        
    def validate(self) -> ValidationResult:
        """验证Accounts模块"""
        if not self.has_accounts:
            if self.module_info.exists:
                return ValidationResult(False, "NonAnchor IDL无accounts时不应生成accounts模块")
            else:
                return ValidationResult(True, "Accounts模块正确跳过生成 (IDL无accounts字段)")
        
        if not self.module_info.exists:
            return ValidationResult(False, "Accounts模块应该存在但未生成")
            
        errors = []
        warnings = []
        
        # 检查账户结构体函数
        account_structs = [s for s in self.module_info.structs if not s.endswith('Keys') and not s.endswith('IxData')]
        
        for struct_name in account_structs:
            # 检查必需函数
            required_functions = [
                ('try_to_vec', 'std::io::Result<Vec<u8>>'),
                ('from_bytes', 'Result<Self, std::io::Error>'),
                ('default', 'Self')  # Default trait
            ]
            
            for func_name, expected_return in required_functions:
                if func_name == 'default':
                    key = f"{struct_name}::default"  # Default trait impl
                else:
                    key = f"{struct_name}::{func_name}"
                    
                if key not in self.module_info.functions:
                    errors.append(f"{struct_name} 缺少函数 {func_name}()")
        
        # 检查discriminator常量 (Anchor合约)
        discm_constants = [c for c in self.module_info.constants if c.endswith('_ACCOUNT_DISCM')]
        if not discm_constants:
            warnings.append("未找到账户discriminator常量 (可能是NonAnchor合约)")
            
        success = len(errors) == 0
        message = f"Accounts模块验证: {'通过' if success else '失败'}"
        
        details = []
        if errors:
            details.extend([f"错误: {e}" for e in errors])
        if warnings:
            details.extend([f"警告: {w}" for w in warnings])
            
        return ValidationResult(success, message, details)

class EventsFunctionValidator:
    """Events模块函数验证器"""
    
    def __init__(self, module_info: ModuleInfo):
        self.module_info = module_info
        
    def validate(self) -> ValidationResult:
        """验证Events模块"""
        if not self.module_info.exists:
            return ValidationResult(True, "Events模块不存在 (IDL无events)")
            
        errors = []
        warnings = []
        
        # 检查事件结构体函数
        event_structs = [s for s in self.module_info.structs if not s.endswith('Keys') and not s.endswith('IxData')]
        
        for struct_name in event_structs:
            # 检查必需函数
            required_functions = [
                ('try_to_vec', 'std::io::Result<Vec<u8>>'),
                ('from_bytes', 'Result<Self, std::io::Error>'),
                ('default', 'Self')  # Default trait
            ]
            
            for func_name, expected_return in required_functions:
                if func_name == 'default':
                    key = f"{struct_name}::default"
                else:
                    key = f"{struct_name}::{func_name}"
                    
                if key not in self.module_info.functions:
                    errors.append(f"{struct_name} 缺少函数 {func_name}()")
        
        # 检查事件discriminator常量
        discm_constants = [c for c in self.module_info.constants if c.endswith('_EVENT_DISCM')]
        if event_structs and not discm_constants:
            warnings.append("事件结构体存在但未找到事件discriminator常量")
            
        success = len(errors) == 0
        message = f"Events模块验证: {'通过' if success else '失败'}"
        
        details = []
        if errors:
            details.extend([f"错误: {e}" for e in errors])
        if warnings:
            details.extend([f"警告: {w}" for w in warnings])
            
        return ValidationResult(success, message, details)

class ParsersFunctionValidator:
    """Parsers模块函数验证器"""
    
    def __init__(self, module_info: ModuleInfo, has_accounts: bool, program_name: str = ""):
        self.module_info = module_info
        self.has_accounts = has_accounts
        self.program_name = program_name
        
    def validate(self) -> ValidationResult:
        """验证Parsers模块"""
        if not self.module_info.exists:
            return ValidationResult(False, "Parsers模块不存在")
            
        errors = []
        warnings = []
        
        # 检查必需函数
        required_functions = [
            'parse_instruction'
        ]
        
        if self.has_accounts:
            required_functions.append('try_unpack_account')
        
        for func_name in required_functions:
            if func_name not in self.module_info.functions:
                errors.append(f"缺少函数 {func_name}()")
        
        # 检查必需枚举 - 支持带项目名前缀的命名
        # 转换程序名称为PascalCase
        program_pascal = self._to_pascal_case(self.program_name)
        
        required_enums = [
            'ProgramInstruction',
            f'{program_pascal}Instruction'
        ]
        if self.has_accounts:
            required_enums.extend([
                'ProgramAccount',
                f'{program_pascal}Account'
            ])
        
        # 检查是否至少有一个变体存在
        instruction_enum_found = False
        for enum_name in ['ProgramInstruction', f'{program_pascal}Instruction']:
            if enum_name in self.module_info.structs:
                instruction_enum_found = True
                break
        
        if not instruction_enum_found:
            errors.append(f"缺少枚举 ProgramInstruction 或 {program_pascal}Instruction")
        
        if self.has_accounts:
            account_enum_found = False
            for enum_name in ['ProgramAccount', f'{program_pascal}Account']:
                if enum_name in self.module_info.structs:
                    account_enum_found = True
                    break
            if not account_enum_found:
                errors.append(f"缺少枚举 ProgramAccount 或 {program_pascal}Account")
        
        # 检查条件性生成
        if not self.has_accounts:
            if 'try_unpack_account' in self.module_info.functions:
                errors.append("NonAnchor IDL无accounts时不应生成try_unpack_account函数")
            for enum_name in ['ProgramAccount', f'{program_pascal}Account']:
                if enum_name in self.module_info.structs:
                    errors.append(f"NonAnchor IDL无accounts时不应生成{enum_name}枚举")
            
        success = len(errors) == 0
        message = f"Parsers模块验证: {'通过' if success else '失败'}"
        
        details = []
        if errors:
            details.extend([f"错误: {e}" for e in errors])
        if warnings:
            details.extend([f"警告: {w}" for w in warnings])
            
        return ValidationResult(success, message, details)
    
    def _to_pascal_case(self, name: str) -> str:
        """将字符串转换为PascalCase"""
        if not name:
            return ""
        
        # 移除非字母数字字符，分割单词
        import re
        words = re.findall(r'[a-zA-Z0-9]+', name)
        
        # 转换为PascalCase
        return ''.join(word.capitalize() for word in words)

class CrossModuleFunctionValidator:
    """跨模块函数一致性验证器"""
    
    def __init__(self, modules: Dict[str, ModuleInfo]):
        self.modules = modules
        
    def validate(self) -> ValidationResult:
        """验证跨模块函数一致性"""
        errors = []
        warnings = []
        
        # 收集需要序列化函数的结构体 (排除Keys结构体)
        data_structs = {}  # 数据结构体：需要try_to_vec, from_bytes, default
        key_structs = {}   # Keys结构体：只需要from (通过From trait)
        
        for module_name, module_info in self.modules.items():
            if module_info.exists:
                for struct_name in module_info.structs:
                    if struct_name.endswith('Keys'):
                        # Keys结构体只需要From trait
                        if struct_name not in key_structs:
                            key_structs[struct_name] = []
                        key_structs[struct_name].append((module_name, module_info))
                    elif self._is_parser_enum(struct_name):
                        # Parser枚举不需要序列化函数，跳过
                        continue
                    else:
                        # 数据结构体需要完整的序列化接口
                        if struct_name not in data_structs:
                            data_structs[struct_name] = []
                        data_structs[struct_name].append((module_name, module_info))
        
        # 检查数据结构体的序列化函数一致性
        missing_try_to_vec = []
        missing_from_bytes = []
        missing_default = []
        
        for struct_name, module_list in data_structs.items():
            for module_name, module_info in module_list:
                # 检查try_to_vec
                if f"{struct_name}::try_to_vec" not in module_info.functions:
                    missing_try_to_vec.append(f"{module_name}::{struct_name}")
                
                # 检查from_bytes
                if f"{struct_name}::from_bytes" not in module_info.functions:
                    missing_from_bytes.append(f"{module_name}::{struct_name}")
                
                # 检查Default实现
                if f"{struct_name}::default" not in module_info.functions:
                    missing_default.append(f"{module_name}::{struct_name}")
        
        # 检查Keys结构体的From trait实现和to_vec方法
        missing_from_trait = []
        missing_to_vec = []
        for struct_name, module_list in key_structs.items():
            for module_name, module_info in module_list:
                if f"{struct_name}::from" not in module_info.functions:
                    missing_from_trait.append(f"{module_name}::{struct_name}")
                
                # 检查to_vec方法
                if f"{struct_name}::to_vec" not in module_info.functions:
                    missing_to_vec.append(f"{module_name}::{struct_name}")
        
        # 报告错误
        if missing_try_to_vec:
            errors.append(f"数据结构体缺少try_to_vec函数: {', '.join(missing_try_to_vec[:3])}" + 
                         (f" 等{len(missing_try_to_vec)}个" if len(missing_try_to_vec) > 3 else ""))
        
        if missing_from_bytes:
            errors.append(f"数据结构体缺少from_bytes函数: {', '.join(missing_from_bytes[:3])}" +
                         (f" 等{len(missing_from_bytes)}个" if len(missing_from_bytes) > 3 else ""))
        
        if missing_from_trait:
            errors.append(f"Keys结构体缺少From trait实现: {', '.join(missing_from_trait[:3])}" +
                         (f" 等{len(missing_from_trait)}个" if len(missing_from_trait) > 3 else ""))
        
        if missing_to_vec:
            errors.append(f"Keys结构体缺少to_vec()方法: {', '.join(missing_to_vec[:3])}" +
                         (f" 等{len(missing_to_vec)}个" if len(missing_to_vec) > 3 else ""))
        
        if missing_default:
            warnings.append(f"缺少Default实现的结构体: {', '.join(missing_default[:3])}" +
                           (f" 等{len(missing_default)}个" if len(missing_default) > 3 else ""))
        
        success = len(errors) == 0
        message = f"跨模块一致性验证: {'通过' if success else '失败'}"
        
        details = []
        if errors:
            details.extend([f"错误: {e}" for e in errors])
        if warnings:
            details.extend([f"警告: {w}" for w in warnings])
        
        # 添加统计信息
        details.append(f"数据结构体总数: {sum(len(v) for v in data_structs.values())}")
        details.append(f"Keys结构体总数: {sum(len(v) for v in key_structs.values())}")
            
        return ValidationResult(success, message, details)
    
    def _is_parser_enum(self, struct_name: str) -> bool:
        """判断是否为parser枚举类型"""
        # Parser枚举通常以这些模式命名：
        # ProgramInstruction, ProgramAccount, {ProjectName}Instruction, {ProjectName}Account
        return (
            struct_name == 'ProgramInstruction' or
            struct_name == 'ProgramAccount' or
            struct_name.endswith('Instruction') or
            struct_name.endswith('Account')
        )

class NamingConventionValidator:
    """Rust命名约定检测器"""
    
    def __init__(self):
        # snake_case 正则：小写字母开头，可包含下划线和数字
        self.snake_case_pattern = re.compile(r'^[a-z][a-z0-9_]*$')
        # PascalCase 正则：大写字母开头，后续为字母数字
        self.pascal_case_pattern = re.compile(r'^[A-Z][a-zA-Z0-9]*$')
        # camelCase 正则：小写字母开头，后续可有大写字母
        self.camel_case_pattern = re.compile(r'^[a-z][a-zA-Z0-9]*$')
        
        # 字段定义模式：pub field_name: Type
        self.field_pattern = re.compile(r'pub\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*:\s*[^,\n}]+[,\n}]')
        # 结构体定义模式：pub struct StructName
        self.struct_name_pattern = re.compile(r'pub\s+struct\s+([a-zA-Z_][a-zA-Z0-9_]*)')
        # 枚举定义模式：pub enum EnumName
        self.enum_name_pattern = re.compile(r'pub\s+enum\s+([a-zA-Z_][a-zA-Z0-9_]*)')
        # 函数定义模式：pub fn function_name
        self.function_name_pattern = re.compile(r'pub\s+fn\s+([a-zA-Z_][a-zA-Z0-9_]*)')
        # 变量声明模式：let variable_name = (不包括模式匹配)
        # 排除 if let 和复杂模式匹配，只匹配简单的 let var = 
        self.variable_pattern = re.compile(r'(?<!if\s)let\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=')
        
    def detect_naming_violations(self, file_path: pathlib.Path) -> Dict[str, List[str]]:
        """检测命名约定违规"""
        violations = {
            'non_snake_case_fields': [],
            'non_snake_case_functions': [], 
            'non_snake_case_variables': [],
            'non_pascal_case_types': [],
            'camel_case_found': []  # 意外的camelCase使用
        }
        
        if not file_path.exists():
            return violations
            
        try:
            content = file_path.read_text(encoding='utf-8')
        except Exception as e:
            log_warning(f"读取文件失败 {file_path}: {e}")
            return violations
        
        # 检测字段名
        for match in self.field_pattern.finditer(content):
            field_name = match.group(1)
            # 跳过一些特殊字段名
            if field_name in ['Self', 'self', '_phantom']:
                continue
                
            if not self.snake_case_pattern.match(field_name):
                if self.camel_case_pattern.match(field_name) and any(c.isupper() for c in field_name[1:]):
                    violations['camel_case_found'].append(f"字段 {field_name}")
                else:
                    violations['non_snake_case_fields'].append(field_name)
        
        # 检测函数名  
        for match in self.function_name_pattern.finditer(content):
            func_name = match.group(1)
            # 跳过一些特殊函数名
            if func_name in ['new', 'default', 'clone', 'from']:
                continue
                
            if not self.snake_case_pattern.match(func_name):
                violations['non_snake_case_functions'].append(func_name)
        
        # 检测结构体名
        for match in self.struct_name_pattern.finditer(content):
            struct_name = match.group(1) 
            if not self.pascal_case_pattern.match(struct_name):
                violations['non_pascal_case_types'].append(f"struct {struct_name}")
        
        # 检测枚举名
        for match in self.enum_name_pattern.finditer(content):
            enum_name = match.group(1)
            if not self.pascal_case_pattern.match(enum_name):
                violations['non_pascal_case_types'].append(f"enum {enum_name}")
        
        # 检测变量名（在函数体内）
        for match in self.variable_pattern.finditer(content):
            var_name = match.group(1)
            # 跳过下划线开头的变量（通常是有意未使用的）
            if var_name.startswith('_'):
                continue
                
            if not self.snake_case_pattern.match(var_name):
                violations['non_snake_case_variables'].append(var_name)
        
        return violations
    
    def validate_module_naming(self, module_info: ModuleInfo) -> ValidationResult:
        """验证模块的命名约定"""
        if not module_info.exists:
            return ValidationResult(True, f"{module_info.name}模块不存在，跳过命名检查")
        
        all_violations = {}
        
        if module_info.path.is_file():
            # 单文件模块
            violations = self.detect_naming_violations(module_info.path)
            if any(violations.values()):
                all_violations[module_info.path.name] = violations
        else:
            # 目录模块
            for rs_file in module_info.path.glob("*.rs"):
                if rs_file.name == "mod.rs":
                    continue
                violations = self.detect_naming_violations(rs_file)
                if any(violations.values()):
                    all_violations[rs_file.name] = violations
        
        # 生成验证结果
        errors = []
        warnings = []
        
        for file_name, violations in all_violations.items():
            if violations['non_snake_case_fields']:
                errors.append(f"{file_name}: 字段名违反snake_case: {', '.join(violations['non_snake_case_fields'][:3])}" + 
                             (f" 等{len(violations['non_snake_case_fields'])}个" if len(violations['non_snake_case_fields']) > 3 else ""))
            
            if violations['non_snake_case_functions']:
                errors.append(f"{file_name}: 函数名违反snake_case: {', '.join(violations['non_snake_case_functions'][:3])}" +
                             (f" 等{len(violations['non_snake_case_functions'])}个" if len(violations['non_snake_case_functions']) > 3 else ""))
            
            if violations['non_pascal_case_types']:
                errors.append(f"{file_name}: 类型名违反PascalCase: {', '.join(violations['non_pascal_case_types'][:3])}" +
                             (f" 等{len(violations['non_pascal_case_types'])}个" if len(violations['non_pascal_case_types']) > 3 else ""))
            
            if violations['non_snake_case_variables']:
                warnings.append(f"{file_name}: 变量名违反snake_case: {', '.join(violations['non_snake_case_variables'][:3])}" +
                               (f" 等{len(violations['non_snake_case_variables'])}个" if len(violations['non_snake_case_variables']) > 3 else ""))
            
            if violations['camel_case_found']:
                warnings.append(f"{file_name}: 发现camelCase字段: {', '.join(violations['camel_case_found'][:3])}" +
                               (f" 等{len(violations['camel_case_found'])}个" if len(violations['camel_case_found']) > 3 else ""))
        
        success = len(errors) == 0
        message = f"{module_info.name}模块命名约定: {'通过' if success else '发现问题'}"
        
        details = []
        if errors:
            details.extend([f"错误: {e}" for e in errors])
        if warnings:
            details.extend([f"警告: {w}" for w in warnings])
        
        return ValidationResult(success, message, details)

class SoloresModuleFunctionValidator:
    """Solores模块函数验证器主类"""
    
    def __init__(self, project_path: pathlib.Path):
        self.project_path = project_path
        self.src_path = project_path / "src"
        self.parser = FunctionSignatureParser()
        self.naming_validator = NamingConventionValidator()
        self.modules = {}
        
        # 检查IDL类型
        self.idl_info = self._detect_idl_type()
        
    def _detect_idl_type(self) -> Dict:
        """检测IDL类型和特征"""
        idl_path = self.project_path / "idl.json"
        if not idl_path.exists():
            return {"type": "unknown", "has_accounts": False}
        
        try:
            with open(idl_path, 'r') as f:
                idl_data = json.load(f)
            
            # 检查是否为Anchor
            is_anchor = False
            if 'metadata' in idl_data and idl_data['metadata'].get('description', '').endswith('Created with Anchor'):
                is_anchor = True
            
            # 检查指令discriminator长度
            if 'instructions' in idl_data and idl_data['instructions']:
                first_instruction = idl_data['instructions'][0]
                if 'discriminator' in first_instruction:
                    if len(first_instruction['discriminator']) == 8:
                        is_anchor = True
            
            # 检查是否有accounts
            has_accounts = bool(idl_data.get('accounts', []))
            
            return {
                "type": "anchor" if is_anchor else "non_anchor",
                "has_accounts": has_accounts,
                "program_name": idl_data.get('metadata', {}).get('name', 'unknown')
            }
        except Exception as e:
            log_warning(f"解析IDL文件失败: {e}")
            return {"type": "unknown", "has_accounts": False}
    
    def scan_modules(self):
        """扫描所有模块"""
        module_names = ['instructions', 'accounts', 'types', 'events', 'parsers']
        
        for module_name in module_names:
            module_path = self.src_path / module_name
            
            if module_name == 'errors':
                # errors是单文件
                module_file = self.src_path / "errors.rs"
                structs, functions, constants = self.parser.extract_functions_from_file(module_file)
                self.modules[module_name] = ModuleInfo(
                    name=module_name,
                    path=module_file,
                    exists=module_file.exists(),
                    structs=structs,
                    functions=functions,
                    constants=constants
                )
            else:
                # 其他是目录模块
                if module_path.exists():
                    all_structs = set()
                    all_functions = {}
                    all_constants = set()
                    
                    # 扫描模块目录下的所有.rs文件
                    for rs_file in module_path.glob("*.rs"):
                        if rs_file.name == "mod.rs":
                            continue
                        structs, functions, constants = self.parser.extract_functions_from_file(rs_file)
                        all_structs.update(structs)
                        all_functions.update(functions)
                        all_constants.update(constants)
                    
                    self.modules[module_name] = ModuleInfo(
                        name=module_name,
                        path=module_path,
                        exists=True,
                        structs=all_structs,
                        functions=all_functions,
                        constants=all_constants
                    )
                else:
                    self.modules[module_name] = ModuleInfo(
                        name=module_name,
                        path=module_path,
                        exists=False,
                        structs=set(),
                        functions={},
                        constants=set()
                    )
    
    def validate_project(self, check_naming: bool = False, strict_naming: bool = False) -> Dict[str, ValidationResult]:
        """验证整个项目"""
        self.scan_modules()
        
        results = {}
        
        # 验证Instructions模块
        if 'instructions' in self.modules:
            validator = InstructionsFunctionValidator(self.modules['instructions'])
            results['instructions'] = validator.validate()
        
        # 验证Accounts模块
        if 'accounts' in self.modules:
            validator = AccountsFunctionValidator(self.modules['accounts'], self.idl_info['has_accounts'])
            results['accounts'] = validator.validate()
        
        # 验证Events模块
        if 'events' in self.modules:
            validator = EventsFunctionValidator(self.modules['events'])
            results['events'] = validator.validate()
        
        # 验证Parsers模块
        if 'parsers' in self.modules:
            program_name = self.idl_info.get('program_name', '')
            validator = ParsersFunctionValidator(self.modules['parsers'], self.idl_info['has_accounts'], program_name)
            results['parsers'] = validator.validate()
        
        # 跨模块一致性验证
        cross_validator = CrossModuleFunctionValidator(self.modules)
        results['cross_module'] = cross_validator.validate()
        
        # 命名约定验证（可选）
        if check_naming:
            for module_name in ['instructions', 'accounts', 'types', 'events', 'parsers']:
                if module_name in self.modules:
                    naming_result = self.naming_validator.validate_module_naming(self.modules[module_name])
                    
                    # 严格模式：将警告视为错误
                    if strict_naming:
                        # 如果存在警告，将其视为失败
                        has_warnings = any('警告:' in detail for detail in naming_result.details)
                        if has_warnings and naming_result.passed:
                            naming_result = ValidationResult(
                                False, 
                                naming_result.message.replace('通过', '严格模式失败'),
                                naming_result.details
                            )
                    
                    results[f'{module_name}_naming'] = naming_result
        
        return results
    
    def print_detailed_report(self, results: Dict[str, ValidationResult]):
        """打印详细报告"""
        log_header(f"🔍 Solores模块函数一致性验证报告")
        
        log_info(f"项目路径: {self.project_path}")
        log_info(f"IDL类型: {self.idl_info.get('type', 'unknown').upper()}")
        log_info(f"程序名称: {self.idl_info.get('program_name', 'unknown')}")
        log_info(f"包含accounts: {'是' if self.idl_info.get('has_accounts') else '否'}")
        log_info(f"验证时间: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        
        print()
        
        total_passed = 0
        total_tests = len(results)
        
        for test_name, result in results.items():
            if result.passed:
                log_success(f"{test_name.title()}模块: {result.message}")
                total_passed += 1
            else:
                log_error(f"{test_name.title()}模块: {result.message}")
            
            # 显示详细信息
            for detail in result.details:
                if detail.startswith("错误"):
                    print(f"  {Colors.ERROR}  {detail}{Style.RESET_ALL}")
                elif detail.startswith("警告"):
                    print(f"  {Colors.WARNING}  {detail}{Style.RESET_ALL}")
                else:
                    print(f"  {Colors.INFO}  {detail}{Style.RESET_ALL}")
        
        print()
        
        # 总结
        if total_passed == total_tests:
            log_success(f"🎯 验证结果: {total_passed}/{total_tests} 通过 - 完全符合函数接口标准!")
        else:
            log_error(f"🎯 验证结果: {total_passed}/{total_tests} 通过 - 存在函数接口问题")
        
        # 模块统计
        log_header("📊 模块统计信息")
        for module_name, module_info in self.modules.items():
            if module_info.exists:
                log_info(f"{module_name.title()}: {len(module_info.structs)}个结构体, {len(module_info.functions)}个函数, {len(module_info.constants)}个常量")
            else:
                log_info(f"{module_name.title()}: 未生成")

def main():
    parser = argparse.ArgumentParser(description="Solores模块函数一致性检查工具")
    parser.add_argument("--project", type=str, help="生成项目路径")
    parser.add_argument("--batch-dir", type=str, help="批量验证目录")
    parser.add_argument("--idl-dir", type=str, help="IDL文件目录")
    parser.add_argument("--check-naming", action="store_true", help="启用命名约定检查")
    parser.add_argument("--strict-naming", action="store_true", help="严格命名约定检查（将警告视为错误）")
    
    args = parser.parse_args()
    
    if not args.project and not args.batch_dir:
        parser.print_help()
        sys.exit(1)
    
    if args.project:
        # 单项目验证
        project_path = pathlib.Path(args.project)
        if not project_path.exists():
            log_error(f"项目路径不存在: {project_path}")
            sys.exit(1)
        
        validator = SoloresModuleFunctionValidator(project_path)
        results = validator.validate_project(
            check_naming=args.check_naming,
            strict_naming=args.strict_naming
        )
        validator.print_detailed_report(results)
        
        # 检查是否所有测试都通过
        all_passed = all(result.passed for result in results.values())
        sys.exit(0 if all_passed else 1)
    
    elif args.batch_dir:
        # 批量验证
        batch_dir = pathlib.Path(args.batch_dir)
        if not batch_dir.exists():
            log_error(f"批量目录不存在: {batch_dir}")
            sys.exit(1)
        
        log_header("🔄 批量验证模式")
        
        total_projects = 0
        passed_projects = 0
        
        for project_dir in batch_dir.iterdir():
            if project_dir.is_dir() and (project_dir / "Cargo.toml").exists():
                total_projects += 1
                log_info(f"验证项目: {project_dir.name}")
                
                validator = SoloresModuleFunctionValidator(project_dir)
                results = validator.validate_project(
                    check_naming=args.check_naming,
                    strict_naming=args.strict_naming
                )
                
                all_passed = all(result.passed for result in results.values())
                if all_passed:
                    passed_projects += 1
                    log_success(f"✅ {project_dir.name}")
                else:
                    log_error(f"❌ {project_dir.name}")
                    # 显示失败详情
                    for test_name, result in results.items():
                        if not result.passed:
                            print(f"    {Colors.ERROR}{test_name}: {result.message}{Style.RESET_ALL}")
        
        print()
        log_header(f"📊 批量验证总结: {passed_projects}/{total_projects} 项目通过")
        
        sys.exit(0 if passed_projects == total_projects else 1)

if __name__ == "__main__":
    main()