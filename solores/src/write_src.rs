use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::quote;
use std::{io::Write, path::Path, time::Instant};

use crate::{
    error::{SoloresError, handle_file_operation},
    idl_format::{IdlFormat, IdlFormatEnum}, 
    utils::open_file_create_overwrite, 
    Args
};

const DEFAULT_PROGRAM_ID_STR: &str = "TH1S1SNoTAVAL1DPUBKEYDoNoTUSE11111111111111";

/// 检查是否应该使用Askama模板系统

/// 检查是否应该使用MiniJinja模板系统
fn should_use_minijinja(_args: &Args) -> bool {
    // CLI参数优先级最高（将来可能添加）
    // if args.use_minijinja {
    //     log::info!("🔧 通过 --use-minijinja 参数启用 MiniJinja 模板系统");
    //     return true;
    // }
    
    // 环境变量检查
    if std::env::var("SOLORES_USE_MINIJINJA").unwrap_or_default() == "true" {
        log::info!("🔧 通过 SOLORES_USE_MINIJINJA 环境变量启用 MiniJinja 模板系统");
        return true;
    }
    
    log::debug!("🔧 未启用 MiniJinja 模板系统");
    false
}


const MAX_BASE58_LEN: usize = 44;
const PUBKEY_BYTES_SIZE: usize = 32;

/// Copied from solana_program::Pubkey::from_str()
/// so that we dont have to have solana_program as a dep
fn is_valid_pubkey(s: &str) -> bool {
    if s.len() > MAX_BASE58_LEN {
        return false;
    }
    let pubkey_vec = match bs58::decode(s).into_vec() {
        Ok(v) => v,
        Err(_) => return false,
    };
    if pubkey_vec.len() != PUBKEY_BYTES_SIZE {
        return false;
    }
    true
}
/// 带诊断功能的模块生成器
pub fn generate_module_with_diagnostics(
    module_name: &str,
    generator: impl FnOnce() -> Result<TokenStream, String>
) -> Result<TokenStream, SoloresError> {
    log::debug!("🔧 开始生成模块: {}", module_name);
    
    let start_time = Instant::now();
    let result = generator().map_err(|reason| {
        log::error!("❌ 模块{}生成失败: {}", module_name, reason);
        SoloresError::CodeGenError {
            module: module_name.to_string(),
            reason,
            context: None,
        }
    })?;
    
    let duration = start_time.elapsed();
    log::debug!("✅ 模块{}生成完成，耗时: {:?}", module_name, duration);
    
    // 验证生成的代码语法
    if let Err(e) = syn::parse2::<syn::File>(result.clone()) {
        // 输出TokenStream进行调试
        log::error!("❌ 语法验证失败的TokenStream内容:");
        log::error!("=== TokenStream开始 ===");
        log::error!("{}", result.to_string());
        log::error!("=== TokenStream结束 ===");
        log::error!("语法错误详情: {}", e);
        
        return Err(SoloresError::CodeGenError {
            module: module_name.to_string(),
            reason: format!("生成的代码语法错误: {}", e),
            context: Some("文件写入前语法验证".to_string()),
        });
    }
    
    log::debug!("✓ 模块{}代码语法验证通过", module_name);
    Ok(result)
}

/// 验证程序ID的有效性
fn validate_program_id(args: &Args, idl: &dyn IdlFormat) -> Result<(), SoloresError> {
    if let Some(program_id) = &args.program_id {
        if !is_valid_pubkey(program_id) {
            return Err(SoloresError::ValidationError {
                message: "提供的程序ID格式无效".to_string(),
                field_path: Some("program_id".to_string()),
                expected: Some("有效的Base58编码的公钥 (44个字符以内)".to_string()),
                actual: Some(program_id.clone()),
            });
        }
    }
    
    if let Some(idl_address) = idl.program_address() {
        if !is_valid_pubkey(idl_address) {
            log::warn!("IDL中的程序地址格式可能无效: {}", idl_address);
        }
    }
    
    Ok(())
}

/// 创建输出目录
fn create_output_directories(args: &Args) -> Result<(), SoloresError> {
    let src_dir = args.output_dir.join("src");
    
    handle_file_operation("创建目录", &args.output_dir, || {
        std::fs::create_dir_all(&args.output_dir)
    })?;
    
    handle_file_operation("创建src目录", &src_dir, || {
        std::fs::create_dir_all(&src_dir)
    })?;
    
    log::debug!("✓ 输出目录创建成功: {}", args.output_dir.display());
    Ok(())
}

/// writes the lib.rs file
pub fn write_lib(args: &Args, idl: &dyn IdlFormat) -> std::io::Result<()> {
    match write_lib_with_diagnostics(args, idl) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("{}", crate::error::format_user_error(&e));
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string()
            ))
        }
    }
}


/// 使用 MiniJinja 模板系统生成代码
pub fn write_lib_with_minijinja(args: &Args, _idl: &dyn IdlFormat) -> Result<(), SoloresError> {
    log::info!("🔧 使用 MiniJinja 模板系统生成代码");
    
    // 通过重新解析 IDL 文件来获取完整数据
    let idl_format = convert_dyn_idl_to_enum_with_reparse(args)?;
    
    // 创建 MiniJinja 模板生成器
    let mut generator = crate::minijinja::MinijinjaTemplateGenerator::new(idl_format)?;
    
    // 生成多文件架构
    generator.generate_multi_file_structure(
        &args.output_dir,
        args.generate_to_json, // 使用generate_to_json作为serde特性标志
        args.generate_parser,
    )?;
    
    // 复制IDL文件到输出目录
    if let Ok(content) = std::fs::read_to_string(&args.idl_path) {
        let idl_output_path = args.output_dir.join("idl.json");
        std::fs::write(&idl_output_path, content).map_err(|e| SoloresError::FileOperationError {
            operation: "copy IDL file".to_string(),
            path: idl_output_path.display().to_string(),
            current_dir: std::env::current_dir().ok().map(|p| p.display().to_string()),
            resolved_path: None,
            source: e,
            suggestion: Some("检查目录权限".to_string()),
        })?;
    }
    
    log::info!("✅ MiniJinja 代码生成完成");
    Ok(())
}


/// 向后兼容的包装函数
pub fn write_lib_with_diagnostics(args: &Args, idl: &dyn IdlFormat) -> Result<(), SoloresError> {
    // 优先级：MiniJinja > 传统系统
    if should_use_minijinja(args) {
        log::info!("🔧 启用 MiniJinja 模板系统 - 现代化多文件生成");
        write_lib_with_minijinja(args, idl)
    } else {
        log::info!("🔧 使用传统模板系统");
        write_lib_with_diagnostics_legacy(args, idl)
    }
}

/// 将 dyn IdlFormat 转换为 IdlFormatEnum
/// 通过重新解析 IDL 文件来获取完整数据
fn convert_dyn_idl_to_enum_with_reparse(args: &Args) -> Result<IdlFormatEnum, SoloresError> {
    log::debug!("通过重新解析 IDL 文件获取完整数据");
    
    // 重新读取和解析 IDL 文件
    let content = std::fs::read_to_string(&args.idl_path)
        .map_err(|e| SoloresError::FileOperationError {
            operation: "read IDL file for template system".to_string(),
            path: args.idl_path.to_string_lossy().to_string(),
            current_dir: std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string()),
            resolved_path: Some(args.idl_path.to_string_lossy().to_string()),
            source: e,
            suggestion: Some("检查文件路径是否正确并且文件可读".to_string()),
        })?;
    
    // 使用新的解析器直接获取 IdlFormatEnum
    match crate::idl_format::parse_idl_json(&content) {
        Ok(idl_format) => {
            log::info!("✅ 成功重新解析 IDL 文件用于模板系统");
            Ok(idl_format)
        }
        Err(e) => {
            log::error!("❌ 重新解析 IDL 文件失败: {}", e);
            Err(SoloresError::IdlParseError {
                message: format!("Failed to parse IDL for Askama: {}", e),
                line: None,
                column: None,
                file_path: Some(args.idl_path.clone()),
            })
        }
    }
}


pub fn write_lib_with_diagnostics_legacy(args: &Args, idl: &dyn IdlFormat) -> Result<(), SoloresError> {
    log::info!("🚀 开始为程序{}生成lib.rs", idl.program_name());
    log::debug!("程序版本: {}", idl.program_version());
    
    // 1. 验证程序ID
    validate_program_id(args, idl)?;
    
    // 2. 创建输出目录
    create_output_directories(args)?;
    
    // 3. 生成程序ID声明 - 使用pubkey!替换declare_id!
    let program_id = get_program_id(args, idl);
    log::debug!("使用程序ID: {}", program_id);
    
    let mut contents = quote! {
        // 屏蔽生成代码的常见警告
        #![allow(unused_imports)]
        #![allow(dead_code)]
        #![allow(unused_variables)]
        
        use solana_pubkey::{pubkey, Pubkey};
        
        pub static ID: Pubkey = pubkey!(#program_id);
        
        /// 获取程序ID
        pub fn id() -> Pubkey {
            ID
        }
    };
    
    // 4. 生成模块
    let modules = idl.modules(args);
    log::info!("计划生成{}个模块", modules.len());
    
    let mut has_parsers = false;
    
    for (i, module) in modules.iter().enumerate() {
        let module_name = module.name();
        log::debug!("🔄 处理模块 {}/{}: {}", i+1, modules.len(), module_name);
        
        let is_parser = module_name.ends_with("_parser");
        
        if is_parser {
            has_parsers = true;
        } else {
            // 只有非parser模块才添加到lib.rs中
            let module_ident = Ident::new(module_name, Span::call_site());
            contents.extend(quote! {
                pub mod #module_ident;
                // 不再使用通配符导出，用户需要使用完整路径访问模块内容
                // 例如: use crate::instructions::CreatePool; 
            });
        }
        
        // 生成模块文件 - 统一使用多文件架构
        let module_result = generate_multi_file_module(args, module.as_ref(), module_name);
        
        match module_result {
            Ok(()) => {
                log::debug!("✅ 模块{}生成成功", module_name);
            }
            Err(e) => {
                log::error!("❌ 模块{}生成失败: {}", module_name, e);
                return Err(e);
            }
        }
    }
    
    // 5. 如果有parser模块，添加parsers模块声明
    if has_parsers {
        contents.extend(quote! {
            pub mod parsers;
            // 避免全局导入歧义，只导出parsers模块而不使用通配符
        });
        
        // 生成parsers/mod.rs
        generate_parsers_mod_file(args)?;
    }
    
    // 6. 写入lib.rs文件
    write_lib_file(args, contents)?;
    
    log::info!("🎉 所有模块生成完成");
    Ok(())
}

/// 获取程序ID
pub fn get_program_id<'a>(args: &'a Args, idl: &'a dyn IdlFormat) -> &'a str {
    let user_provided_id_opt =
        args.program_id
            .as_ref()
            .and_then(|s| if is_valid_pubkey(s) { Some(s) } else { None });
    user_provided_id_opt
        .map(|string| string.as_ref())
        .unwrap_or_else(|| {
            idl.program_address().unwrap_or_else(|| {
                log::warn!(
                    "program address not in IDL, setting to default: {}",
                    DEFAULT_PROGRAM_ID_STR
                );
                DEFAULT_PROGRAM_ID_STR
            })
        })
}

/// 生成多文件模块
fn generate_multi_file_module(
    args: &Args, 
    module: &dyn crate::idl_format::IdlCodegenModule, 
    module_name: &str
) -> Result<(), SoloresError> {
    log::debug!("📁 生成多文件模块: {}", module_name);
    
    // 检查是否是根目录单文件（如errors.rs）
    // 直接使用IdlCodegenModule的has_multiple_files方法来判断
    let has_multiple = module.has_multiple_files();
    let is_root_file = !has_multiple && module_name == "errors";
    log::debug!("🔍 模块{}的has_multiple_files()结果: {}, is_root_file: {}", module_name, has_multiple, is_root_file);
    
    if is_root_file {
        // 根目录单文件模式：直接生成到src/目录下
        log::debug!("🗂️  生成根目录单文件: {}", module_name);
        
        for (filename, file_contents) in module.gen_files() {
            let file_path = format!("src/{}", filename);
            write_src_file_with_diagnostics(args, &file_path, file_contents)?;
        }
    } else {
        // 目录多文件模式：创建模块目录
        let module_dir = args.output_dir.join("src").join(module_name);
        handle_file_operation("创建模块目录", &module_dir, || {
            std::fs::create_dir_all(&module_dir)
        })?;
        
        // 生成mod.rs文件
        let mod_contents = module.gen_mod_file();
        write_src_file_with_diagnostics(args, &format!("src/{}/mod.rs", module_name), mod_contents)?;
        
        // 生成各个文件
        for (filename, file_contents) in module.gen_files() {
            let file_path = format!("src/{}/{}", module_name, filename);
            write_src_file_with_diagnostics(args, &file_path, file_contents)?;
        }
    }
    
    log::debug!("✅ 多文件模块{}生成完成", module_name);
    Ok(())
}


/// 生成parsers/mod.rs文件
fn generate_parsers_mod_file(args: &Args) -> Result<(), SoloresError> {
    log::debug!("📄 生成parsers/mod.rs文件");
    
    // Check which parser files actually exist
    let accounts_exists = args.output_dir.join("src/parsers/accounts.rs").exists();
    let instructions_exists = args.output_dir.join("src/parsers/instructions.rs").exists();
    
    let mut mod_declarations = Vec::new();
    let mut re_exports = Vec::new();
    
    if accounts_exists {
        mod_declarations.push(quote! { pub mod accounts; });
        re_exports.push(quote! { pub use accounts::try_unpack_account; });
    }
    
    if instructions_exists {
        mod_declarations.push(quote! { pub mod instructions; });
        re_exports.push(quote! { pub use instructions::parse_instruction; });
    }
    
    let parsers_mod_contents = quote! {
        #(#mod_declarations)*
        
        // 只导出主要的解析函数，避免discriminator常量冲突
        #(#re_exports)*
        
        // 如果需要访问具体的解析器常量或枚举，请直接使用 parsers::instructions::* 或 parsers::accounts::*
    };
    
    write_src_file_with_diagnostics(args, "src/parsers/mod.rs", parsers_mod_contents)?;
    log::debug!("✅ parsers/mod.rs生成完成");
    Ok(())
}

/// 写入lib.rs文件
fn write_lib_file(args: &Args, contents: proc_macro2::TokenStream) -> Result<(), SoloresError> {
    log::debug!("📄 写入lib.rs文件");
    write_src_file_with_diagnostics(args, "src/lib.rs", contents)?;
    log::debug!("✅ lib.rs文件写入完成");
    Ok(())
}

/// 带诊断功能的文件写入函数
fn write_src_file_with_diagnostics<P: AsRef<Path>>(
    args: &Args,
    src_file_path: P,
    contents: TokenStream,
) -> Result<(), SoloresError> {
    let path = src_file_path.as_ref();
    log::debug!("📝 写入文件: {}", path.display());
    
    // 检查写入前的use crate::*数量
    let content_str = contents.to_string();
    let use_crate_count = content_str.matches("use crate::*").count();
    log::debug!("📄 写入前检查 - use crate::* 出现次数: {}", use_crate_count);
    
    if use_crate_count > 1 {
        log::warn!("⚠️ 检测到重复导入！详细内容:");
        for (i, line) in content_str.lines().enumerate() {
            if line.contains("use crate::*") {
                log::warn!("  第{}行: {}", i+1, line.trim());
            }
        }
    }
    
    let sanitized_contents = sanitize_tokens(contents);
    
    // 验证生成的内容语法
    if let Err(e) = syn::parse2::<syn::File>(sanitized_contents.clone()) {
        // 输出TokenStream进行调试 - 第二个验证点
        log::error!("❌ 第二个验证点语法失败的TokenStream内容:");
        log::error!("=== 第二个验证点TokenStream开始 ===");
        log::error!("{}", sanitized_contents.to_string());
        log::error!("=== 第二个验证点TokenStream结束 ===");
        log::error!("第二个验证点语法错误详情: {}", e);
        
        // 写入调试文件以便详细分析
        let debug_file_path = std::path::Path::new("/tmp/debug_tokenstream.rs");
        let code = prettyplease::unparse(&syn::parse2(sanitized_contents.clone()).unwrap_or_else(|_| {
            // 如果无法解析为 syn::File，尝试将其作为单个 TokenStream 输出
            syn::parse_str::<syn::File>(&format!("mod debug {{ {} }}", sanitized_contents.to_string())).unwrap()
        }));
        std::fs::write(debug_file_path, code).ok();
        log::error!("调试文件已写入: {}", debug_file_path.display());
        
        return Err(SoloresError::CodeGenError {
            module: path.display().to_string(),
            reason: format!("生成的代码语法错误: {}", e),
            context: Some("文件写入前语法验证".to_string()),
        });
    }
    
    let code = prettyplease::unparse(&syn::parse2(sanitized_contents).unwrap());
    
    // 添加文件头部注释
    let header = generate_file_header();
    let final_code = format!("{}{}", header, code);
    
    let full_path = args.output_dir.join(path);
    
    handle_file_operation("写入文件", &full_path, || {
        let mut file = open_file_create_overwrite(&full_path)?;
        file.write_all(final_code.as_bytes())?;
        file.flush()
    })?;
    
    log::debug!("✅ 文件写入完成: {}", path.display());
    Ok(())
}


/// 生成文件头部注释
fn generate_file_header() -> String {
    let now = chrono::Utc::now().format("%Y-%m-%d").to_string();
    format!(
        "//! This file was automatically generated by Solores
//! Solana IDL to Rust interface generator
//!
//! GitHub: https://github.com/cpkt9762/solores
//!
//! Generated on: {}
//!
//! DO NOT EDIT - This file is auto-generated
//! Changes will be lost when regenerating from IDL

",
        now
    )
}

fn sanitize_tokens(input: TokenStream) -> TokenStream {
    input.into_iter().map(sanitize_token).collect()
}

fn sanitize_token(token: TokenTree) -> TokenTree {
    match token {
        TokenTree::Group(group) => {
            let content = sanitize_tokens(group.stream());
            TokenTree::Group(proc_macro2::Group::new(group.delimiter(), content))
        }
        TokenTree::Ident(ident) if ident == "type" => {
            let raw_type = quote! {  type };
            raw_type.into_iter().next().unwrap()
        }
        _ => token,
    }
}
