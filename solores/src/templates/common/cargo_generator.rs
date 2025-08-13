//! 统一的Cargo.toml生成器
//!
//! 提供细分化的Solana依赖管理，完全移除对solana-program的依赖
//! 使用pubkey!宏替代declare_id!宏

use serde::Serialize;
use toml::{map::Map, Value};

use crate::{Args, templates::{ContractMode, TemplateGenerator}, utils::open_file_create_overwrite, idl_format::IdlFormat};
// 移除导入，使用完整路径 std::io::Write

/// Cargo.toml生成器
pub struct CargoTomlGenerator<'a> {
    /// 程序名称
    pub program_name: &'a str,
    /// 程序版本
    pub program_version: &'a str,
    /// 依赖配置信息
    pub dependency_profile: DependencyProfile,
    /// CLI参数
    pub args: &'a Args,
}

/// 依赖配置信息
pub struct DependencyProfile {
    /// 是否有错误定义
    pub has_errors: bool,
    /// 是否使用zero-copy
    pub has_zero_copy: bool,
    /// 合约模式
    pub contract_mode: ContractMode,
}

impl<'a> CargoTomlGenerator<'a> {
    /// 创建新的生成器
    pub fn new(
        program_name: &'a str,
        program_version: &'a str,
        args: &'a Args,
        dependency_profile: DependencyProfile,
    ) -> Self {
        Self {
            program_name,
            program_version,
            args,
            dependency_profile,
        }
    }

    /// 生成完整的Cargo.toml内容
    pub fn generate_cargo_toml(&self) -> String {
        let cargo_toml = CargoToml::from_generator(self);
        toml::to_string(&cargo_toml).unwrap()
    }

    /// 直接写入Cargo.toml文件
    pub fn write_cargo_toml(&self) -> std::io::Result<()> {
        let cargo_toml_content = self.generate_cargo_toml();
        let path = self.args.output_dir.join("Cargo.toml");
        let mut file = open_file_create_overwrite(path)?;
        std::io::Write::write_all(&mut file, cargo_toml_content.as_bytes())?;
        std::io::Write::flush(&mut file)
    }

    /// 获取细分化的依赖映射
    pub fn get_fine_grained_dependencies(&self) -> Map<String, Value> {
        let mut deps = Map::new();

        // Borsh序列化库
        deps.insert("borsh".to_string(), self.create_dependency_value(&self.args.borsh_vers));

        // 完全细分化的Solana依赖 - 无需solana-program
        deps.insert("solana-pubkey".to_string(), self.create_features_dependency_value("2.4.0", vec!["borsh", "curve25519"]));
        deps.insert("solana-program-error".to_string(), self.create_dependency_value("2.2.2"));
        deps.insert("solana-instruction".to_string(), self.create_dependency_value("2.3.0"));
        
        // 可选的扩展功能依赖
        deps.insert("solana-account-info".to_string(), self.create_optional_dependency_value("2.3.0"));
        deps.insert("solana-program-entrypoint".to_string(), self.create_optional_dependency_value("2.3.0"));
        deps.insert("solana-cpi".to_string(), self.create_optional_dependency_value("2.2.1"));

        // 可选的serde依赖
        deps.insert("serde".to_string(), self.create_optional_dependency_value(&self.args.serde_vers));
        deps.insert("serde_with".to_string(), self.create_optional_dependency_value(&self.args.serde_with_vers));

        // 错误处理相关依赖
        if self.dependency_profile.has_errors {
            deps.insert("thiserror".to_string(), self.create_dependency_value(&self.args.thiserror_vers));
            deps.insert("num-derive".to_string(), self.create_dependency_value(&self.args.num_derive_vers));
            deps.insert("num-traits".to_string(), self.create_dependency_value(&self.args.num_traits_vers));
        }

        // Zero-copy支持
        if self.dependency_profile.has_zero_copy {
            deps.insert("bytemuck".to_string(), self.create_features_dependency_value(&self.args.bytemuck_vers, vec!["derive"]));
        }

        deps
    }

    /// 创建简单依赖值
    fn create_dependency_value(&self, version: &str) -> Value {
        let mut map = Map::new();
        map.insert("version".to_string(), Value::String(version.to_string()));
        Value::Table(map)
    }

    /// 创建带特性的依赖值
    fn create_features_dependency_value(&self, version: &str, features: Vec<&str>) -> Value {
        let mut map = Map::new();
        map.insert("version".to_string(), Value::String(version.to_string()));
        
        let feature_values: Vec<Value> = features.into_iter()
            .map(|f| Value::String(f.to_string()))
            .collect();
        map.insert("features".to_string(), Value::Array(feature_values));
        
        Value::Table(map)
    }

    /// 创建可选依赖值
    fn create_optional_dependency_value(&self, version: &str) -> Value {
        let mut map = Map::new();
        map.insert("version".to_string(), Value::String(version.to_string()));
        map.insert("optional".to_string(), Value::Boolean(true));
        Value::Table(map)
    }

    /// 获取features配置
    fn get_features(&self) -> Map<String, Value> {
        let mut features = Map::new();
        
        // serde feature
        let serde_deps = vec![
            Value::String("dep:serde".to_string()),
            Value::String("dep:serde_with".to_string()),
        ];
        features.insert("serde".to_string(), Value::Array(serde_deps));

        // 可选的扩展功能 features
        features.insert("account-info".to_string(), Value::Array(vec![
            Value::String("dep:solana-account-info".to_string()),
        ]));
        
        features.insert("program-entrypoint".to_string(), Value::Array(vec![
            Value::String("dep:solana-program-entrypoint".to_string()),
        ]));
        
        features.insert("cpi".to_string(), Value::Array(vec![
            Value::String("dep:solana-cpi".to_string()),
        ]));
        
        // 便利 feature 包含所有扩展功能
        features.insert("full-solana".to_string(), Value::Array(vec![
            Value::String("account-info".to_string()),
            Value::String("program-entrypoint".to_string()),
            Value::String("cpi".to_string()),
        ]));

        features
    }
}

/// 便利函数：从IDL格式创建并写入Cargo.toml
pub fn write_fine_grained_cargo_toml(args: &Args, idl: &dyn IdlFormat) -> std::io::Result<()> {
    // 检查是否有错误定义 - 基于IDL内容而不是项目名称
    let has_errors = idl.has_errors();

    // 创建依赖配置
    let profile = DependencyProfile {
        has_errors,
        has_zero_copy: !args.zero_copy.is_empty(),
        contract_mode: if idl.is_anchor_contract() { 
            ContractMode::Anchor 
        } else { 
            ContractMode::NonAnchor 
        },
    };

    // 创建生成器并写入文件
    let generator = CargoTomlGenerator::new(
        idl.program_name(),
        idl.program_version(),
        args,
        profile
    );

    generator.write_cargo_toml()
}

/// Cargo.toml主结构
#[derive(Serialize)]
pub struct CargoToml<'a> {
    pub package: Package<'a>,
    pub workspace: Map<String, Value>,
    pub dependencies: Map<String, Value>,
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub features: Map<String, Value>,
}

impl<'a> CargoToml<'a> {
    /// 从生成器创建Cargo.toml
    pub fn from_generator(generator: &'a CargoTomlGenerator<'a>) -> Self {
        Self {
            package: Package {
                name: &generator.args.output_crate_name,
                version: generator.program_version,
                edition: "2021",
            },
            workspace: Map::new(), // 空workspace
            dependencies: generator.get_fine_grained_dependencies(),
            features: generator.get_features(),
        }
    }
}

/// 包信息
#[derive(Serialize)]
pub struct Package<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub edition: &'a str,
}

/// 实现TemplateGenerator trait，使其能在模板系统中使用
impl<'a> TemplateGenerator for CargoTomlGenerator<'a> {
    fn get_standard_module_name(&self) -> &'static str {
        "cargo"
    }

    fn gen_files(&self) -> Vec<(String, proc_macro2::TokenStream)> {
        // 这个生成器不生成Rust代码，而是生成TOML文件
        // 在实际实现中，应该由特殊的TOML写入逻辑处理
        vec![("Cargo.toml".to_string(), quote::quote! {})]
    }

    fn gen_mod_file(&self) -> proc_macro2::TokenStream {
        quote::quote! {}
    }

    fn is_single_root_file(&self) -> bool {
        true // Cargo.toml应该在项目根目录
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_args() -> Args {
        Args {
            output_crate_name: "test_crate".to_string(),
            borsh_vers: "^1.5".to_string(),
            serde_vers: "^1.0".to_string(),
            serde_with_vers: "^3.0".to_string(),
            thiserror_vers: "^1.0".to_string(),
            num_derive_vers: "0.4.2".to_string(),
            num_traits_vers: "^0.2".to_string(),
            bytemuck_vers: "^1.16".to_string(),
            zero_copy: vec![],
            ..Default::default()
        }
    }

    #[test]
    fn test_basic_dependencies() {
        let args = create_test_args();
        let profile = DependencyProfile {
            has_errors: false,
            has_zero_copy: false,
            contract_mode: ContractMode::Anchor,
        };

        let generator = CargoTomlGenerator::new("test", "0.1.0", &args, profile);
        let toml_content = generator.generate_cargo_toml();

        // 验证包含细分化依赖
        assert!(toml_content.contains("solana-pubkey"));
        assert!(toml_content.contains("solana-program-error"));
        assert!(toml_content.contains("solana-account-info"));
        
        // 验证完全不包含solana-program依赖
        assert!(!toml_content.contains("solana-program"));
        
        // 验证包含所有细分化依赖
        assert!(toml_content.contains("solana-account-info"));
        assert!(toml_content.contains("solana-instruction"));
        assert!(toml_content.contains("solana-program-entrypoint"));
        assert!(toml_content.contains("solana-cpi"));
    }

    #[test]
    fn test_error_dependencies() {
        let args = create_test_args();
        let profile = DependencyProfile {
            has_errors: true,
            has_zero_copy: false,
            contract_mode: ContractMode::Anchor,
        };

        let generator = CargoTomlGenerator::new("test", "0.1.0", &args, profile);
        let toml_content = generator.generate_cargo_toml();

        // 验证包含错误处理依赖
        assert!(toml_content.contains("thiserror"));
        assert!(toml_content.contains("num-derive"));
        assert!(toml_content.contains("num-traits"));
    }

    #[test]
    fn test_zero_copy_dependencies() {
        let args = create_test_args();
        let profile = DependencyProfile {
            has_errors: false,
            has_zero_copy: true,
            contract_mode: ContractMode::NonAnchor,
        };

        let generator = CargoTomlGenerator::new("test", "0.1.0", &args, profile);
        let toml_content = generator.generate_cargo_toml();

        // 验证包含bytemuck依赖
        assert!(toml_content.contains("bytemuck"));
    }
}

/// 为workspace成员生成Cargo.toml
pub fn write_workspace_member_cargo_toml(args: &Args, idl: &dyn IdlFormat) -> std::io::Result<()> {
    use crate::workspace::generate_member_cargo_toml;
    use crate::write_src::get_program_id;
    use std::fs;

    let program_id = get_program_id(args, idl);
    
    let cargo_toml_content = generate_member_cargo_toml(
        &args.output_crate_name,
        &program_id,
        args.generate_parser,
        args.test,
        &args.zero_copy,
    );

    let cargo_toml_path = args.output_dir.join("Cargo.toml");
    fs::write(&cargo_toml_path, cargo_toml_content)?;

    log::info!("✅ Generated workspace member Cargo.toml for: {}", args.output_crate_name);
    Ok(())
}

/// 检查是否为workspace模式（通过环境或参数判断）
pub fn should_use_workspace_cargo_toml(args: &Args) -> bool {
    // 如果workspace标志为真且在批量模式下
    args.workspace && args.batch
}