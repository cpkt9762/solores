//! 解析错误类型定义

/// 解析结果类型
pub type ParseResult<T> = error_stack::Result<T, ParseError>;

/// 解析错误枚举
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("解析被过滤器排除")]
    Filtered,
    
    #[error("无效的指令数据: {0}")]
    InvalidInstructionData(String),
    
    #[error("缺少必需数据: {field}")]
    MissingData { field: String },
    
    #[error("discriminator 不匹配: 期望 {expected:?}, 实际 {found:?}")]
    DiscriminatorMismatch { 
        expected: [u8; 8], 
        found: [u8; 8] 
    },
    
    #[error("数据长度不足: 期望至少 {expected} 字节, 实际 {found} 字节")]
    DataTooShort { 
        expected: usize, 
        found: usize 
    },
    
    #[error("反序列化失败: {0}")]
    DeserializationFailed(String),
    
    #[error("其他错误: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}