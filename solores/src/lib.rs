#![doc = include_str!("../README.md")]

use std::{
    env,
    fs::{self, File, OpenOptions},
    io::Read,
    path::PathBuf,
    process::Command,
};

use clap::{command, Parser};
use idl_format::{IdlFormat, IdlFormatEnum, parse_idl_json};

use crate::error::{SoloresError, diagnose_json_error, validate_idl_structure, format_user_error};

// Just make all mods pub to allow ppl to use the lib

pub mod error;
pub mod idl_format;
pub mod templates;  // 新增模板系统
pub mod utils;
pub mod write_gitignore;
pub mod write_readme;
pub mod write_src;

use templates::common::cargo_generator::write_fine_grained_cargo_toml;
use write_gitignore::write_gitignore;
use write_readme::write_readme;
use write_src::*;

const DEFAULT_OUTPUT_CRATE_NAME_MSG: &str = "<name-of-program>_interface";
const DEFAULT_PROGRAM_ID_MSG: &str = "program ID in IDL else system program ID if absent";
const RUST_LOG_ENV_VAR: &str = "RUST_LOG";

#[derive(Parser, Debug, Default, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    pub idl_path: PathBuf,

    #[arg(
        long,
        short,
        help = "directory to output generated crate to",
        default_value = "./"
    )]
    pub output_dir: PathBuf,

    #[arg(
        long,
        help = "output crate name",
        default_value = DEFAULT_OUTPUT_CRATE_NAME_MSG,
    )]
    pub output_crate_name: String,

    #[arg(long, short, help = "program ID / address / pubkey", default_value = DEFAULT_PROGRAM_ID_MSG)]
    pub program_id: Option<String>,

    #[arg(
        long,
        short,
        help = "typedefs and accounts to derive bytemuck::Pod for. Does not currently check validity of derivation."
    )]
    pub zero_copy: Vec<String>,

    #[arg(
        long,
        short,
        help = "solana-program dependency version for generated crate",
        default_value = "^2.0"
    )]
    pub solana_program_vers: String,

    #[arg(
        long,
        short,
        help = "borsh dependency version for generated crate",
        default_value = "^1.5"
    )]
    pub borsh_vers: String,

    #[arg(
        long,
        help = "thiserror dependency version for generated crate",
        default_value = "^1.0"
    )]
    pub thiserror_vers: String,

    #[arg(
        long,
        help = "num-derive dependency version for generated crate",
        default_value = "0.4.2"
    )]
    pub num_derive_vers: String,

    #[arg(
        long,
        help = "num-traits dependency version for generated crate",
        default_value = "^0.2"
    )]
    pub num_traits_vers: String,

    #[arg(
        long,
        help = "serde dependency version for generated crate",
        default_value = "^1.0"
    )]
    pub serde_vers: String,

    #[arg(
        long,
        help = "serde_with dependency version for generated crate",
        default_value = "^3.0"
    )]
    pub serde_with_vers: String,

    #[arg(
        long,
        help = "bytemuck dependency version for generated crate",
        default_value = "^1.16"
    )]
    pub bytemuck_vers: String,

    #[arg(
        long,
        default_value = "true",
        help = "generate parser functions for account and instruction parsing (enabled by default)"
    )]
    pub generate_parser: bool,

    #[arg(
        long,
        help = "only generate parser code (skip interface generation)"
    )]
    pub parser_only: bool,

    #[arg(
        long,
        help = "批量处理模式 - 处理指定目录中的所有IDL文件"
    )]
    pub batch: bool,

    #[arg(
        long,
        help = "批量生成时的基础输出目录",
        default_value = "./batch_output"
    )]
    pub batch_output_dir: PathBuf,
}

/// The CLI entrypoint
pub fn main() {
    if env::var(RUST_LOG_ENV_VAR).is_err() {
        env::set_var(RUST_LOG_ENV_VAR, "info")
    }
    env_logger::init();
    log_panics::init();

    let args = Args::parse();

    if args.batch {
        process_batch(args);
    } else {
        process_single_file(args);
    }
}

/// Process a single IDL file (original functionality)
fn process_single_file(mut args: Args) {
    let mut file = OpenOptions::new().read(true).open(&args.idl_path).unwrap();

    let idl = load_idl(&mut file);

    if args.output_crate_name == DEFAULT_OUTPUT_CRATE_NAME_MSG {
        args.output_crate_name = format!("sol_{}_interface", idl.program_name());
    }

    args.program_id = args.program_id.and_then(|s| {
        if s == DEFAULT_PROGRAM_ID_MSG {
            None
        } else {
            Some(s)
        }
    });

    args.output_dir.push(&args.output_crate_name);
    fs::create_dir_all(args.output_dir.join("src/")).unwrap();

    // TODO: multithread, 1 thread per generated file
    write_gitignore(&args).unwrap();
    write_fine_grained_cargo_toml(&args, idl.as_ref()).unwrap();
    
    log::info!("Writing lib.rs for IDL: {}", idl.program_name());
    log::debug!("IDL address: {:?}", idl.program_address());
    write_lib(&args, idl.as_ref())
        .unwrap_or_else(|e| {
            log::error!("Failed to write lib.rs: {}", e);
            panic!("write_lib failed: {}", e);
        });
    write_readme(&args, idl.as_ref()).unwrap();
    
    // Copy IDL file to output directory
    let idl_dest = args.output_dir.join("idl.json");
    if let Err(e) = std::fs::copy(&args.idl_path, &idl_dest) {
        log::warn!("Failed to copy IDL file: {}", e);
    } else {
        log::info!("IDL file copied to {}", idl_dest.display());
    }
    
    // Format generated code with cargo fmt
    log::debug!("🎨 准备运行cargo fmt...");
    
    // 检查格式化前的一个样本文件
    let sample_instruction_file = args.output_dir.join("src/instructions").join("create_platform_config.rs");
    let use_count_before = if sample_instruction_file.exists() {
        let content = std::fs::read_to_string(&sample_instruction_file).unwrap_or_default();
        content.matches("use crate::*").count()
    } else {
        0
    };
    log::debug!("🎨 格式化前样本文件 use crate::* 数量: {}", use_count_before);
    
    let fmt_result = Command::new("cargo")
        .args(&["fmt"])
        .current_dir(&args.output_dir)
        .output();

    match fmt_result {
        Ok(output) => {
            if output.status.success() {
                // 检查格式化后的同一个样本文件
                let use_count_after = if sample_instruction_file.exists() {
                    let content = std::fs::read_to_string(&sample_instruction_file).unwrap_or_default();
                    content.matches("use crate::*").count()
                } else {
                    0
                };
                log::debug!("🎨 格式化后样本文件 use crate::* 数量: {}", use_count_after);
                
                if use_count_before != use_count_after {
                    log::warn!("⚠️ cargo fmt改变了导入数量！前: {}, 后: {}", use_count_before, use_count_after);
                }
                
                log::info!("Code formatted successfully with cargo fmt");
            } else {
                log::warn!("cargo fmt failed: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            log::warn!("Failed to run cargo fmt: {}", e);
        }
    }
    
    log::info!(
        "{} crate written to {}",
        args.output_crate_name,
        args.output_dir.to_string_lossy()
    );
}

/// Process multiple IDL files in batch mode
fn process_batch(args: Args) {
    log::info!("🚀 启动批量处理模式");
    log::info!("📁 扫描目录: {}", args.idl_path.display());
    log::info!("📁 输出目录: {}", args.batch_output_dir.display());

    // Create batch output directory
    if let Err(e) = fs::create_dir_all(&args.batch_output_dir) {
        log::error!("❌ 无法创建批量输出目录: {}", e);
        panic!("Failed to create batch output directory: {}", e);
    }

    // Scan for IDL files
    let idl_files = scan_idl_files(&args.idl_path);
    if idl_files.is_empty() {
        log::warn!("⚠️  在目录 {} 中未找到IDL文件", args.idl_path.display());
        return;
    }

    log::info!("📋 找到 {} 个IDL文件待处理", idl_files.len());

    let mut success_count = 0;
    let mut failure_count = 0;
    let mut failed_files = Vec::new();

    for (idx, idl_file) in idl_files.iter().enumerate() {
        log::info!("🔄 处理文件 {}/{}: {}", idx + 1, idl_files.len(), idl_file.display());
        
        match process_single_idl_file(&args, &idl_file) {
            Ok(output_dir) => {
                success_count += 1;
                log::info!("✅ 成功生成: {}", output_dir.display());
            }
            Err(e) => {
                failure_count += 1;
                log::error!("❌ 处理失败 {}: {}", idl_file.display(), e);
                failed_files.push((idl_file.clone(), e));
            }
        }
    }

    // Print summary
    log::info!("🎉 批量处理完成!");
    log::info!("✅ 成功: {} 个文件", success_count);
    if failure_count > 0 {
        log::warn!("❌ 失败: {} 个文件", failure_count);
        for (file, error) in failed_files {
            log::warn!("   {} - {}", file.display(), error);
        }
    }
    log::info!("📁 所有生成的库位于: {}", args.batch_output_dir.display());
}

/// Scan directory for IDL files
fn scan_idl_files(dir_path: &PathBuf) -> Vec<PathBuf> {
    let mut idl_files = Vec::new();
    
    if !dir_path.is_dir() {
        log::error!("❌ 指定路径不是目录: {}", dir_path.display());
        return idl_files;
    }

    match fs::read_dir(dir_path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(extension) = path.extension() {
                            if extension == "json" {
                                // Quick validation - check if it's a valid JSON file
                                if let Ok(mut file) = File::open(&path) {
                                    let mut content = String::new();
                                    if file.read_to_string(&mut content).is_ok() {
                                        if let Ok(_) = serde_json::from_str::<serde_json::Value>(&content) {
                                            idl_files.push(path);
                                        } else {
                                            log::debug!("⚠️  跳过无效JSON文件: {}", path.display());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            log::error!("❌ 无法读取目录 {}: {}", dir_path.display(), e);
        }
    }

    // Sort files for consistent processing order
    idl_files.sort();
    idl_files
}

/// Process a single IDL file for batch mode
fn process_single_idl_file(base_args: &Args, idl_file_path: &PathBuf) -> Result<PathBuf, String> {
    // Clone base args and customize for this specific file
    let mut args = base_args.clone();
    args.idl_path = idl_file_path.clone();

    // Load and validate IDL
    let mut file = match OpenOptions::new().read(true).open(&args.idl_path) {
        Ok(f) => f,
        Err(e) => return Err(format!("无法打开文件: {}", e)),
    };

    let idl = match load_idl_safely(&mut file) {
        Ok(idl) => idl,
        Err(e) => return Err(format!("IDL解析失败: {}", e)),
    };

    // Generate output crate name
    if args.output_crate_name == DEFAULT_OUTPUT_CRATE_NAME_MSG {
        args.output_crate_name = format!("sol_{}_interface", idl.program_name());
    }

    // Set up output directory in batch output directory
    args.output_dir = base_args.batch_output_dir.join(&args.output_crate_name);

    // Handle program ID
    args.program_id = args.program_id.and_then(|s| {
        if s == DEFAULT_PROGRAM_ID_MSG {
            None
        } else {
            Some(s)
        }
    });

    // Create output directory
    if let Err(e) = fs::create_dir_all(args.output_dir.join("src/")) {
        return Err(format!("无法创建输出目录: {}", e));
    }

    // Generate files
    if let Err(e) = write_gitignore(&args) {
        return Err(format!("生成.gitignore失败: {}", e));
    }

    if let Err(e) = write_fine_grained_cargo_toml(&args, idl.as_ref()) {
        return Err(format!("生成Cargo.toml失败: {}", e));
    }

    if let Err(e) = write_lib(&args, idl.as_ref()) {
        return Err(format!("生成lib.rs失败: {}", e));
    }

    if let Err(e) = write_readme(&args, idl.as_ref()) {
        return Err(format!("生成README.md失败: {}", e));
    }

    // Copy IDL file
    let idl_dest = args.output_dir.join("idl.json");
    if let Err(e) = std::fs::copy(&args.idl_path, &idl_dest) {
        log::warn!("复制IDL文件失败: {}", e);
    }

    // Format code (non-blocking)
    let _ = Command::new("cargo")
        .args(&["fmt"])
        .current_dir(&args.output_dir)
        .output();

    Ok(args.output_dir)
}

/// Safe IDL loading that doesn't panic
fn load_idl_safely(file: &mut File) -> Result<Box<dyn IdlFormat>, String> {
    match load_idl_with_diagnostics(file) {
        Ok(idl) => Ok(idl),
        Err(e) => Err(format!("{}", e)),
    }
}

pub fn load_idl(file: &mut File) -> Box<dyn IdlFormat> {
    match load_idl_with_diagnostics(file) {
        Ok(idl) => idl,
        Err(e) => {
            eprintln!("{}", format_user_error(&e));
            std::process::exit(1);
        }
    }
}

/// 带详细诊断的IDL加载函数
pub fn load_idl_with_diagnostics(file: &mut File) -> Result<Box<dyn IdlFormat>, SoloresError> {
    log::debug!("开始IDL解析诊断流程");
    
    // 1. 读取文件内容
    let mut content = String::new();
    file.read_to_string(&mut content).map_err(|e| {
        SoloresError::file_operation_error("读取IDL文件", "IDL file", e)
    })?;
    
    log::debug!("IDL文件大小: {} bytes", content.len());
    
    // 2. 验证JSON格式
    let json_value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| diagnose_json_error(&content, &e))?;
    
    log::debug!("JSON格式验证通过");
    
    // 3. 检查基本结构
    validate_idl_structure(&json_value)?;
    
    log::debug!("IDL基本结构验证通过");
    
    // 4. 尝试不同的IDL格式
    try_parse_idl_formats(&content)
}

/// 尝试解析不同的IDL格式（使用新的二元架构）
fn try_parse_idl_formats(content: &str) -> Result<Box<dyn IdlFormat>, SoloresError> {
    log::debug!("使用新的二元架构解析IDL格式");
    
    // 使用统一的IDL解析接口
    match parse_idl_json(content) {
        Ok(idl_format) => {
            match &idl_format {
                IdlFormatEnum::Anchor(anchor_idl) => {
                    log::info!("✓ 成功加载Anchor IDL格式: {}", anchor_idl.program_name());
                }
                IdlFormatEnum::NonAnchor(non_anchor_idl) => {
                    log::info!("✓ 成功加载NonAnchor IDL格式: {}", non_anchor_idl.program_name());
                }
            }
            Ok(Box::new(idl_format))
        }
        Err(e) => {
            log::error!("IDL格式解析失败");
            
            // 提供详细的错误诊断
            let error_msg = e.to_string();
            if error_msg.contains("duplicate field") {
                let field_name = error_msg
                    .split("duplicate field `")
                    .nth(1)
                    .and_then(|s| s.split('`').next())
                    .unwrap_or("unknown");
                    
                Err(SoloresError::DuplicateFieldError {
                    field: field_name.to_string(),
                    location: "IDL file".to_string(),
                    suggestion: format!(
                        "字段'{}'在IDL文件中重复出现。请手动编辑IDL文件，合并或删除重复的字段定义。\n\
                        常见情况：metadata字段在某些IDL中会重复出现，需要手动合并。",
                        field_name
                    ),
                })
            } else if error_msg.contains("missing field") {
                let field_name = error_msg
                    .split("missing field `")
                    .nth(1)
                    .and_then(|s| s.split('`').next())
                    .unwrap_or("unknown");
                    
                Err(SoloresError::MissingFieldError {
                    field: field_name.to_string(),
                    context: "IDL根对象".to_string(),
                    suggestion: Some(get_missing_field_suggestion(field_name)),
                })
            } else {
                Err(SoloresError::InvalidIdlFormat {
                    details: format!("无法识别的IDL格式: {}", error_msg),
                    expected_format: Some(
                        "支持的格式包括:\n\
                        - Anchor IDL (8字节discriminator)\n\
                        - NonAnchor IDL (1字节discriminator或其他识别方式)".to_string()
                    ),
                })
            }
        }
    }
}

/// 根据缺失字段提供具体建议
fn get_missing_field_suggestion(field_name: &str) -> String {
    match field_name {
        "name" => "IDL文件需要包含程序名称。对于Anchor IDL，应该在metadata.name中；对于Shank IDL，应该在根级别。".to_string(),
        "version" => "建议添加version字段来标识IDL版本。".to_string(),
        "instructions" => "IDL文件应该包含instructions数组来定义程序指令。".to_string(),
        "metadata" => "Anchor IDL需要metadata对象，其中应包含name和version等信息。".to_string(),
        _ => format!("请检查IDL格式规范，确保包含必需的{}字段。", field_name),
    }
}
