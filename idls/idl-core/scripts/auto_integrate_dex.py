#!/usr/bin/env uv run
# /// script
# dependencies = ["toml", "jinja2"]
# ///
"""
🚀 DEX协议自动集成工具 (UV版本)

功能:
- 智能去除Cargo.toml中的workspace声明
- 自动生成src/dex/下的parser实现  
- 重复运行保护和智能判断
- 现代化的UV工具链支持

用法:
    uv run scripts/auto_integrate_dex.py
    或
    chmod +x scripts/auto_integrate_dex.py && ./scripts/auto_integrate_dex.py
"""

import os
import re
import sys
from pathlib import Path
from typing import List, Dict, Optional, Tuple
import toml
from jinja2 import Environment, FileSystemLoader

class UVDexIntegrator:
    def __init__(self):
        self.base_dir = Path.cwd()
        self.crates_dir = self.base_dir / "crates"
        self.dex_dir = self.base_dir / "src" / "dex"
        self.scripts_dir = self.base_dir / "scripts"
        self.templates_dir = self.scripts_dir / "templates"
        
        # 设置Jinja2环境
        self.jinja_env = Environment(
            loader=FileSystemLoader(self.templates_dir),
            trim_blocks=True,
            lstrip_blocks=True
        )
        
    def run(self) -> bool:
        """主执行函数"""
        print("🚀 UV DEX协议自动集成工具启动")
        print("=" * 50)
        
        if not self.validate_environment():
            return False
            
        # 1. 扫描接口库
        interface_libs = self.scan_interface_libraries()
        if not interface_libs:
            print("❌ 未发现任何接口库")
            return False
            
        print(f"📦 发现 {len(interface_libs)} 个接口库")
        
        # 2. 处理workspace (智能判断)
        print("\n🔧 处理Cargo.toml workspace声明:")
        workspace_results = self.process_cargo_workspaces(interface_libs)
        
        # 3. 生成DEX解析器 (智能判断)
        print("\n🏗️  生成DEX解析器:")
        parser_results = self.generate_dex_parsers(interface_libs)
        
        # 4. 输出摘要
        self.print_summary(workspace_results, parser_results)
        return True
    
    def validate_environment(self) -> bool:
        """验证运行环境"""
        if not self.crates_dir.exists():
            print("❌ crates目录不存在，请确保在idl-core根目录运行")
            return False
            
        if not self.dex_dir.exists():
            print(f"📁 创建dex目录: {self.dex_dir}")
            self.dex_dir.mkdir(parents=True)
            
        if not self.templates_dir.exists():
            print(f"📁 创建templates目录: {self.templates_dir}")
            self.templates_dir.mkdir(parents=True)
            
        return True
    
    def scan_interface_libraries(self) -> List[Dict]:
        """扫描并分析所有接口库"""
        libs = []
        
        for crate_path in self.crates_dir.glob("sol_*_interface"):
            if crate_path.is_dir():
                lib_info = self.analyze_interface_library(crate_path)
                if lib_info:
                    libs.append(lib_info)
                    
        return sorted(libs, key=lambda x: x['name'])
    
    def analyze_interface_library(self, crate_path: Path) -> Optional[Dict]:
        """分析单个接口库的信息"""
        try:
            # 查找实际的Cargo.toml (处理嵌套目录结构)
            cargo_toml_path = self.find_cargo_toml(crate_path)
            if not cargo_toml_path:
                return None
            
            # 读取Cargo.toml获取包信息
            cargo_data = toml.load(cargo_toml_path)
            package_name = cargo_data.get('package', {}).get('name', crate_path.name)
            
            # 提取协议信息
            protocol_info = self.extract_protocol_info(package_name)
            
            lib_info = {
                'name': package_name,
                'crate_name': package_name.replace('-', '_'),
                'crate_path': crate_path,
                'cargo_toml_path': cargo_toml_path,
                **protocol_info
            }
            
            return lib_info
            
        except Exception as e:
            print(f"❌ {crate_path.name}: 分析失败 - {e}")
            return None
    
    def find_cargo_toml(self, crate_path: Path) -> Optional[Path]:
        """智能查找Cargo.toml文件"""
        # 直接查找
        direct_cargo = crate_path / "Cargo.toml"
        if direct_cargo.exists():
            return direct_cargo
            
        # 查找子目录中的Cargo.toml (处理嵌套结构)
        for sub_dir in crate_path.iterdir():
            if sub_dir.is_dir():
                cargo_toml = sub_dir / "Cargo.toml"
                if cargo_toml.exists():
                    return cargo_toml
                    
        return None
    
    def extract_protocol_info(self, package_name: str) -> Dict:
        """从包名提取协议信息"""
        # 协议映射表
        protocol_mapping = {
            'sol_raydium_interface': {
                'dex_name': 'raydium',
                'display_name': 'Raydium',
                'instruction_type': 'ProgramInstruction',
                'account_type': 'RaydiumAccount',
            },
            'sol_raydium_launchpad_interface': {
                'dex_name': 'raydium_launchpad',
                'display_name': 'RaydiumLaunchpad',
                'instruction_type': 'RaydiumLaunchpadInstruction',
                'account_type': 'RaydiumLaunchpadAccount',
            },
            'sol_pump_fun_interface': {
                'dex_name': 'pump_fun',
                'display_name': 'PumpFun',
                'instruction_type': 'PumpfunInstruction',
                'account_type': 'PumpfunAccount',
            },
            'sol_orca_whirlpool_interface': {
                'dex_name': 'orca_whirlpool',
                'display_name': 'OrcaWhirlpool',
                'instruction_type': 'WhirlpoolInstruction',
                'account_type': 'WhirlpoolAccount',
            },
            'sol_meteora_dbc_interface': {
                'dex_name': 'meteora_dbc',
                'display_name': 'MeteoraDbc',
                'instruction_type': 'MeteoraDbcInstruction',
                'account_type': 'MeteoraDbcAccount',
            },
            'sol_pump_amm_interface': {
                'dex_name': 'pump_amm',
                'display_name': 'PumpAmm',
                'instruction_type': 'PumpAmmInstruction',
                'account_type': 'PumpAmmAccount',
            },
            'sol_phoenix_interface': {
                'dex_name': 'phoenix',
                'display_name': 'Phoenix',
                'instruction_type': 'PhoenixInstruction',
                'account_type': 'PhoenixAccount',
            },
            'sol_serum_interface': {
                'dex_name': 'serum',
                'display_name': 'Serum',
                'instruction_type': 'SerumInstruction',
                'account_type': 'SerumAccount',
            },
        }
        
        # 返回映射信息或默认值
        return protocol_mapping.get(package_name, {
            'dex_name': package_name.replace('sol_', '').replace('_interface', ''),
            'display_name': package_name.replace('sol_', '').replace('_interface', '').title(),
            'instruction_type': 'ProgramInstruction',
            'account_type': 'ProgramAccount',
        })
    
    def is_workspace_already_removed(self, cargo_toml_path: Path) -> bool:
        """检查workspace是否已经被删除"""
        try:
            content = cargo_toml_path.read_text()
            # 检查是否包含独立的[workspace]行
            return not re.search(r'^\[workspace\]\s*$', content, re.MULTILINE)
        except Exception:
            return False
    
    def is_parser_already_generated(self, dex_name: str) -> bool:
        """检查DEX解析器是否已经生成"""
        parser_dir = self.dex_dir / dex_name / "parser"
        protocol_mod = self.dex_dir / dex_name / "mod.rs"
        
        required_files = [
            parser_dir / "instructions.rs",
            parser_dir / "accounts.rs",
            parser_dir / "mod.rs",
            protocol_mod,
        ]
        
        # 检查所有必需文件存在且有内容
        return all(
            f.exists() and f.stat().st_size > 50 
            for f in required_files
        )
    
    def remove_workspace_declaration(self, cargo_toml_path: Path):
        """删除Cargo.toml中的workspace声明"""
        content = cargo_toml_path.read_text()
        
        # 删除独立的[workspace]行
        updated_content = re.sub(r'^\[workspace\]\s*\n', '', content, flags=re.MULTILINE)
        
        # 只有内容发生变化时才写入
        if updated_content != content:
            cargo_toml_path.write_text(updated_content)
    
    def create_parser_implementation(self, lib_info: Dict):
        """创建完整的parser实现"""
        dex_name = lib_info['dex_name']
        parser_dir = self.dex_dir / dex_name / "parser"
        protocol_dir = self.dex_dir / dex_name
        
        # 创建目录
        parser_dir.mkdir(parents=True, exist_ok=True)
        
        # 生成instructions.rs
        instructions_content = self.render_template('instruction_parser.rs.jinja2', lib_info)
        (parser_dir / "instructions.rs").write_text(instructions_content)
        
        # 生成accounts.rs
        accounts_content = self.render_template('account_parser.rs.jinja2', lib_info)
        (parser_dir / "accounts.rs").write_text(accounts_content)
        
        # 生成parser/mod.rs
        parser_mod_content = self.render_template('parser_mod.rs.jinja2', lib_info)
        (parser_dir / "mod.rs").write_text(parser_mod_content)
        
        # 生成protocol/mod.rs
        protocol_mod_content = self.render_template('protocol_mod.rs.jinja2', lib_info)
        (protocol_dir / "mod.rs").write_text(protocol_mod_content)
    
    def render_template(self, template_name: str, variables: Dict) -> str:
        """渲染Jinja2模板"""
        template = self.jinja_env.get_template(template_name)
        return template.render(**variables)
    
    def process_cargo_workspaces(self, libs: List[Dict]) -> Dict:
        """智能处理Cargo.toml workspace声明"""
        results = {"processed": 0, "skipped": 0, "failed": 0}
        
        for lib in libs:
            cargo_toml_path = lib['cargo_toml_path']
            
            # 智能判断是否已处理
            if self.is_workspace_already_removed(cargo_toml_path):
                print(f"   ⏭️  {lib['name']}: workspace已删除")
                results["skipped"] += 1
                continue
            
            # 删除workspace声明
            try:
                self.remove_workspace_declaration(cargo_toml_path)
                print(f"   ✅ {lib['name']}: 删除workspace声明")
                results["processed"] += 1
            except Exception as e:
                print(f"   ❌ {lib['name']}: 删除失败 - {e}")
                results["failed"] += 1
                
        return results
    
    def generate_dex_parsers(self, libs: List[Dict]) -> Dict:
        """智能生成DEX解析器"""
        results = {"generated": 0, "skipped": 0, "failed": 0}
        
        for lib in libs:
            dex_name = lib['dex_name']
            
            # 智能判断是否已生成
            if self.is_parser_already_generated(dex_name):
                print(f"   ⏭️  {lib['name']}: parser已存在")
                results["skipped"] += 1
                continue
            
            # 生成parser实现
            try:
                self.create_parser_implementation(lib)
                print(f"   ✅ {lib['name']}: 生成parser完成")
                results["generated"] += 1
            except Exception as e:
                print(f"   ❌ {lib['name']}: 生成失败 - {e}")
                results["failed"] += 1
                
        return results
    
    def print_summary(self, workspace_results: Dict, parser_results: Dict):
        """打印执行摘要"""
        print("\n📊 执行摘要:")
        print(f"   Workspace处理: {workspace_results['processed']}个处理, {workspace_results['skipped']}个跳过, {workspace_results['failed']}个失败")
        print(f"   Parser生成: {parser_results['generated']}个生成, {parser_results['skipped']}个跳过, {parser_results['failed']}个失败")
        
        if workspace_results['failed'] == 0 and parser_results['failed'] == 0:
            print("\n✅ 所有操作成功完成!")
        else:
            print("\n⚠️  部分操作有问题，请检查上述日志")

def main():
    """主函数"""
    try:
        integrator = UVDexIntegrator()
        success = integrator.run()
        
        if success:
            print("\n🎉 DEX协议集成工具执行完成!")
            sys.exit(0)
        else:
            print("\n❌ 执行失败!")
            sys.exit(1)
            
    except KeyboardInterrupt:
        print("\n\n⏹️  用户中断")
        sys.exit(130)
    except Exception as e:
        print(f"\n❌ 意外错误: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()