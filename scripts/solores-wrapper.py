#!/usr/bin/env uv run
# /// script
# requires-python = ">=3.8"
# dependencies = []
# ///

"""
Solores智能包装器 - 确保每次使用最新的二进制文件
"""

import os
import sys
import subprocess
import pathlib
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
    
    def run_solores(self, args: list[str]):
        """运行solores命令"""
        cmd = [str(self.solores_bin)] + args
        log_info(f"🎯 执行: {' '.join(cmd)}")
        
        try:
            # 直接执行solores，保持输出流
            os.execv(str(self.solores_bin), cmd)
        except Exception as e:
            log_error(f"❌ 执行失败: {e}")
            sys.exit(1)
    
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
        
        # 执行solores命令
        self.run_solores(args)

if __name__ == "__main__":
    wrapper = SoloresWrapper()
    wrapper.main(sys.argv[1:])