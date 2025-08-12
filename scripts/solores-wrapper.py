#!/usr/bin/env uv run
# /// script
# requires-python = ">=3.8"
# dependencies = []
# ///

"""
Solores智能包装器 - 确保每次使用最新的二进制文件
支持自动检测raydium.json并应用修复脚本
"""

import os
import sys
import subprocess
import pathlib
import json
from datetime import datetime
import time

# 颜色输出
class Colors:
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    NC = '\033[0m'  # No Color

def log_info(msg: str):
    print(f"{Colors.GREEN}[INFO]{Colors.NC} {msg}")

def log_warn(msg: str):
    print(f"{Colors.YELLOW}[WARN]{Colors.NC} {msg}")

def log_error(msg: str):
    print(f"{Colors.RED}[ERROR]{Colors.NC} {msg}")

def log_debug(msg: str):
    print(f"{Colors.BLUE}[DEBUG]{Colors.NC} {msg}")

class SoloresWrapper:
    def __init__(self):
        self.script_dir = pathlib.Path(__file__).parent
        self.project_root = self.script_dir.parent
        self.solores_bin = self.project_root / "target" / "release" / "solores"
        self.src_dir = self.project_root / "solores" / "src"
        self.cargo_toml = self.project_root / "solores" / "Cargo.toml"
        self.fix_raydium_script = self.script_dir / "fix_raydium_interface.py"
        
        # Raydium程序地址 - 仅修复这个特定的IDL
        self.RAYDIUM_PROGRAM_ADDRESS = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
    
    def check_build_needed(self) -> bool:
        """检查是否需要重新构建"""
        
        # 检查二进制文件是否存在
        if not self.solores_bin.exists():
            log_warn(f"二进制文件不存在: {self.solores_bin}")
            return True
        
        binary_mtime = self.solores_bin.stat().st_mtime
        log_debug(f"二进制文件修改时间: {datetime.fromtimestamp(binary_mtime)}")
        
        # 检查源码文件
        if self.src_dir.exists():
            for rs_file in self.src_dir.rglob("*.rs"):
                if rs_file.stat().st_mtime > binary_mtime:
                    log_warn(f"源码文件已更新: {rs_file.relative_to(self.project_root)}")
                    log_warn(f"  文件时间: {datetime.fromtimestamp(rs_file.stat().st_mtime)}")
                    return True
        
        # 检查Cargo.toml
        if self.cargo_toml.exists() and self.cargo_toml.stat().st_mtime > binary_mtime:
            log_warn("Cargo.toml已更新，需要重新构建")
            return True
        
        return False
    
    def auto_build(self) -> bool:
        """自动重新构建"""
        log_info("🔨 开始自动重新构建...")
        
        try:
            # 切换到项目根目录
            os.chdir(self.project_root)
            
            # 运行cargo build --release
            log_info("执行: cargo build --release")
            start_time = time.time()
            
            subprocess.run(
                ["cargo", "build", "--release"],
                capture_output=False,
                check=True
            )
            
            build_time = time.time() - start_time
            log_info(f"✅ 构建成功! (耗时: {build_time:.1f}秒)")
            
            # 显示二进制文件信息
            stat = self.solores_bin.stat()
            size_mb = stat.st_size / (1024 * 1024)
            mtime = datetime.fromtimestamp(stat.st_mtime)
            log_info(f"二进制文件: {size_mb:.1f}MB, 修改时间: {mtime}")
            
            return True
            
        except subprocess.CalledProcessError as e:
            log_error(f"❌ 构建失败! 退出码: {e.returncode}")
            return False
        except Exception as e:
            log_error(f"❌ 构建出错: {e}")
            return False
    
    def is_raydium_idl(self, idl_path: str) -> bool:
        """检查是否是raydium.json IDL文件"""
        try:
            idl_file = pathlib.Path(idl_path)
            
            # 首先检查文件是否存在
            if not idl_file.exists():
                return False
            
            # 读取并解析JSON
            with open(idl_file, 'r', encoding='utf-8') as f:
                idl_data = json.load(f)
            
            # 检查程序地址
            program_address = idl_data.get('address', '')
            if program_address == self.RAYDIUM_PROGRAM_ADDRESS:
                log_debug(f"检测到Raydium IDL: {idl_file.name} (地址: {program_address})")
                return True
            
            return False
            
        except (json.JSONDecodeError, FileNotFoundError, KeyError) as e:
            log_debug(f"检查IDL文件失败: {e}")
            return False
    
    def get_output_dir(self, args: list[str]) -> str:
        """从参数中提取输出目录"""
        try:
            for i, arg in enumerate(args):
                if arg in ['-o', '--output'] and i + 1 < len(args):
                    return args[i + 1]
                elif arg.startswith('--output='):
                    return arg.split('=', 1)[1]
            return None
        except Exception as e:
            log_debug(f"解析输出目录失败: {e}")
            return None
    
    def get_batch_output_dir(self, args: list[str]) -> str:
        """从参数中提取批量输出目录"""
        try:
            for i, arg in enumerate(args):
                if arg == '--batch-output-dir' and i + 1 < len(args):
                    return args[i + 1]
                elif arg.startswith('--batch-output-dir='):
                    return arg.split('=', 1)[1]
            return None
        except Exception as e:
            log_debug(f"解析批量输出目录失败: {e}")
            return None
    
    def get_output_interface_dir(self, args: list[str]) -> str:
        """从参数中提取输出目录，并构造接口目录路径"""
        try:
            output_dir = self.get_output_dir(args)
            if not output_dir:
                return None
            
            # 构造接口目录路径: output_dir/sol_raydium_interface
            interface_dir = pathlib.Path(output_dir) / "sol_raydium_interface"
            return str(interface_dir)
            
        except Exception as e:
            log_debug(f"解析输出目录失败: {e}")
            return None
    
    def parse_command_args(self, args: list[str]) -> dict:
        """解析命令行参数，提取关键信息"""
        try:
            return {
                'is_batch': '--batch' in args,
                'input_path': args[0] if args else None,
                'output_dir': self.get_output_dir(args),
                'batch_output_dir': self.get_batch_output_dir(args),
                'has_generate_parser': '--generate-parser' in args
            }
        except Exception as e:
            log_debug(f"解析命令行参数失败: {e}")
            return {
                'is_batch': False,
                'input_path': None,
                'output_dir': None,
                'batch_output_dir': None,
                'has_generate_parser': False
            }
    
    def scan_batch_for_raydium(self, input_path: str) -> list[str]:
        """扫描批量输入目录，找出所有raydium.json文件"""
        raydium_files = []
        
        try:
            input_path_obj = pathlib.Path(input_path)
            
            if input_path_obj.is_file() and input_path_obj.suffix == '.json':
                # 单个文件
                if self.is_raydium_idl(str(input_path_obj)):
                    raydium_files.append(input_path_obj.name)
                    log_debug(f"检测到单个Raydium IDL文件: {input_path_obj.name}")
            elif input_path_obj.is_dir():
                # 目录扫描
                for json_file in input_path_obj.glob('*.json'):
                    if self.is_raydium_idl(str(json_file)):
                        raydium_files.append(json_file.name)
                        log_debug(f"检测到目录中的Raydium IDL文件: {json_file.name}")
            else:
                log_debug(f"输入路径无效或不存在: {input_path}")
                
        except Exception as e:
            log_debug(f"扫描批量raydium文件失败: {e}")
            
        return raydium_files
    
    def apply_raydium_fix(self, interface_dir: str) -> bool:
        """应用Raydium接口修复脚本"""
        try:
            if not self.fix_raydium_script.exists():
                log_error(f"❌ Raydium修复脚本不存在: {self.fix_raydium_script}")
                log_error("   请确保 fix_raydium_interface.py 存在于scripts目录中")
                return False
            
            log_info(f"🔧 应用Raydium接口修复: {interface_dir}")
            
            # 执行修复脚本
            cmd = [str(self.fix_raydium_script), "--interface-dir", interface_dir]
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                check=True
            )
            
            log_info("✅ Raydium接口修复成功!")
            return True
            
        except subprocess.CalledProcessError as e:
            log_error(f"❌ Raydium接口修复失败! 退出码: {e.returncode}")
            log_error(f"   修复脚本: {self.fix_raydium_script}")
            log_error(f"   接口目录: {interface_dir}")
            
            if e.stdout:
                log_error(f"   标准输出:\n{e.stdout}")
            if e.stderr:
                log_error(f"   错误输出:\n{e.stderr}")
            
            log_error("   🛠️  建议检查:")
            log_error("   1. 接口目录是否正确生成")
            log_error("   2. fix_raydium_interface.py脚本是否存在") 
            log_error("   3. 生成的接口结构是否符合预期")
            log_error(f"   4. 手动执行: {' '.join(cmd)}")
            
            return False
        except Exception as e:
            log_error(f"❌ 执行修复脚本时出错: {e}")
            log_error(f"   修复脚本路径: {self.fix_raydium_script}")
            log_error("   🛠️  建议检查脚本权限和依赖")
            return False
    
    def apply_batch_raydium_fixes(self, batch_output_dir: str, raydium_files: list[str]) -> int:
        """对批量生成的raydium接口应用修复"""
        success_count = 0
        
        if not batch_output_dir:
            log_error("❌ 批量输出目录未指定，无法应用修复")
            return 0
        
        batch_path = pathlib.Path(batch_output_dir)
        if not batch_path.exists():
            log_error(f"❌ 批量输出目录不存在: {batch_output_dir}")
            return 0
        
        log_info(f"🔍 开始批量修复处理，检测到{len(raydium_files)}个Raydium IDL文件")
        
        for raydium_file in raydium_files:
            try:
                # 构造接口目录路径: 去除.json后缀，添加sol_前缀和_interface后缀
                file_stem = pathlib.Path(raydium_file).stem
                interface_name = f"sol_{file_stem}_interface"
                interface_dir = batch_path / interface_name
                
                if interface_dir.exists():
                    log_info(f"🔧 检测到批量生成的Raydium接口: {interface_name}")
                    if self.apply_raydium_fix(str(interface_dir)):
                        success_count += 1
                        log_info(f"✅ {interface_name} 修复成功")
                    else:
                        log_warn(f"⚠️  {interface_name} 修复失败")
                else:
                    log_warn(f"⚠️  未找到预期的接口目录: {interface_dir}")
                    log_debug(f"   期望路径: {interface_dir}")
                    log_debug(f"   来源文件: {raydium_file}")
                    
            except Exception as e:
                log_error(f"❌ 处理{raydium_file}时出错: {e}")
        
        return success_count
    
    def run_solores(self, args: list[str]) -> bool:
        """运行solores命令，返回是否成功"""
        cmd = [str(self.solores_bin)] + args
        log_info(f"🎯 执行: {' '.join(cmd)}")
        
        try:
            # 使用subprocess.run保持输出流，但允许后续处理
            result = subprocess.run(cmd, check=True)
            log_info("✅ Solores执行成功")
            return True
        except subprocess.CalledProcessError as e:
            log_error(f"❌ Solores执行失败! 退出码: {e.returncode}")
            return False
        except Exception as e:
            log_error(f"❌ 执行失败: {e}")
            return False
    
    def main(self, args: list[str]):
        """主函数"""
        log_info("🚀 Solores智能包装器启动 (UV版本)")
        
        # 检查项目结构
        if not self.src_dir.exists():
            log_error(f"未找到源码目录: {self.src_dir}")
            sys.exit(1)
        
        # 检查是否需要重新构建
        if self.check_build_needed():
            log_info("需要重新构建，开始自动构建...")
            if not self.auto_build():
                log_error("构建失败，停止执行")
                sys.exit(1)
        else:
            log_info("✅ 二进制文件已是最新版本")
            stat = self.solores_bin.stat()
            mtime = datetime.fromtimestamp(stat.st_mtime)
            log_info(f"二进制文件时间: {mtime}")
        
        # 解析命令参数
        cmd_info = self.parse_command_args(args)
        log_debug(f"解析的命令参数: {cmd_info}")
        
        # 检测raydium文件
        raydium_files = []
        needs_fix = False
        
        if cmd_info['input_path']:
            if cmd_info['is_batch']:
                # 批量处理模式
                raydium_files = self.scan_batch_for_raydium(cmd_info['input_path'])
                needs_fix = len(raydium_files) > 0 and cmd_info['has_generate_parser']
                
                if raydium_files:
                    log_info(f"🎯 检测到批量处理中包含Raydium IDL: {', '.join(raydium_files)}")
            
            elif cmd_info['input_path'].endswith('.json'):
                # 单文件处理模式
                if self.is_raydium_idl(cmd_info['input_path']):
                    raydium_files = [pathlib.Path(cmd_info['input_path']).name]
                    needs_fix = cmd_info['has_generate_parser']
                    log_info(f"🎯 检测到单文件Raydium IDL: {raydium_files[0]}")
        
        # 执行solores命令
        success = self.run_solores(args)
        
        # 如果生成成功且需要修复
        if success and needs_fix:
            log_info("🔧 开始自动Raydium接口修复...")
            
            if cmd_info['is_batch']:
                # 批量修复
                fixed_count = self.apply_batch_raydium_fixes(
                    cmd_info['batch_output_dir'], raydium_files
                )
                log_info(f"✅ 批量修复完成: {fixed_count}/{len(raydium_files)} 个接口修复成功")
                
                if fixed_count < len(raydium_files):
                    log_warn(f"⚠️  部分接口修复失败，请检查日志获取详细信息")
            else:
                # 单文件修复
                interface_dir = self.get_output_interface_dir(args)
                if interface_dir:
                    if self.apply_raydium_fix(interface_dir):
                        log_info("✅ 单文件Raydium接口修复成功")
                    else:
                        log_warn("⚠️  单文件Raydium接口修复失败，请检查详细错误信息")
                else:
                    log_warn("⚠️  无法确定输出接口目录，跳过修复")
        
        elif not success:
            log_error("❌ Solores执行失败，跳过修复步骤")
            sys.exit(1)
        elif needs_fix and not success:
            log_debug("Solores执行失败，跳过Raydium修复")
        else:
            log_debug("无需应用Raydium修复")
        
        log_info("🎉 Solores包装器执行完成")

if __name__ == "__main__":
    wrapper = SoloresWrapper()
    wrapper.main(sys.argv[1:])