//! 统一的错误处理系统
//! 
//! 提供详细的错误诊断和用户友好的错误信息

use std::path::PathBuf;
use thiserror::Error;

/// Solores的统一错误类型
#[derive(Debug, Error)]
pub enum SoloresError {
    #[error("IDL parsing failed: {message}")]
    IdlParseError { 
        message: String, 
        line: Option<usize>, 
        column: Option<usize>,
        file_path: Option<PathBuf>,
    },
    
    #[error("Invalid JSON structure: {field} at {location}")]
    JsonStructureError { 
        field: String, 
        location: String,
        suggestion: Option<String>,
    },
    
    #[error("File operation failed: {operation} on {path}")]
    FileOperationError { 
        operation: String, 
        path: String, 
        current_dir: Option<String>,
        resolved_path: Option<String>,
        source: std::io::Error,
        suggestion: Option<String>,
    },
    
    #[error("Code generation failed for {module}: {reason}")]
    CodeGenError { 
        module: String, 
        reason: String,
        context: Option<String>,
    },
    
    #[error("Invalid IDL format: {details}")]
    InvalidIdlFormat { 
        details: String,
        expected_format: Option<String>,
    },
    
    #[error("Missing required field: {field} in {context}")]
    MissingFieldError { 
        field: String, 
        context: String,
        suggestion: Option<String>,
    },
    
    #[error("Type conversion failed: {from_type} to {to_type} in {context}")]
    TypeConversionError { 
        from_type: String, 
        to_type: String, 
        context: String 
    },
    
    #[error("Validation failed: {message}")]
    ValidationError {
        message: String,
        field_path: Option<String>,
        expected: Option<String>,
        actual: Option<String>,
    },
    
    #[error("Duplicate field found: {field} in {location}")]
    DuplicateFieldError {
        field: String,
        location: String,
        suggestion: String,
    },
}

impl SoloresError {
    /// 创建IDL解析错误
    pub fn idl_parse_error(message: impl Into<String>) -> Self {
        Self::IdlParseError {
            message: message.into(),
            line: None,
            column: None,
            file_path: None,
        }
    }
    
    /// 创建带位置信息的IDL解析错误
    pub fn idl_parse_error_with_location(
        message: impl Into<String>, 
        line: usize, 
        column: usize
    ) -> Self {
        Self::IdlParseError {
            message: message.into(),
            line: Some(line),
            column: Some(column),
            file_path: None,
        }
    }
    
    /// 创建文件操作错误
    pub fn file_operation_error(
        operation: impl Into<String>,
        path: impl Into<String>,
        source: std::io::Error
    ) -> Self {
        let path_str = path.into();
        let current_dir = std::env::current_dir()
            .ok()
            .map(|d| d.display().to_string());
        let resolved_path = if let Some(ref current) = current_dir {
            let current_path = std::path::PathBuf::from(current);
            Some(current_path.join(&path_str).display().to_string())
        } else {
            None
        };
        
        Self::FileOperationError {
            operation: operation.into(),
            path: path_str,
            current_dir,
            resolved_path,
            source,
            suggestion: None,
        }
    }
    
    /// 添加建议信息
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        match &mut self {
            Self::JsonStructureError { suggestion: s, .. } |
            Self::FileOperationError { suggestion: s, .. } |
            Self::MissingFieldError { suggestion: s, .. } => {
                *s = Some(suggestion.into());
            }
            _ => {}
        }
        self
    }
}

/// 诊断JSON错误并提供有用的错误信息
pub fn diagnose_json_error(content: &str, error: &serde_json::Error) -> SoloresError {
    let line = error.line();
    let column = error.column();
    
    let message = match error.classify() {
        serde_json::error::Category::Syntax => {
            let suggestion = suggest_syntax_fix(content, line, column);
            format!("JSON语法错误: {}. {}", error, suggestion)
        }
        serde_json::error::Category::Data => {
            format!("数据类型错误: {}. {}", error, suggest_data_fix(error))
        }
        serde_json::error::Category::Eof => {
            "JSON文件意外结束，可能缺少闭合括号或引号".to_string()
        }
        _ => format!("JSON解析错误: {}", error)
    };
    
    SoloresError::idl_parse_error_with_location(message, line, column)
}

/// 根据语法错误位置提供修复建议
fn suggest_syntax_fix(content: &str, line: usize, column: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    
    if line == 0 || line > lines.len() {
        return "请检查JSON语法".to_string();
    }
    
    let error_line = lines[line - 1];
    let chars: Vec<char> = error_line.chars().collect();
    
    if column > 0 && column <= chars.len() {
        let context_start = (column.saturating_sub(10)).max(0);
        let context_end = (column + 10).min(chars.len());
        let context: String = chars[context_start..context_end].iter().collect();
        
        // 检查常见的语法错误
        if chars.get(column.saturating_sub(1)) == Some(&',') {
            return format!("在'{}...'附近可能有多余的逗号", context);
        }
        if chars.get(column.saturating_sub(1)) == Some(&'"') {
            return format!("在'{}...'附近可能缺少闭合引号", context);
        }
        
        return format!("检查'{}...'附近的语法", context);
    }
    
    "请检查JSON语法，确保所有括号和引号正确配对".to_string()
}

/// 根据数据错误提供修复建议
fn suggest_data_fix(error: &serde_json::Error) -> String {
    let error_str = error.to_string();
    
    if error_str.contains("duplicate field") {
        if let Some(field_name) = extract_duplicate_field_name(&error_str) {
            return format!("字段'{}'重复出现，请合并或删除重复的字段", field_name);
        }
        return "存在重复字段，请检查并合并重复的字段定义".to_string();
    }
    
    if error_str.contains("missing field") {
        if let Some(field_name) = extract_missing_field_name(&error_str) {
            return format!("缺少必需字段'{}'，请添加该字段", field_name);
        }
        return "缺少必需字段，请检查IDL格式规范".to_string();
    }
    
    if error_str.contains("invalid type") {
        return "字段类型不匹配，请检查字段值的类型是否正确".to_string();
    }
    
    "数据格式错误，请检查字段值是否符合预期类型".to_string()
}

/// 从错误信息中提取重复字段名
fn extract_duplicate_field_name(error_str: &str) -> Option<String> {
    error_str
        .split("duplicate field `")
        .nth(1)?
        .split('`')
        .next()
        .map(|s| s.to_string())
}

/// 从错误信息中提取缺失字段名
fn extract_missing_field_name(error_str: &str) -> Option<String> {
    error_str
        .split("missing field `")
        .nth(1)?
        .split('`')
        .next()
        .map(|s| s.to_string())
}

/// 格式化用户友好的错误信息
pub fn format_user_error(error: &SoloresError) -> String {
    match error {
        SoloresError::IdlParseError { message, line, column, file_path } => {
            let mut output = format!("❌ IDL解析失败:\n{}", message);
            
            if let (Some(l), Some(c)) = (line, column) {
                output.push_str(&format!("\n📍 位置: 第{}行第{}列", l, c));
            }
            
            if let Some(path) = file_path {
                output.push_str(&format!("\n📁 文件: {}", path.display()));
            }
            
            output.push_str("\n💡 建议: 请检查JSON语法，确保所有括号和引号正确配对");
            output
        }
        
        SoloresError::FileOperationError { operation, path, current_dir, resolved_path, source, suggestion } => {
            let mut output = format!("❌ 文件操作失败:\n操作: {}\n请求路径: {}", operation, path);
            
            // 显示当前工作目录
            if let Some(cwd) = current_dir {
                output.push_str(&format!("\n当前目录: {}", cwd));
            }
            
            // 显示解析后的完整路径
            if let Some(resolved) = resolved_path {
                output.push_str(&format!("\n解析路径: {}", resolved));
            }
            
            // 根据错误类型提供具体建议
            let specific_suggestion = match source.kind() {
                std::io::ErrorKind::NotFound => {
                    if let Some(cwd) = current_dir {
                        if cwd.contains("test_output") || cwd.contains("batch_output") {
                            "您当前在生成的项目目录中，请使用 'cd /Users/pingzi/Developer/work/solana/solores' 回到项目根目录"
                        } else {
                            "文件或目录不存在，请检查路径是否正确"
                        }
                    } else {
                        "文件或目录不存在，请检查路径是否正确"
                    }
                },
                std::io::ErrorKind::PermissionDenied => "权限被拒绝，请检查文件权限设置",
                std::io::ErrorKind::AlreadyExists => "文件已存在，请选择不同的路径或删除现有文件",
                _ => "请检查文件系统状态和权限",
            };
            
            output.push_str(&format!("\n💡 建议: {}", 
                suggestion.as_deref().unwrap_or(specific_suggestion)));
            output
        }
        
        SoloresError::CodeGenError { module, reason, context } => {
            let mut output = format!("❌ 代码生成失败:\n模块: {}\n原因: {}", module, reason);
            
            if let Some(ctx) = context {
                output.push_str(&format!("\n📝 上下文: {}", ctx));
            }
            
            output.push_str(&format!("\n💡 建议: 请检查IDL文件中{}模块的定义", module));
            output
        }
        
        SoloresError::JsonStructureError { field, location, suggestion } => {
            let mut output = format!("❌ JSON结构错误:\n字段: {}\n位置: {}", field, location);
            
            if let Some(s) = suggestion {
                output.push_str(&format!("\n💡 建议: {}", s));
            }
            output
        }
        
        SoloresError::MissingFieldError { field, context, suggestion } => {
            let mut output = format!("❌ 缺少必需字段:\n字段: {}\n上下文: {}", field, context);
            
            if let Some(s) = suggestion {
                output.push_str(&format!("\n💡 建议: {}", s));
            } else {
                output.push_str(&format!("\n💡 建议: 请在{}中添加{}字段", context, field));
            }
            output
        }
        
        SoloresError::DuplicateFieldError { field, location, suggestion } => {
            format!("❌ 重复字段错误:\n字段: {}\n位置: {}\n💡 建议: {}", 
                field, location, suggestion)
        }
        
        _ => format!("❌ 错误: {}", error)
    }
}

/// 验证IDL基本结构
pub fn validate_idl_structure(json: &serde_json::Value) -> Result<(), SoloresError> {
    let obj = json.as_object().ok_or_else(|| {
        SoloresError::InvalidIdlFormat {
            details: "IDL必须是一个JSON对象".to_string(),
            expected_format: Some("根级别应该是一个JSON对象 {}".to_string()),
        }
    })?;
    
    // 检查版本字段（如果存在）
    if let Some(version) = obj.get("version") {
        if !version.is_string() {
            return Err(SoloresError::JsonStructureError {
                field: "version".to_string(),
                location: "root".to_string(),
                suggestion: Some("version字段应该是字符串类型".to_string()),
            });
        }
    }
    
    // 检查metadata或name字段
    if !obj.contains_key("metadata") && !obj.contains_key("name") {
        return Err(SoloresError::MissingFieldError {
            field: "metadata or name".to_string(),
            context: "root IDL object".to_string(),
            suggestion: Some("Anchor IDL应该包含metadata.name字段，Shank IDL应该包含根级别的name字段".to_string()),
        });
    }
    
    // 检查instructions字段（如果存在）
    if let Some(instructions) = obj.get("instructions") {
        if !instructions.is_array() {
            return Err(SoloresError::JsonStructureError {
                field: "instructions".to_string(),
                location: "root".to_string(),
                suggestion: Some("instructions字段应该是数组类型".to_string()),
            });
        }
    }
    
    // 检查accounts字段（如果存在）
    if let Some(accounts) = obj.get("accounts") {
        if !accounts.is_array() {
            return Err(SoloresError::JsonStructureError {
                field: "accounts".to_string(),
                location: "root".to_string(),
                suggestion: Some("accounts字段应该是数组类型".to_string()),
            });
        }
    }
    
    // 检查types字段（如果存在）
    if let Some(types) = obj.get("types") {
        if !types.is_array() {
            return Err(SoloresError::JsonStructureError {
                field: "types".to_string(),
                location: "root".to_string(),
                suggestion: Some("types字段应该是数组类型".to_string()),
            });
        }
    }
    
    log::debug!("IDL基本结构验证通过");
    Ok(())
}

/// 处理文件操作的统一错误处理包装器
pub fn handle_file_operation<T>(
    operation: &str, 
    path: &std::path::Path, 
    f: impl FnOnce() -> std::io::Result<T>
) -> Result<T, SoloresError> {
    f().map_err(|e| {
        SoloresError::file_operation_error(
            operation,
            path.display().to_string(),
            e
        )
    })
}