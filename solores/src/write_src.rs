
use crate::{
    error::SoloresError,
    idl_format::{IdlFormat, IdlFormatEnum}, 
    Args
};

const DEFAULT_PROGRAM_ID_STR: &str = "TH1S1SNoTAVAL1DPUBKEYDoNoTUSE11111111111111";

/// 检查是否应该使用MiniJinja模板系统
/// 现在固定使用MiniJinja模板系统，保留函数接口以便将来扩展
#[allow(dead_code)]
fn should_use_minijinja(_args: &Args) -> bool {
    log::debug!("🔧 使用 MiniJinja 模板系统（唯一选项）");
    true
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


/// 使用 MiniJinja 模板系统生成代码（默认模板系统）
pub fn write_lib_with_minijinja(args: &Args, _idl: &dyn IdlFormat) -> Result<(), SoloresError> {
    log::info!("🚀 使用 MiniJinja 模板系统生成代码（默认）");
    
    // 通过重新解析 IDL 文件来获取完整数据
    let idl_format = convert_dyn_idl_to_enum_with_reparse(args)?;
    
    // 创建 MiniJinja 模板生成器
    let mut generator = crate::minijinja::MinijinjaTemplateGenerator::new(idl_format)?;
    
    // 生成多文件架构
    generator.generate_multi_file_structure(
        &args.output_dir,
        args.generate_to_json, // 使用generate_to_json作为serde特性标志
        args.generate_parser,
        args.no_empty_workspace,
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


/// 模板系统选择的主要入口函数
pub fn write_lib_with_diagnostics(args: &Args, idl: &dyn IdlFormat) -> Result<(), SoloresError> {
    // 现在直接使用 MiniJinja 模板系统
    log::info!("🚀 使用 MiniJinja 模板系统 - 现代化多文件架构生成");
    write_lib_with_minijinja(args, idl)
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





