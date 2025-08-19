//! 解析结果类型定义

use crate::types::Pubkey;

/// 统一的解析事件类型
#[derive(Debug)]
pub enum ParsedEvent {
    /// 自定义事件类型 - 使用 Box<dyn Any> 支持任意事件类型
    Custom(Box<dyn std::any::Any + Send + Sync>),
}

// 手动实现 Clone
impl Clone for ParsedEvent {
    fn clone(&self) -> Self {
        match self {
            ParsedEvent::Custom(_) => {
                // 无法 clone Any 类型，创建空占位符
                ParsedEvent::Custom(Box::new(()))
            }
        }
    }
}

impl ParsedEvent {
    /// 尝试转换为特定事件类型
    pub fn try_as<T: 'static>(&self) -> Option<&T> {
        match self {
            ParsedEvent::Custom(custom) => custom.downcast_ref::<T>(),
        }
    }
    
    /// 检查是否为特定事件类型
    pub fn is_type<T: 'static>(&self) -> bool {
        matches!(self, ParsedEvent::Custom(custom) if custom.is::<T>())
    }
    
    /// 创建自定义事件
    pub fn custom<T: 'static + Send + Sync>(event: T) -> Self {
        ParsedEvent::Custom(Box::new(event))
    }
}

/// 解析结果容器
#[derive(Debug, Clone)]
pub struct ParsedResult {
    /// 解析得到的指令
    pub instruction: ParsedInstruction,
    /// 程序ID
    pub program_id: Pubkey,
    /// 解析器名称
    pub parser_name: String,
}

/// 解析指令类型 (简化版)
#[derive(Debug)]
pub enum ParsedInstruction {
    /// 自定义指令类型
    Custom(Box<dyn std::any::Any + Send + Sync>),
}

// 手动实现 Clone
impl Clone for ParsedInstruction {
    fn clone(&self) -> Self {
        match self {
            ParsedInstruction::Custom(_) => {
                // 无法 clone Any 类型，创建空占位符
                ParsedInstruction::Custom(Box::new(()))
            }
        }
    }
}

impl ParsedInstruction {
    /// 尝试转换为特定指令类型
    pub fn try_as<T: 'static>(&self) -> Option<&T> {
        match self {
            ParsedInstruction::Custom(custom) => custom.downcast_ref::<T>(),
        }
    }
    
    /// 创建自定义指令
    pub fn custom<T: 'static + Send + Sync>(instruction: T) -> Self {
        ParsedInstruction::Custom(Box::new(instruction))
    }
}