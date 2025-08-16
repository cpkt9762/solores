//! Solores Runtime System for Parser Management and Instruction Dispatch
//! 
//! This module provides a runtime system that mimics yellowstone-vixen's core
//! dispatcher mechanism but using our own idl-core infrastructure.

use std::collections::HashMap;
use crate::{
    Parser, ProgramParser, ParseError,
    TransactionUpdate, instruction::InstructionUpdate,
    builtin_parsers::{
        SystemProgramParser, SplTokenParser, SplToken2022Parser,
        system_program::SystemProgramIx,
        token_program::TokenProgramIx,
        token_extension_program::TokenExtensionProgramIx,
        compute_budget_program::{InstructionParser as ComputeBudgetParser, ComputeBudgetProgramIx},
        memo_program::{InstructionParser as MemoParser, MemoProgramIx},
    },
};

use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;

/// Unified parsed instruction types
#[derive(Debug)]
pub enum ParsedInstruction {
    SystemProgram(SystemProgramIx),
    SplToken(TokenProgramIx),
    SplToken2022(TokenExtensionProgramIx),  // 重新启用
    ComputeBudget(ComputeBudgetProgramIx),
    Memo(MemoProgramIx),
    Custom(Box<dyn std::any::Any + Send + Sync>),
}

/// Parsed result container
#[derive(Debug)]
pub struct ParsedResult {
    pub instruction: ParsedInstruction,
    pub program_id: crate::Pubkey,
    pub parser_name: String,
}

/// Runtime configuration
#[derive(Debug, Default)]
pub struct RuntimeConfig {
    pub enable_system_program: bool,
    pub enable_spl_token: bool,
    pub enable_token_2022: bool,
    pub enable_compute_budget: bool,
    pub enable_memo_program: bool,
    pub max_instructions_per_transaction: usize,
}

impl RuntimeConfig {
    pub fn new() -> Self {
        Self {
            enable_system_program: true,
            enable_spl_token: true,
            enable_token_2022: true,
            enable_compute_budget: true,
            enable_memo_program: true,
            max_instructions_per_transaction: 1000,
        }
    }
}

/// Parser trait adapter for unified handling - following yellowstone-vixen's elegant design
trait ParsedInstructionParser: Send + Sync + std::fmt::Debug {
    fn id(&self) -> &'static str;
    fn program_id(&self) -> crate::Pubkey;
    fn try_parse<'h>(&'h self, ix: &'h InstructionUpdate) 
        -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<ParsedInstruction>> + Send + 'h>>;
}

/// System program parser adapter
#[derive(Debug)]
struct SystemProgramAdapter;
impl ParsedInstructionParser for SystemProgramAdapter {
    fn id(&self) -> &'static str { "SystemProgram" }
    fn program_id(&self) -> crate::Pubkey { 
        solana_program::system_program::ID.to_bytes().into() 
    }
    fn try_parse<'h>(&'h self, ix: &'h InstructionUpdate) 
        -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<ParsedInstruction>> + Send + 'h>> {
        Box::pin(async {
            let parser = SystemProgramParser;
            match parser.parse(ix).await {
                Ok(instruction) => Some(ParsedInstruction::SystemProgram(instruction)),
                Err(ParseError::Filtered) => None,
                Err(_) => None,
            }
        })
    }
}

/// SPL Token parser adapter
#[derive(Debug)]
struct SplTokenAdapter;
impl ParsedInstructionParser for SplTokenAdapter {
    fn id(&self) -> &'static str { "SplToken" }
    fn program_id(&self) -> crate::Pubkey { 
        spl_token::ID.to_bytes().into() 
    }
    fn try_parse<'h>(&'h self, ix: &'h InstructionUpdate) 
        -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<ParsedInstruction>> + Send + 'h>> {
        Box::pin(async {
            let parser = SplTokenParser;
            match parser.parse(ix).await {
                Ok(instruction) => Some(ParsedInstruction::SplToken(instruction)),
                Err(ParseError::Filtered) => None,
                Err(_) => None,
            }
        })
    }
}

/// Token-2022 parser adapter
#[derive(Debug)]
struct SplToken2022Adapter;
impl ParsedInstructionParser for SplToken2022Adapter {
    fn id(&self) -> &'static str { "SplToken2022" }
    fn program_id(&self) -> crate::Pubkey { 
        spl_token_2022::ID.to_bytes().into() 
    }
    fn try_parse<'h>(&'h self, ix: &'h InstructionUpdate) 
        -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<ParsedInstruction>> + Send + 'h>> {
        Box::pin(async {
            let parser = SplToken2022Parser;
            match parser.parse(ix).await {
                Ok(instruction) => Some(ParsedInstruction::SplToken2022(instruction)),
                Err(ParseError::Filtered) => None,
                Err(_) => None,
            }
        })
    }
}

/// Compute Budget parser adapter
#[derive(Debug)]
struct ComputeBudgetAdapter;
impl ParsedInstructionParser for ComputeBudgetAdapter {
    fn id(&self) -> &'static str { "ComputeBudget" }
    fn program_id(&self) -> crate::Pubkey { 
        [3, 6, 26, 217, 206, 221, 88, 97, 20, 96, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into()
    }
    fn try_parse<'h>(&'h self, ix: &'h InstructionUpdate) 
        -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<ParsedInstruction>> + Send + 'h>> {
        Box::pin(async {
            let parser = ComputeBudgetParser;
            match parser.parse(ix).await {
                Ok(instruction) => Some(ParsedInstruction::ComputeBudget(instruction)),
                Err(ParseError::Filtered) => None,
                Err(_) => None,
            }
        })
    }
}

/// Memo program parser adapter
#[derive(Debug)]
struct MemoAdapter;
impl ParsedInstructionParser for MemoAdapter {
    fn id(&self) -> &'static str { "Memo" }
    fn program_id(&self) -> crate::Pubkey { 
        [5, 78, 193, 174, 4, 69, 1, 191, 212, 127, 185, 21, 204, 91, 84, 75, 65, 251, 123, 201, 151, 
         220, 105, 138, 126, 217, 51, 212, 126, 93, 166, 165].into()
    }
    fn try_parse<'h>(&'h self, ix: &'h InstructionUpdate) 
        -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<ParsedInstruction>> + Send + 'h>> {
        Box::pin(async {
            let parser = MemoParser;
            match parser.parse(ix).await {
                Ok(instruction) => Some(ParsedInstruction::Memo(instruction)),
                Err(ParseError::Filtered) => None,
                Err(_) => None,
            }
        })
    }
}

/// Custom parser adapter for user-defined parsers
#[derive(Debug)]
struct CustomAdapter<P> {
    parser: P,
    name: String,
}

impl<P> CustomAdapter<P> {
    fn new(parser: P, name: String) -> Self {
        Self { parser, name }
    }
}

impl<P> ParsedInstructionParser for CustomAdapter<P>
where
    P: Parser<Input = InstructionUpdate> + ProgramParser + Send + Sync + std::fmt::Debug,
    P::Output: Send + Sync + 'static,
{
    fn id(&self) -> &'static str { 
        Box::leak(self.name.clone().into_boxed_str()) 
    }
    
    fn program_id(&self) -> crate::Pubkey { 
        self.parser.program_id() 
    }
    
    fn try_parse<'h>(&'h self, ix: &'h InstructionUpdate) 
        -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<ParsedInstruction>> + Send + 'h>> {
        Box::pin(async {
            match self.parser.parse(ix).await {
                Ok(instruction) => Some(ParsedInstruction::Custom(Box::new(instruction))),
                Err(ParseError::Filtered) => None,
                Err(_) => None,
            }
        })
    }
}

/// Runtime builder
pub struct RuntimeBuilder {
    config: RuntimeConfig,
    custom_parsers: Vec<Box<dyn ParsedInstructionParser>>,
}

impl RuntimeBuilder {
    pub fn new() -> Self {
        Self {
            config: RuntimeConfig::new(),
            custom_parsers: Vec::new(),
        }
    }
    
    pub fn config(mut self, config: RuntimeConfig) -> Self {
        self.config = config;
        self
    }
    
    pub fn with_system_program(mut self, enable: bool) -> Self {
        self.config.enable_system_program = enable;
        self
    }
    
    pub fn with_spl_token(mut self, enable: bool) -> Self {
        self.config.enable_spl_token = enable;
        self
    }
    
    pub fn with_token_2022(mut self, enable: bool) -> Self {
        self.config.enable_token_2022 = enable;
        self
    }
    
    pub fn with_compute_budget(mut self, enable: bool) -> Self {
        self.config.enable_compute_budget = enable;
        self
    }
    
    pub fn with_memo_program(mut self, enable: bool) -> Self {
        self.config.enable_memo_program = enable;
        self
    }
    
    /// Register custom instruction parser
    pub fn instruction<P>(mut self, parser: P, name: String) -> Self 
    where 
        P: Parser<Input = InstructionUpdate> + ProgramParser + Send + Sync + std::fmt::Debug + 'static,
        P::Output: Send + Sync + 'static,
    {
        self.custom_parsers.push(Box::new(CustomAdapter::new(parser, name)));
        self
    }
    
    pub fn build(self) -> SoloresRuntime {
        let mut parsers: Vec<Box<dyn ParsedInstructionParser>> = Vec::new();
        
        // Add builtin parsers based on config
        if self.config.enable_system_program {
            parsers.push(Box::new(SystemProgramAdapter));
        }
        if self.config.enable_spl_token {
            parsers.push(Box::new(SplTokenAdapter));
        }
        if self.config.enable_token_2022 {
            parsers.push(Box::new(SplToken2022Adapter));
        }
        if self.config.enable_compute_budget {
            parsers.push(Box::new(ComputeBudgetAdapter));
        }
        if self.config.enable_memo_program {
            parsers.push(Box::new(MemoAdapter));
        }
        
        // Add custom parsers
        parsers.extend(self.custom_parsers);
        
        SoloresRuntime {
            parsers,
            config: self.config,
        }
    }
}

impl Default for RuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Solores runtime system
pub struct SoloresRuntime {
    parsers: Vec<Box<dyn ParsedInstructionParser>>,
    config: RuntimeConfig,
}

impl SoloresRuntime {
    pub fn builder() -> RuntimeBuilder {
        RuntimeBuilder::new()
    }
    
    /// Process transaction update - core dispatch logic (mimics yellowstone-vixen)
    pub async fn process_transaction(&self, tx: &TransactionUpdate) -> Result<Vec<ParsedResult>, ParseError> {
        let mut results = Vec::new();
        
        // Parse transaction into instruction list
        let ixs = InstructionUpdate::parse_from_txn(tx)?;
        
        if ixs.len() > self.config.max_instructions_per_transaction {
            return Err(ParseError::Other(
                format!("Too many instructions: {} > {}", 
                       ixs.len(), self.config.max_instructions_per_transaction).into()
            ));
        }
        
        // Traverse all instructions (including inner instructions) - yellowstone-vixen pattern
        for insn in ixs.iter().flat_map(|i| i.visit_all()) {
            // Try all registered parsers for each instruction - core dispatch mechanism
            for parser in &self.parsers {
                if let Some(parsed_instruction) = parser.try_parse(insn).await {
                    results.push(ParsedResult {
                        instruction: parsed_instruction,
                        program_id: insn.program,
                        parser_name: parser.id().to_string(),
                    });
                    break; // Found a parser, stop trying others
                }
            }
        }
        
        Ok(results)
    }
    
    /// Process encoded transaction (our unique capability)
    pub async fn process_encoded_transaction(
        &self, 
        encoded: &EncodedConfirmedTransactionWithStatusMeta
    ) -> Result<Vec<ParsedResult>, ParseError> {
        // Use our parse_from_meta to convert first
        let instructions = InstructionUpdate::parse_from_meta(encoded, encoded.slot)?;
        
        let mut results = Vec::new();
        for instruction in &instructions {
            for parser in &self.parsers {
                if let Some(parsed) = parser.try_parse(instruction).await {
                    results.push(ParsedResult {
                        instruction: parsed,
                        program_id: instruction.program,
                        parser_name: parser.id().to_string(),
                    });
                    break;
                }
            }
        }
        Ok(results)
    }
    
    /// Get statistics about registered parsers
    pub fn get_parser_info(&self) -> HashMap<String, crate::Pubkey> {
        self.parsers.iter()
            .map(|p| (p.id().to_string(), p.program_id()))
            .collect()
    }
    
    /// Create a pre-configured DeFi runtime with all major protocols
    #[cfg(feature = "crates")]
    pub fn defi_runtime() -> Self {
        Self::builder()
            .with_system_program(true)
            .with_spl_token(true)
            .with_compute_budget(true)
            .with_memo_program(true)
            .instruction(crate::dex::RaydiumAmmInstructionParser, "RaydiumAmm".to_string())
            .instruction(crate::dex::RaydiumLaunchpadInstructionParser, "RaydiumLaunchpad".to_string())
            .instruction(crate::dex::PumpFunInstructionParser, "PumpFun".to_string())
            .instruction(crate::dex::OrcaWhirlpoolInstructionParser, "OrcaWhirlpool".to_string())
            .build()
    }
    
    /// Create a minimal runtime with only builtin parsers
    pub fn minimal_runtime() -> Self {
        Self::builder()
            .with_system_program(true)
            .with_spl_token(true)
            .with_compute_budget(false)
            .with_memo_program(false)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_runtime_builder() {
        let runtime = SoloresRuntime::builder()
            .with_system_program(true)
            .with_spl_token(true)
            .with_token_2022(false)
            .build();
            
        let parser_info = runtime.get_parser_info();
        assert!(parser_info.contains_key("SystemProgram"));
        assert!(parser_info.contains_key("SplToken"));
        assert!(!parser_info.contains_key("SplToken2022"));
    }
}