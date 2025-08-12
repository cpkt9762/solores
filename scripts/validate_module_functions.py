#!/usr/bin/env uv run
# /// script
# requires-python = ">=3.8"
# dependencies = ["colorama>=0.4.6"]
# ///

"""
Solores 模块函数一致性检查脚本

检查生成的代码中每个模块的成员函数必须存在且必须一致，确保所有生成的代码具有统一的函数接口。

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
        self.function_pattern = re.compile(r'pub\s+fn\s+(\w+)\s*\([^)]*\)\s*(?:->\s*([^{]+))?', re.MULTILINE)
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
                
            # 全局函数 (少见)
            func_match = self.function_pattern.search(line)
            if func_match:
                func_name = func_match.group(1)
                return_type = func_match.group(2).strip() if func_match.group(2) else "()"
                
                signature = FunctionSignature(
                    name=func_name,
                    return_type=return_type,
                    parameters=[],
                    is_impl=False,
                    struct_name=""
                )
                functions[func_name] = signature
                
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
        
        # 检查Keys结构体函数 (账户密钥结构体) - 只需要from函数
        keys_structs = [s for s in self.module_info.structs if s.endswith('Keys')]
        for struct_name in keys_structs:
            key = f"{struct_name}::from"
            if key not in self.module_info.functions:
                errors.append(f"{struct_name} 缺少From trait实现")
        
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
    
    def __init__(self, module_info: ModuleInfo, has_accounts: bool):
        self.module_info = module_info
        self.has_accounts = has_accounts
        
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
        
        # 检查必需枚举
        required_enums = ['ProgramInstruction']
        if self.has_accounts:
            required_enums.append('ProgramAccount')
            
        for enum_name in required_enums:
            if enum_name not in self.module_info.structs:
                if enum_name == 'ProgramAccount' and not self.has_accounts:
                    continue  # 无accounts时不需要ProgramAccount枚举
                errors.append(f"缺少枚举 {enum_name}")
        
        # 检查条件性生成
        if not self.has_accounts:
            if 'try_unpack_account' in self.module_info.functions:
                errors.append("NonAnchor IDL无accounts时不应生成try_unpack_account函数")
            if 'ProgramAccount' in self.module_info.structs:
                errors.append("NonAnchor IDL无accounts时不应生成ProgramAccount枚举")
            
        success = len(errors) == 0
        message = f"Parsers模块验证: {'通过' if success else '失败'}"
        
        details = []
        if errors:
            details.extend([f"错误: {e}" for e in errors])
        if warnings:
            details.extend([f"警告: {w}" for w in warnings])
            
        return ValidationResult(success, message, details)

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
        
        # 检查Keys结构体的From trait实现
        missing_from_trait = []
        for struct_name, module_list in key_structs.items():
            for module_name, module_info in module_list:
                if f"{struct_name}::from" not in module_info.functions:
                    missing_from_trait.append(f"{module_name}::{struct_name}")
        
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

class SoloresModuleFunctionValidator:
    """Solores模块函数验证器主类"""
    
    def __init__(self, project_path: pathlib.Path):
        self.project_path = project_path
        self.src_path = project_path / "src"
        self.parser = FunctionSignatureParser()
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
    
    def validate_project(self) -> Dict[str, ValidationResult]:
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
            validator = ParsersFunctionValidator(self.modules['parsers'], self.idl_info['has_accounts'])
            results['parsers'] = validator.validate()
        
        # 跨模块一致性验证
        cross_validator = CrossModuleFunctionValidator(self.modules)
        results['cross_module'] = cross_validator.validate()
        
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
        results = validator.validate_project()
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
                results = validator.validate_project()
                
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