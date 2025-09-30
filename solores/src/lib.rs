#![doc = include_str!("../README.md")]

use std::{
    env,
    fs::{self, File, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

use clap::{command, Parser};
use idl_format::{parse_idl_json, IdlFormat, IdlFormatEnum};

use crate::error::{diagnose_json_error, format_user_error, validate_idl_structure, SoloresError};

// Just make all mods pub to allow ppl to use the lib

pub mod cargo; // Cargo.toml 生成功能
pub mod error;
pub mod idl_format;
pub mod minijinja; // MiniJinja 模块化模板系统
                   // pub mod templates;  // 传统模板系统 - 已移除
// unified_library功能已集成到minijinja模块中
pub mod utils;
pub mod workspace; // 新增workspace生成功能
pub mod write_gitignore;
pub mod write_readme;
pub mod write_src;

use cargo::{
    should_use_workspace_cargo_toml, write_fine_grained_cargo_toml,
    write_workspace_member_cargo_toml,
};
use workspace::{add_workspace_member, finalize_workspace, validate_workspace_config};
use minijinja::generator::{auto_group_protocols, UnifiedLibraryConfig};
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
        help = "serde-big-array dependency version for generated crate",
        default_value = "^0.5"
    )]
    pub serde_big_array_vers: String,

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
        default_value = "true",
        help = "generate to_json methods for instructions and accounts (serde feature, enabled by default)"
    )]
    pub generate_to_json: bool,

    #[arg(long, help = "only generate parser code (skip interface generation)")]
    pub parser_only: bool,

    #[arg(long, help = "批量处理模式 - 处理指定目录中的所有IDL文件")]
    pub batch: bool,

    #[arg(
        long,
        help = "批量生成时的基础输出目录",
        default_value = "./batch_output"
    )]
    pub batch_output_dir: PathBuf,

    #[arg(
        long,
        help = "generate test code for parsers (默认不生成测试以减少文件数量)"
    )]
    pub test: bool,

    #[arg(long, help = "生成workspace结构（适用于批量处理）")]
    pub workspace: bool,

    #[arg(long, help = "指定workspace名称", default_value = "solana_workspace")]
    pub workspace_name: String,

    #[arg(
        long,
        help = "批量处理时排除的JSON文件名，支持通配符，多个用逗号分隔",
        default_value = ""
    )]
    pub batch_exclude: String,

    #[arg(
        long,
        help = "批量处理时仅包含的JSON文件名，支持通配符，多个用逗号分隔",
        default_value = ""
    )]
    pub batch_include: String,

    #[arg(
        long,
        help = "不生成空的 [workspace] 表（适用于独立项目）",
        default_value = "true"
    )]
    pub no_empty_workspace: bool,

    #[arg(long, help = "生成统一接口库（将多个IDL合并到单一库中）")]
    pub unified_library: bool,

    #[arg(
        long,
        help = "统一接口库的名称",
        default_value = "solana_protocols"
    )]
    pub unified_library_name: String,
}

/// 获取用于错误显示的绝对路径字符串
/// 优先使用 canonicalize，如果失败则手动构建绝对路径
fn get_absolute_path_for_error(path: &Path) -> String {
    // 先尝试 canonicalize 获取真实绝对路径（处理符号链接等）
    if let Ok(canonical_path) = fs::canonicalize(path) {
        return canonical_path.display().to_string();
    }

    // 如果 canonicalize 失败（通常是因为文件不存在）
    // 手动构建绝对路径用于错误显示
    if path.is_absolute() {
        // 已经是绝对路径，直接返回
        path.display().to_string()
    } else {
        // 相对路径，与当前目录组合，然后尝试清理路径
        match env::current_dir() {
            Ok(current_dir) => {
                let combined_path = current_dir.join(path);

                // 尝试清理路径（去掉 .. 和 . 等）
                // 注意：这种方法对不存在的文件也有效
                clean_path(&combined_path).display().to_string()
            }
            Err(_) => {
                // 如果连当前目录都获取不到，回退到原始路径
                path.display().to_string()
            }
        }
    }
}

/// 清理路径，去掉 .. 和 . 等路径组件
/// 即使文件不存在也能正常工作
fn clean_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            std::path::Component::CurDir => {
                // 忽略当前目录 "."
            }
            std::path::Component::ParentDir => {
                // 处理父目录 ".."
                if !components.is_empty()
                    && components.last() != Some(&std::path::Component::ParentDir)
                {
                    components.pop();
                } else if components.is_empty() {
                    components.push(component);
                }
            }
            _ => {
                components.push(component);
            }
        }
    }

    components.iter().collect()
}

/// 设置日志系统，debug及以上级别输出到文件，终端不输出日志
fn setup_logging() {
    use env_logger::{Builder, Target};
    use log::LevelFilter;

    // 清理旧的日志文件
    let log_file_path = "debug_output.log";
    if std::path::Path::new(log_file_path).exists() {
        let _ = std::fs::remove_file(log_file_path);
    }

    // 创建文件写入器
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_file_path)
        .expect("无法创建日志文件");

    let current_dir = env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "<unknown>".to_string());

    // 获取日志级别
    let log_level = env::var(RUST_LOG_ENV_VAR).unwrap_or_else(|_| "debug".to_string());

    // 配置：所有日志输出到文件，终端不输出日志
    let mut builder = Builder::new();
    builder
        .target(Target::Pipe(Box::new(log_file)))
        .filter_level(LevelFilter::Debug)
        .format(|buf, record| {
            use std::io::Write;
            let timestamp = chrono::Utc::now().format("%H:%M:%S%.3f");
            let file = record.file().unwrap_or("unknown");
            let line = record.line().unwrap_or(0);
            writeln!(
                buf,
                "[{}] {} [{}:{}]: {}",
                timestamp,
                record.level(),
                file,
                line,
                record.args()
            )
        })
        .init();

    // 终端只显示日志文件位置
    println!("🔍 Debug日志输出到: {}/{}", current_dir, log_file_path);
    println!("📊 当前日志级别: {}", log_level);
}

/// The CLI entrypoint
pub fn main() {
    if env::var(RUST_LOG_ENV_VAR).is_err() {
        env::set_var(RUST_LOG_ENV_VAR, "debug")
    }

    // 配置日志输出到文件和终端
    setup_logging();
    log_panics::init();

    let args = Args::parse();
    
    // 🔍 简单的文件路径验证和调试输出
    if !args.batch {
        // 检查单个IDL文件是否可以打开
        if let Err(e) = std::fs::File::open(&args.idl_path) {
            // 立即向终端输出调试信息
            eprintln!("❌ 无法打开IDL文件");
            eprintln!("📁 当前工作目录: {}", 
                env::current_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "<unknown>".to_string())
            );
            eprintln!("📄 尝试打开的文件: {}", args.idl_path.display());
            eprintln!("🔍 错误详情: {}", e);
            std::process::exit(1);
        }
    }

    if args.batch {
        process_batch(args);
    } else {
        if let Err(e) = process_single_file(args) {
            log::error!("处理文件失败: {}", e);
            std::process::exit(1);
        }
    }
}

/// Process a single IDL file (original functionality)
fn process_single_file(mut args: Args) -> Result<(), SoloresError> {
    let mut file = OpenOptions::new()
        .read(true)
        .open(&args.idl_path)
        .map_err(|e| {
            SoloresError::file_operation_error(
                "读取IDL文件",
                get_absolute_path_for_error(&args.idl_path),
                e,
            )
        })?;

    let idl = load_idl_with_diagnostics(&mut file)?;

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
    fs::create_dir_all(args.output_dir.join("src/")).map_err(|e| {
        SoloresError::file_operation_error("创建输出目录", args.output_dir.display().to_string(), e)
    })?;

    // TODO: multithread, 1 thread per generated file
    write_gitignore(&args).map_err(|e| {
        SoloresError::file_operation_error(
            "创建.gitignore文件",
            args.output_dir.join(".gitignore").display().to_string(),
            e,
        )
    })?;

    // Choose appropriate Cargo.toml generation based on workspace mode
    if should_use_workspace_cargo_toml(&args) {
        write_workspace_member_cargo_toml(&args, idl.as_ref()).map_err(|e| {
            SoloresError::file_operation_error(
                "创建workspace Cargo.toml文件",
                args.output_dir.join("Cargo.toml").display().to_string(),
                e,
            )
        })?;
    } else {
        write_fine_grained_cargo_toml(&args, idl.as_ref()).map_err(|e| {
            SoloresError::file_operation_error(
                "创建Cargo.toml文件",
                args.output_dir.join("Cargo.toml").display().to_string(),
                e,
            )
        })?;
    }

    // 调用 write_lib，内部会根据配置选择合适的模板系统
    log::info!("Writing lib.rs for IDL: {}", idl.program_name());
    log::debug!("IDL address: {:?}", idl.program_address());
    write_lib(&args, idl.as_ref()).map_err(|e| {
        SoloresError::file_operation_error(
            "创建lib.rs文件",
            args.output_dir.join("src/lib.rs").display().to_string(),
            e,
        )
    })?;
    write_readme(&args, idl.as_ref()).map_err(|e| {
        SoloresError::file_operation_error(
            "创建README.md文件",
            args.output_dir.join("README.md").display().to_string(),
            e,
        )
    })?;

    // Copy IDL file to output directory
    let idl_dest = args.output_dir.join("idl.json");
    if let Err(e) = std::fs::copy(&args.idl_path, &idl_dest) {
        log::warn!("Failed to copy IDL file: {}", e);
    } else {
        log::info!("IDL file copied to {}", idl_dest.display());
    }

    // Format generated code with prettyplease
    log::debug!("🎨 使用prettyplease格式化生成的代码...");

    // 格式化所有生成的Rust文件
    let src_dir = args.output_dir.join("src");
    if src_dir.exists() {
        format_rust_files_with_prettyplease(&src_dir)?;
        log::info!("✅ 代码格式化完成 (prettyplease)");
    } else {
        log::warn!("⚠️ src目录不存在，跳过格式化");
    }

    log::info!(
        "{} crate written to {}",
        args.output_crate_name,
        args.output_dir.to_string_lossy()
    );

    Ok(())
}

/// Process multiple IDL files in batch mode
fn process_batch(args: Args) {
    log::info!("🚀 启动批量处理模式");
    log::info!("📁 扫描目录: {}", args.idl_path.display());
    log::info!("📁 输出目录: {}", args.batch_output_dir.display());

    // 显示过滤器信息
    if !args.batch_exclude.is_empty() {
        log::info!("🚫 排除模式: {}", args.batch_exclude);
    }
    if !args.batch_include.is_empty() {
        log::info!("✅ 包含模式: {}", args.batch_include);
    }

    // Initialize workspace configuration if enabled
    let mut workspace_config = match validate_workspace_config(&args) {
        Ok(Some(config)) => {
            log::info!("🏗️  Workspace模式已启用: '{}'", config.name);
            Some(config)
        }
        Ok(None) => None,
        Err(e) => {
            log::error!("❌ Workspace配置验证失败: {}", e);
            panic!("Failed to validate workspace configuration: {}", e);
        }
    };

    // Create batch output directory
    if let Err(e) = fs::create_dir_all(&args.batch_output_dir) {
        log::error!("❌ 无法创建批量输出目录: {}", e);
        panic!("Failed to create batch output directory: {}", e);
    }

    // 扫描IDL文件并应用过滤器
    let idl_files = scan_idl_files_with_filters(&args.idl_path, &args);
    if idl_files.is_empty() {
        log::warn!(
            "⚠️  在目录 {} 中未找到匹配的IDL文件",
            args.idl_path.display()
        );
        if !args.batch_exclude.is_empty() || !args.batch_include.is_empty() {
            log::info!("💡 提示: 检查您的过滤模式是否正确");
        }
        return;
    }

    // 检查是否启用统一库模式
    if args.unified_library {
        log::info!("🚀 启用统一库生成模式");
        process_unified_library(args, &idl_files);
        return;
    }

    log::info!(
        "📋 找到 {} 个IDL文件待处理（应用过滤器后）",
        idl_files.len()
    );

    let mut success_count = 0;
    let mut failure_count = 0;
    let mut failed_files = Vec::new();
    let mut generated_crates = Vec::new();

    for (idx, idl_file) in idl_files.iter().enumerate() {
        log::info!(
            "🔄 处理文件 {}/{}: {}",
            idx + 1,
            idl_files.len(),
            idl_file.display()
        );

        match process_single_idl_file(&args, &idl_file) {
            Ok(output_dir) => {
                success_count += 1;
                log::info!("✅ 成功生成: {}", output_dir.display());

                // Add to workspace if enabled
                if let Some(ref mut workspace) = workspace_config {
                    if let Some(crate_name) = output_dir.file_name() {
                        let crate_name_str = crate_name.to_string_lossy().to_string();
                        add_workspace_member(workspace, crate_name_str.clone());
                        generated_crates.push(crate_name_str);
                    }
                }
            }
            Err(e) => {
                failure_count += 1;
                log::error!("❌ 处理失败 {}: {}", idl_file.display(), e);
                failed_files.push((idl_file.clone(), e));
            }
        }
    }

    // Finalize workspace if enabled
    if let Some(workspace) = workspace_config {
        if let Err(e) = finalize_workspace(&workspace) {
            log::error!("❌ Workspace生成失败: {}", e);
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

    if !generated_crates.is_empty() {
        log::info!("📦 生成的crate:");
        for crate_name in &generated_crates {
            log::info!("   - {}", crate_name);
        }
    }

    log::info!("📁 所有生成的库位于: {}", args.batch_output_dir.display());
}

/// 解析文件模式字符串
fn parse_file_patterns(pattern_str: &str) -> Vec<String> {
    if pattern_str.is_empty() {
        return vec![];
    }

    pattern_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// 检查文件名是否匹配模式
fn filename_matches_pattern(filename: &str, pattern: &str) -> bool {
    // 简单通配符匹配
    if pattern.contains('*') {
        // 转换为正则表达式
        let regex_pattern = pattern.replace("*", ".*");
        if let Ok(regex) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
            return regex.is_match(filename);
        }
    }

    // 精确匹配
    filename == pattern
}

/// 扫描IDL文件并应用过滤器
fn scan_idl_files_with_filters(dir_path: &PathBuf, args: &Args) -> Vec<PathBuf> {
    let all_idl_files = scan_idl_files(dir_path);

    // 解析排除和包含模式
    let exclude_patterns = parse_file_patterns(&args.batch_exclude);
    let include_patterns = parse_file_patterns(&args.batch_include);

    // 应用过滤器
    all_idl_files
        .into_iter()
        .filter(|path| {
            let filename = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");

            // 检查排除模式
            if !exclude_patterns.is_empty() {
                for pattern in &exclude_patterns {
                    if filename_matches_pattern(filename, pattern) {
                        log::debug!("🚫 排除文件: {} (匹配模式: {})", filename, pattern);
                        return false;
                    }
                }
            }

            // 检查包含模式
            if !include_patterns.is_empty() {
                for pattern in &include_patterns {
                    if filename_matches_pattern(filename, pattern) {
                        log::debug!("✅ 包含文件: {} (匹配模式: {})", filename, pattern);
                        return true;
                    }
                }
                // 如果有包含模式但都不匹配，则排除
                log::debug!("🚫 排除文件: {} (不匹配任何包含模式)", filename);
                return false;
            }

            true
        })
        .collect()
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
                                        if let Ok(_) =
                                            serde_json::from_str::<serde_json::Value>(&content)
                                        {
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
        Err(e) => {
            let abs_path = get_absolute_path_for_error(&args.idl_path);
            return Err(format!("无法打开文件 {}: {}", abs_path, e));
        }
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

    // Choose appropriate Cargo.toml generation based on workspace mode
    if should_use_workspace_cargo_toml(&args) {
        if let Err(e) = write_workspace_member_cargo_toml(&args, idl.as_ref()) {
            return Err(format!("生成workspace成员Cargo.toml失败: {}", e));
        }
    } else {
        if let Err(e) = write_fine_grained_cargo_toml(&args, idl.as_ref()) {
            return Err(format!("生成Cargo.toml失败: {}", e));
        }
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

    // Format generated code with prettyplease
    log::debug!("🎨 使用prettyplease格式化生成的代码...");

    // 格式化所有生成的Rust文件
    let output_src_dir = args.output_dir.join("src");
    if output_src_dir.exists() {
        if let Err(e) = format_rust_files_with_prettyplease(&output_src_dir) {
            log::warn!("⚠️ prettyplease格式化失败: {}，尝试使用cargo fmt", e);
            // Fallback to cargo fmt if prettyplease fails
            let _ = Command::new("cargo")
                .args(&["fmt"])
                .current_dir(&args.output_dir)
                .output();
        } else {
            log::debug!("✅ 代码格式化完成 (prettyplease)");
        }
    } else {
        log::warn!("⚠️ src目录不存在，跳过格式化");
    }

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
    file.read_to_string(&mut content)
        .map_err(|e| SoloresError::file_operation_error("读取IDL文件", "IDL file", e))?;

    log::debug!("IDL文件大小: {} bytes", content.len());

    // 2. 验证JSON格式
    let json_value: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| diagnose_json_error(&content, &e))?;

    log::debug!("JSON格式验证通过");

    // 3. 检查基本结构
    validate_idl_structure(&json_value)?;

    log::debug!("IDL基本结构验证通过");

    // 4. 尝试不同的IDL格式
    try_parse_idl_formats(&content)
}

/// 加载IDL文件并返回IdlFormatEnum（用于统一库生成）
pub fn load_idl_as_enum(file: &mut File) -> Result<IdlFormatEnum, SoloresError> {
    log::debug!("开始IDL解析诊断流程（返回枚举）");

    // 1. 读取文件内容
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| SoloresError::file_operation_error("读取IDL文件", "IDL file", e))?;

    log::debug!("IDL文件大小: {} bytes", content.len());

    // 2. 验证JSON格式
    let json_value: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| diagnose_json_error(&content, &e))?;

    log::debug!("JSON格式验证通过");

    // 3. 检查基本结构
    validate_idl_structure(&json_value)?;

    log::debug!("IDL基本结构验证通过");

    // 4. 解析为IdlFormatEnum
    parse_idl_json(&content).map_err(|e| {
        log::error!("IDL格式解析失败");
        let error_msg = e.to_string();
        if error_msg.contains("duplicate field") {
            let field_name = error_msg
                .split("duplicate field `")
                .nth(1)
                .and_then(|s| s.split('`').next())
                .unwrap_or("unknown");
            SoloresError::DuplicateFieldError {
                field: field_name.to_string(),
                location: "IDL parsing".to_string(),
                suggestion: format!(
                    "IDL文件中存在重复的字段 '{}'。请检查:\n\
                     1. 指令定义中是否有重复的参数名\n\
                     2. 账户定义中是否有重复的字段名\n\
                     3. 类型定义中是否有重复的成员名",
                    field_name
                ),
            }
        } else if error_msg.contains("missing field") {
            let field_name = error_msg
                .split("missing field `")
                .nth(1)
                .and_then(|s| s.split('`').next())
                .unwrap_or("unknown");
            SoloresError::MissingFieldError {
                field: field_name.to_string(),
                context: "IDL parsing".to_string(),
                suggestion: Some(get_missing_field_suggestion(field_name)),
            }
        } else {
            SoloresError::InvalidIdlFormat {
                details: format!("无法识别的IDL格式: {}", error_msg),
                expected_format: Some(
                    "支持的格式包括:\n\
                    - Anchor IDL (8字节discriminator)\n\
                    - NonAnchor IDL (1字节discriminator或其他识别方式)"
                        .to_string(),
                ),
            }
        }
    })
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
                    log::info!(
                        "✓ 成功加载NonAnchor IDL格式: {}",
                        non_anchor_idl.program_name()
                    );
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
                        - NonAnchor IDL (1字节discriminator或其他识别方式)"
                            .to_string(),
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

/// 使用prettyplease格式化目录中的所有Rust文件
fn format_rust_files_with_prettyplease(dir: &Path) -> Result<(), SoloresError> {
    use std::fs;

    let entries = fs::read_dir(dir).map_err(|e| SoloresError::FileOperationError {
        operation: "读取目录".to_string(),
        path: dir.display().to_string(),
        current_dir: std::env::current_dir()
            .ok()
            .map(|p| p.display().to_string()),
        resolved_path: None,
        source: e,
        suggestion: Some("检查目录权限".to_string()),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| SoloresError::FileOperationError {
            operation: "读取目录项".to_string(),
            path: dir.display().to_string(),
            current_dir: std::env::current_dir()
                .ok()
                .map(|p| p.display().to_string()),
            resolved_path: None,
            source: e,
            suggestion: Some("检查目录权限".to_string()),
        })?;

        let path = entry.path();

        if path.is_dir() {
            // 递归处理子目录
            format_rust_files_with_prettyplease(&path)?;
        } else if path.extension().map_or(false, |ext| ext == "rs") {
            // 格式化Rust文件
            format_single_rust_file(&path)?;
        }
    }

    Ok(())
}

/// 使用prettyplease格式化单个Rust文件
fn format_single_rust_file(file_path: &Path) -> Result<(), SoloresError> {
    use std::fs;

    // 读取文件内容
    let content = fs::read_to_string(file_path).map_err(|e| SoloresError::FileOperationError {
        operation: "读取文件".to_string(),
        path: file_path.display().to_string(),
        current_dir: std::env::current_dir()
            .ok()
            .map(|p| p.display().to_string()),
        resolved_path: None,
        source: e,
        suggestion: Some("检查文件权限".to_string()),
    })?;

    // 解析为语法树
    match syn::parse_file(&content) {
        Ok(syntax_tree) => {
            // 使用prettyplease格式化
            let formatted = prettyplease::unparse(&syntax_tree);

            // 写回文件
            fs::write(file_path, formatted).map_err(|e| SoloresError::FileOperationError {
                operation: "写入格式化文件".to_string(),
                path: file_path.display().to_string(),
                current_dir: std::env::current_dir()
                    .ok()
                    .map(|p| p.display().to_string()),
                resolved_path: None,
                source: e,
                suggestion: Some("检查文件权限".to_string()),
            })?;

            log::debug!("✅ 格式化完成: {}", file_path.display());
        }
        Err(e) => {
            log::warn!("⚠️ 跳过格式化 {} (语法错误): {}", file_path.display(), e);
        }
    }

    Ok(())
}

/// 处理统一库生成
fn process_unified_library(args: Args, idl_files: &[PathBuf]) {
    log::info!("🔄 开始统一库生成流程");
    
    // 自动分组协议
    let protocol_groups = match auto_group_protocols(idl_files) {
        Ok(groups) => groups,
        Err(e) => {
            log::error!("❌ 协议分组失败: {}", e);
            return;
        }
    };

    if protocol_groups.is_empty() {
        log::warn!("⚠️  没有找到任何协议组");
        return;
    }

    // 创建统一库配置
    let config = UnifiedLibraryConfig {
        library_name: args.unified_library_name.clone(),
        output_dir: args.batch_output_dir.clone(),
        protocol_groups,
        base_args: args,
    };

    // 生成统一库
    match minijinja::generator::MinijinjaTemplateGenerator::generate_unified_library(&config) {
        Ok(()) => {
            log::info!("✅ 统一库生成成功");
            log::info!("📁 输出位置: {}/{}", config.output_dir.display(), config.library_name);
        }
        Err(e) => {
            log::error!("❌ 统一库生成失败: {}", e);
        }
    }
}
