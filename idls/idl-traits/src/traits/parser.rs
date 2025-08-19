//! 核心解析器 trait 定义

use std::borrow::Cow;
use crate::types::{Prefilter, Pubkey, ParseResult, ParsedEvent};

/// 基础解析器 trait
/// 
/// 定义所有解析器的通用接口，用于处理 Yellowstone 数据更新
pub trait Parser {
    /// 输入数据类型 (如 InstructionUpdate, AccountUpdate)
    type Input;
    /// 输出解析结果类型 (如 ProgramInstruction, ProgramAccount)
    type Output;

    /// 解析器唯一标识符
    /// 
    /// 用于注册和查找解析器实例
    fn id(&self) -> Cow<str>;

    /// 预过滤器配置
    /// 
    /// 传递给 Yellowstone 用于粗粒度数据过滤
    fn prefilter(&self) -> Prefilter;

    /// 解析数据更新
    /// 
    /// 将输入数据解析为结构化输出
    fn parse(&self, value: &Self::Input) -> ParseResult<Self::Output>;
}

/// 程序级解析器 trait
/// 
/// 扩展基础 Parser，为特定 Solana 程序提供解析能力
pub trait ProgramParser: Parser {
    /// 关联的程序ID
    fn program_id(&self) -> Pubkey;
    
    /// 尝试解析程序事件数据 (新增功能)
    /// 
    /// # Arguments
    /// * `data` - Self CPI Log 事件数据 (包含 8字节 discriminator)
    /// 
    /// # Returns
    /// * `Some(ParsedEvent)` - 成功解析的事件
    /// * `None` - 不支持的事件类型或解析失败 (默认实现)
    /// 
    /// # Example
    /// ```rust
    /// let parser = PumpFunInstructionParser;
    /// if let Some(event) = parser.try_parse_any_event(&cpi_log_data) {
    ///     // 处理解析到的事件
    /// }
    /// ```
    fn try_parse_any_event(&self, _data: &[u8]) -> Option<ParsedEvent> {
        None  // 默认实现：不支持事件解析
    }
}

// 移除了冗余的辅助 trait，直接使用 Parser 和 ProgramParser trait