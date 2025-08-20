//! Cargo.toml 生成功能
//! 
//! 从 templates 模块中提取出来的独立功能

use crate::{Args, idl_format::IdlFormat};

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

/// 生成标准的Cargo.toml（非workspace模式）
pub fn write_fine_grained_cargo_toml(args: &Args, idl: &dyn IdlFormat) -> std::io::Result<()> {
    use crate::utils::open_file_create_overwrite;
    use std::io::Write;
    
    // 生成基本的Cargo.toml内容
    let _program_id = crate::write_src::get_program_id(args, idl);
    
    let cargo_content = generate_basic_cargo_toml(
        &args.output_crate_name,
        idl.program_version(),
        args.generate_to_json, // 是否需要serde支持
        !args.zero_copy.is_empty(), // 是否需要bytemuck支持
        idl.has_errors(), // 是否有错误定义
        args.no_empty_workspace,
    );

    let cargo_toml_path = args.output_dir.join("Cargo.toml");
    let mut file = open_file_create_overwrite(&cargo_toml_path)?;
    file.write_all(cargo_content.as_bytes())?;
    file.flush()?;

    log::info!("✅ Generated standard Cargo.toml for: {}", args.output_crate_name);
    Ok(())
}

/// 生成基本的Cargo.toml内容
fn generate_basic_cargo_toml(
    crate_name: &str,
    version: &str,
    with_serde: bool,
    with_bytemuck: bool,
    with_errors: bool,
    no_empty_workspace: bool,
) -> String {
    let mut content = format!(
        r#"[package]
name = "{}"
version = "{}"
edition = "2021"
"#,
        crate_name, version
    );

    // 根据no_empty_workspace参数决定是否添加空workspace表
    if !no_empty_workspace {
        content.push_str(r#"
[workspace]
# 空 workspace 表，防止被父目录 workspace 控制
"#);
    }

    content.push_str(r#"
# 核心Solana依赖
solana-pubkey = { version = "2.4.0", features = ["borsh", "curve25519"] }
solana-instruction = "2.3.0"
solana-account-info = "2.4.0"
solana-program-error = "2.3.0"

# 序列化
borsh = "1.5"
"#);

    // 可选的serde支持
    if with_serde {
        content.push_str(r#"
# Serde支持
serde = { version = "1.0", features = ["derive"], optional = true }
serde_json = { version = "1.0", optional = true }
"#);
    }

    // 可选的bytemuck支持
    if with_bytemuck {
        content.push_str(r#"
# Zero-copy支持
bytemuck = { version = "1.16", features = ["derive"] }
"#);
    }

    // 可选的错误处理支持
    if with_errors {
        content.push_str(r#"
# 错误处理
thiserror = "1.0"
num-derive = "0.4.2"
num-traits = "0.2"
"#);
    }

    // Features
    if with_serde {
        content.push_str(r#"
[features]
serde = ["dep:serde", "dep:serde_json"]
"#);
    }

    content
}