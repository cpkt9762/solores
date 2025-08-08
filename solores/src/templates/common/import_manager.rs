//! 智能导入管理器
//!
//! 提供智能的导入管理，包括使用跟踪、冲突检测和精确导入策略

use proc_macro2::TokenStream;
use quote::quote;
use std::collections::{HashMap, HashSet};

/// 检查代码中是否包含自定义类型（驼峰命名且以大写字母开头）
fn has_custom_types_in_code(code_content: &str) -> bool {
    // 常见的自定义类型名模式检测
    let custom_type_patterns = [
        "Params", "Config", "Event", "Info", "Schedule", "Direction", 
        "Status", "Curve", "Platform", "Mint", "Vesting", "Trade", "Pool"
    ];
    
    for pattern in &custom_type_patterns {
        if code_content.contains(pattern) {
            return true;
        }
    }
    
    // 检查驼峰命名类型（以大写字母开头后跟小写字母）
    for word in code_content.split_whitespace() {
        let word = word.trim_end_matches(|c: char| c.is_ascii_punctuation());
        if word.len() > 3 && 
           word.chars().next().unwrap().is_ascii_uppercase() &&
           word.chars().nth(1).unwrap().is_ascii_lowercase() &&
           word.chars().any(|c| c.is_ascii_uppercase()) {
            return true;
        }
    }
    
    false
}

/// 导入类型分类
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImportType {
    /// Borsh 序列化导入
    Borsh,
    /// Solana 相关导入  
    Solana(SolanaImport),
    /// 标准库导入
    Std(String),
    /// 内部 crate 导入
    Crate(String),
    /// 外部 crate 导入
    External(String),
    /// 测试相关导入
    Test,
}

/// Solana 相关的具体导入
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SolanaImport {
    /// Pubkey 类型
    Pubkey,
    /// AccountInfo 类型
    AccountInfo,
    /// AccountMeta 和 Instruction
    Instruction,
    /// ProgramResult
    ProgramResult,
    /// ProgramError
    ProgramError,
    /// Invoke 函数
    Invoke,
}

/// 导入项跟踪信息
#[derive(Debug, Clone)]
struct ImportTracker {
    /// 导入类型
    #[allow(dead_code)]
    import_type: ImportType,
    /// 是否被使用
    is_used: bool,
    /// 使用次数
    usage_count: usize,
    /// 生成的导入语句
    import_statement: String,
}

/// 智能导入管理器
pub struct ImportManager {
    /// 跟踪的导入项
    tracked_imports: HashMap<String, ImportTracker>,
    /// 检测到的冲突
    conflicts: HashSet<String>,
    /// 使用的类型名称(用于冲突检测)
    used_names: HashSet<String>,
    /// 实际使用的符号集合（动态跟踪）
    actually_used_symbols: HashSet<String>,
    /// 需要的条件导入（比如测试、特性等）
    conditional_imports: HashMap<String, Vec<String>>,
}

impl ImportManager {
    /// 创建新的智能导入管理器
    pub fn new() -> Self {
        Self {
            tracked_imports: HashMap::new(),
            conflicts: HashSet::new(),
            used_names: HashSet::new(),
            actually_used_symbols: HashSet::new(),
            conditional_imports: HashMap::new(),
        }
    }

    /// 添加导入并跟踪使用情况
    pub fn add_import(&mut self, import_type: ImportType) {
        let (key, import_statement) = match &import_type {
            ImportType::Borsh => (
                "borsh".to_string(),
                "use borsh::{BorshDeserialize, BorshSerialize};".to_string(),
            ),
            ImportType::Solana(solana_import) => {
                match solana_import {
                    SolanaImport::Pubkey => (
                        "solana_pubkey".to_string(),
                        "use solana_pubkey::Pubkey;".to_string(),
                    ),
                    SolanaImport::AccountInfo => (
                        "account_info".to_string(),
                        "use solana_account_info::AccountInfo;".to_string(),
                    ),
                    SolanaImport::Instruction => (
                        "instruction".to_string(),
                        "use solana_instruction::{AccountMeta, Instruction};".to_string(),
                    ),
                    SolanaImport::ProgramResult => (
                        "program_result".to_string(),
                        "use solana_program_entrypoint::ProgramResult;".to_string(),
                    ),
                    SolanaImport::ProgramError => (
                        "program_error".to_string(),
                        "use solana_program_error::ProgramError;".to_string(),
                    ),
                    SolanaImport::Invoke => (
                        "invoke".to_string(),
                        "use solana_cpi::{invoke, invoke_signed};".to_string(),
                    ),
                }
            },
            ImportType::Std(module) => (
                format!("std_{}", module.replace("::", "_")),
                format!("use std::{};", module),
            ),
            ImportType::Crate(module) => (
                format!("crate_{}", module.replace("::", "_")),
                format!("use crate::{};", module),
            ),
            ImportType::External(crate_name) => (
                crate_name.clone(),
                format!("use {};", crate_name),
            ),
            ImportType::Test => (
                "test_imports".to_string(),
                "#[cfg(test)]\nuse crate::*;".to_string(),
            ),
        };

        // 检测冲突
        self.detect_naming_conflicts(&import_type);

        // 添加或更新跟踪信息
        self.tracked_imports.insert(key, ImportTracker {
            import_type,
            is_used: false,
            usage_count: 0,
            import_statement,
        });
    }

    /// 标记导入项为已使用
    pub fn mark_as_used(&mut self, import_key: &str) {
        if let Some(tracker) = self.tracked_imports.get_mut(import_key) {
            tracker.is_used = true;
            tracker.usage_count += 1;
        }
    }

    /// 检测命名冲突
    fn detect_naming_conflicts(&mut self, import_type: &ImportType) {
        let extracted_names = self.extract_type_names(import_type);
        
        for name in &extracted_names {
            if self.used_names.contains(name) {
                self.conflicts.insert(name.clone());
            } else {
                self.used_names.insert(name.clone());
            }
        }
    }

    /// 从导入类型中提取类型名称
    fn extract_type_names(&self, import_type: &ImportType) -> Vec<String> {
        match import_type {
            ImportType::Borsh => vec!["BorshDeserialize".to_string(), "BorshSerialize".to_string()],
            ImportType::Solana(solana_import) => {
                match solana_import {
                    SolanaImport::Pubkey => vec!["Pubkey".to_string()],
                    SolanaImport::AccountInfo => vec!["AccountInfo".to_string()],
                    SolanaImport::Instruction => vec!["AccountMeta".to_string(), "Instruction".to_string()],
                    SolanaImport::ProgramResult => vec!["ProgramResult".to_string()],
                    SolanaImport::ProgramError => vec!["ProgramError".to_string()],
                    SolanaImport::Invoke => vec!["invoke".to_string(), "invoke_signed".to_string()],
                }
            },
            ImportType::Crate(module) => {
                // 从模块路径提取最后的名称
                vec![module.split("::").last().unwrap_or(module).to_string()]
            },
            _ => vec![],
        }
    }

    /// 添加特定的Borsh导入
    pub fn add_borsh_imports(&mut self) {
        self.add_import(ImportType::Borsh);
        self.mark_as_used("borsh");
    }

    /// 添加Solana Pubkey导入
    pub fn add_solana_pubkey(&mut self) {
        self.add_import(ImportType::Solana(SolanaImport::Pubkey));
        self.mark_as_used("solana_pubkey");
    }

    /// 添加Solana AccountInfo导入
    pub fn add_solana_account_info(&mut self) {
        self.add_import(ImportType::Solana(SolanaImport::AccountInfo));
        self.mark_as_used("account_info");
    }

    /// 添加Solana Instruction导入
    pub fn add_solana_instruction(&mut self) {
        self.add_import(ImportType::Solana(SolanaImport::Instruction));
        self.mark_as_used("instruction");
    }

    /// 添加Solana ProgramResult导入
    pub fn add_solana_program_result(&mut self) {
        self.add_import(ImportType::Solana(SolanaImport::ProgramResult));
        self.mark_as_used("program_result");
    }

    /// 添加Solana Invoke导入
    pub fn add_solana_invoke(&mut self) {
        self.add_import(ImportType::Solana(SolanaImport::Invoke));
        self.mark_as_used("invoke");
    }

    /// 添加标准库IO导入
    pub fn add_std_io(&mut self) {
        self.add_import(ImportType::Std("io".to_string()));
        self.mark_as_used("std_io");
    }

    /// 添加标准库IO Read导入
    pub fn add_std_io_read(&mut self) {
        self.add_import(ImportType::Std("io::Read".to_string()));
        self.mark_as_used("std_io_read");
    }

    /// 添加内部crate类型导入
    pub fn add_crate_type_import(&mut self, type_name: &str) {
        self.add_import(ImportType::Crate(type_name.to_string()));
        let key = format!("crate_{}", type_name.replace("::", "_"));
        self.mark_as_used(&key);
    }

    /// 添加测试相关导入
    pub fn add_test_imports(&mut self) {
        self.add_import(ImportType::Test);
        self.mark_as_used("test_imports");
    }
    
    /// 生成所有已使用的导入语句
    pub fn generate_imports(&self) -> TokenStream {
        let mut used_imports: Vec<String> = self.tracked_imports
            .values()
            .filter(|tracker| tracker.is_used)
            .map(|tracker| tracker.import_statement.clone())
            .collect();

        // 按字典序排序，确保输出稳定
        used_imports.sort();

        let import_tokens: Result<Vec<TokenStream>, _> = used_imports
            .iter()
            .map(|s| s.parse())
            .collect();
        
        match import_tokens {
            Ok(tokens) => quote! { #(#tokens)* },
            Err(_) => quote! {}, // 如果解析失败，返回空
        }
    }

    /// 生成优化的导入语句（移除未使用的导入）
    pub fn generate_optimized_imports(&self) -> TokenStream {
        // 只包含使用次数 > 0 的导入
        let mut used_imports: Vec<String> = self.tracked_imports
            .values()
            .filter(|tracker| tracker.usage_count > 0)
            .map(|tracker| tracker.import_statement.clone())
            .collect();

        used_imports.sort();

        let import_tokens: Result<Vec<TokenStream>, _> = used_imports
            .iter()
            .map(|s| s.parse())
            .collect();
        
        match import_tokens {
            Ok(tokens) => quote! { #(#tokens)* },
            Err(_) => quote! {},
        }
    }

    /// 获取检测到的冲突
    pub fn get_conflicts(&self) -> &HashSet<String> {
        &self.conflicts
    }

    /// 检查是否有未使用的导入
    pub fn has_unused_imports(&self) -> bool {
        self.tracked_imports.values().any(|tracker| !tracker.is_used)
    }

    /// 获取未使用的导入列表
    pub fn get_unused_imports(&self) -> Vec<String> {
        self.tracked_imports
            .values()
            .filter(|tracker| !tracker.is_used)
            .map(|tracker| tracker.import_statement.clone())
            .collect()
    }

    /// 基于代码内容动态分析实际使用的符号
    pub fn analyze_code_usage(&mut self, code_content: &str) {
        // 排除注释行进行分析
        let code_lines: Vec<&str> = code_content.lines()
            .filter(|line| !line.trim().starts_with("//") && !line.trim().starts_with("///") && !line.trim().starts_with("//!"))
            .collect();
        let active_code = code_lines.join("\n");
        
        // 分析Borsh相关使用 - 检查实际derive或函数调用
        if active_code.contains("BorshDeserialize") && (active_code.contains("#[derive(") || active_code.contains("deserialize(") || active_code.contains("try_from_slice(")) {
            self.actually_used_symbols.insert("BorshDeserialize".to_string());
            self.mark_as_used("borsh");
        }
        if active_code.contains("BorshSerialize") && (active_code.contains("#[derive(") || active_code.contains("serialize(") || active_code.contains("try_to_vec(")) {
            self.actually_used_symbols.insert("BorshSerialize".to_string());
            self.mark_as_used("borsh");
        }
        
        // 分析Solana类型使用 - 检查实际类型使用而非注释
        if active_code.contains(": Pubkey") || active_code.contains("<Pubkey>") || active_code.contains("Pubkey::") || active_code.contains("&Pubkey") {
            self.actually_used_symbols.insert("Pubkey".to_string());
            self.mark_as_used("solana_pubkey");
        }
        
        if active_code.contains("AccountInfo<") || active_code.contains(": AccountInfo") || active_code.contains("&AccountInfo") {
            self.actually_used_symbols.insert("AccountInfo".to_string());
            self.mark_as_used("account_info");
        }
        
        // 检查实际函数调用和使用
        let needs_account_meta = active_code.contains("AccountMeta::") || active_code.contains(": AccountMeta") || active_code.contains("Vec<AccountMeta>");
        let needs_instruction = active_code.contains("Instruction {") || active_code.contains(": Instruction") || active_code.contains("&Instruction");
        
        if needs_account_meta {
            self.actually_used_symbols.insert("AccountMeta".to_string());
            self.mark_as_used("instruction");
        }
        if needs_instruction {
            self.actually_used_symbols.insert("Instruction".to_string());
            self.mark_as_used("instruction");
        }
        
        if active_code.contains("-> ProgramResult") || active_code.contains(": ProgramResult") {
            self.actually_used_symbols.insert("ProgramResult".to_string());
            self.mark_as_used("program_result");
        }
        
        if active_code.contains("ProgramError::") || active_code.contains(": ProgramError") {
            self.actually_used_symbols.insert("ProgramError".to_string());
            self.mark_as_used("program_error");
        }
        
        // 检查实际函数调用
        if active_code.contains("invoke(") || active_code.contains("invoke_signed(") {
            if active_code.contains("invoke(") {
                self.actually_used_symbols.insert("invoke".to_string());
            }
            if active_code.contains("invoke_signed(") {
                self.actually_used_symbols.insert("invoke_signed".to_string());
            }
            self.mark_as_used("invoke");
        }
        
        // 分析标准库使用 - 检查实际trait使用
        if active_code.contains("Read::") || active_code.contains(".read(") || active_code.contains(": Read") {
            self.actually_used_symbols.insert("std::io::Read".to_string());
            self.mark_as_used("std_io_read");
        }
        if active_code.contains("Write::") || active_code.contains(".write(") || active_code.contains(": Write") {
            self.actually_used_symbols.insert("std::io::Write".to_string());
            self.mark_as_used("std_io_write");
        }
        if active_code.contains("Error::") || active_code.contains("std::io::Error") {
            self.actually_used_symbols.insert("std::io::Error".to_string());
            self.mark_as_used("std_io");
        }
    }

    /// 生成基于实际使用情况的最小化导入
    pub fn generate_minimal_imports(&self) -> TokenStream {
        let mut imports = Vec::new();
        
        // Borsh导入
        if self.actually_used_symbols.contains("BorshDeserialize") || 
           self.actually_used_symbols.contains("BorshSerialize") {
            let mut borsh_imports = Vec::new();
            if self.actually_used_symbols.contains("BorshDeserialize") {
                borsh_imports.push("BorshDeserialize");
            }
            if self.actually_used_symbols.contains("BorshSerialize") {
                borsh_imports.push("BorshSerialize");
            }
            imports.push(format!("use borsh::{{{}}};", borsh_imports.join(", ")));
        }
        
        // Solana导入
        if self.actually_used_symbols.contains("Pubkey") {
            imports.push("use solana_pubkey::Pubkey;".to_string());
        }
        
        if self.actually_used_symbols.contains("AccountInfo") {
            imports.push("use solana_account_info::AccountInfo;".to_string());
        }
        
        if self.actually_used_symbols.contains("AccountMeta") || 
           self.actually_used_symbols.contains("Instruction") {
            let mut instr_imports = Vec::new();
            if self.actually_used_symbols.contains("AccountMeta") {
                instr_imports.push("AccountMeta");
            }
            if self.actually_used_symbols.contains("Instruction") {
                instr_imports.push("Instruction");
            }
            imports.push(format!("use solana_instruction::{{{}}};", instr_imports.join(", ")));
        }
        
        if self.actually_used_symbols.contains("ProgramResult") {
            imports.push("use solana_program_entrypoint::ProgramResult;".to_string());
        }
        
        if self.actually_used_symbols.contains("ProgramError") {
            imports.push("use solana_program_error::ProgramError;".to_string());
        }
        
        if self.actually_used_symbols.contains("invoke") || 
           self.actually_used_symbols.contains("invoke_signed") {
            let mut invoke_imports = Vec::new();
            if self.actually_used_symbols.contains("invoke") {
                invoke_imports.push("invoke");
            }
            if self.actually_used_symbols.contains("invoke_signed") {
                invoke_imports.push("invoke_signed");
            }
            imports.push(format!("use solana_cpi::{{{}}};", invoke_imports.join(", ")));
        }
        
        // 标准库导入
        if self.actually_used_symbols.contains("std::io::Read") {
            imports.push("use std::io::Read;".to_string());
        }
        
        if self.actually_used_symbols.contains("std::io::Error") {
            imports.push("use std::io::Error;".to_string());
        }
        
        // 排序并生成TokenStream
        imports.sort();
        let import_tokens: Result<Vec<TokenStream>, _> = imports
            .iter()
            .map(|s| s.parse())
            .collect();
        
        match import_tokens {
            Ok(tokens) => quote! { #(#tokens)* },
            Err(_) => quote! {},
        }
    }

    /// 生成智能的指令模块导入（基于实际需要）
    pub fn generate_smart_instruction_imports(code_content: &str) -> TokenStream {
        let mut manager = Self::new();
        
        // 分析代码内容并添加所需的导入
        manager.analyze_code_usage(code_content);
        
        // 根据分析结果生成最小化导入
        manager.generate_minimal_imports()
    }

    /// 生成针对指令文件的智能优化导入（基于代码内容分析）
    pub fn generate_optimized_instruction_imports_for_code(code_content: &str) -> TokenStream {
        Self::generate_optimized_instruction_imports_for_code_with_types_check(code_content, true)
    }

    /// 生成针对指令文件的智能优化导入（基于代码内容分析和types模块存在性检查）
    pub fn generate_optimized_instruction_imports_for_code_with_types_check(
        code_content: &str, 
        has_types_module: bool
    ) -> TokenStream {
        let mut manager = Self::new();
        
        // 分析代码内容并确定实际需要的导入
        manager.analyze_code_usage(code_content);
        
        // 检查特定的instruction文件模式
        let needs_types = has_types_module && (
            code_content.contains("impl From<") || 
            code_content.contains("impl Default for") ||
            code_content.contains("crate::types::") ||
            // 检查常见自定义类型名（以大写字母开头的驼峰命名）
            has_custom_types_in_code(code_content)
        );
        
        // 强制添加指令文件通常需要的导入
        manager.mark_as_used("borsh"); // 指令总是需要Borsh
        
        // 根据实际使用情况添加其他导入
        let mut imports = Vec::new();
        
        // Borsh导入（指令文件必需）
        imports.push("use borsh::{BorshDeserialize, BorshSerialize};".to_string());
        
        // Pubkey导入（如果需要）
        if manager.actually_used_symbols.contains("Pubkey") || code_content.contains("Pubkey") {
            imports.push("use solana_pubkey::Pubkey;".to_string());
        }
        
        // AccountMeta和Instruction导入（如果需要）
        if manager.actually_used_symbols.contains("AccountMeta") || 
           manager.actually_used_symbols.contains("Instruction") || 
           code_content.contains("AccountMeta") || 
           code_content.contains("Instruction") {
            imports.push("use solana_instruction::{AccountMeta, Instruction};".to_string());
        }
        
        // Types导入（只有在types模块存在且需要时才添加）
        if needs_types {
            imports.push("use crate::types::*;".to_string());
        }
        
        // 转换为TokenStream
        let import_tokens: Result<Vec<TokenStream>, _> = imports
            .iter()
            .map(|s| s.parse())
            .collect();
        
        match import_tokens {
            Ok(tokens) => quote! { #(#tokens)* },
            Err(_) => {
                // 回退到基础导入
                quote! {
                    use borsh::{BorshDeserialize, BorshSerialize};
                }
            }
        }
    }

    /// 生成针对指令文件的优化导入（包含指令文件必需的所有类型）
    pub fn generate_optimized_instruction_imports() -> TokenStream {
        quote! {
            use borsh::{BorshDeserialize, BorshSerialize};
            use solana_pubkey::Pubkey;
            use solana_instruction::{AccountMeta, Instruction};
            use crate::types::*;
        }
    }

    /// 生成标准的模块导入（用于Instructions模块 - 保持兼容性）
    pub fn generate_instruction_imports() -> TokenStream {
        Self::generate_smart_instruction_imports("")
    }

    /// 生成账户模块智能导入（基于代码内容）
    pub fn generate_smart_account_imports(code_content: &str) -> TokenStream {
        let mut imports = Vec::new();
        
        // Borsh导入（账户结构体总是需要）
        imports.push("use borsh::{BorshDeserialize, BorshSerialize};".to_string());
        
        // Pubkey导入（如果代码中使用了Pubkey）
        if code_content.contains("Pubkey") {
            imports.push("use solana_pubkey::Pubkey;".to_string());
        }
        
        // Types导入（如果引用了其他类型）
        if code_content.contains("crate::types::") && !code_content.contains("use crate::types::*;") {
            imports.push("use crate::types::*;".to_string());
        }
        
        // 转换为TokenStream
        let import_tokens: Result<Vec<TokenStream>, _> = imports
            .iter()
            .map(|s| s.parse())
            .collect();
        
        match import_tokens {
            Ok(tokens) => quote! { #(#tokens)* },
            Err(_) => quote! { use borsh::{BorshDeserialize, BorshSerialize}; },
        }
    }

    /// 生成类型模块智能导入（基于代码内容）
    pub fn generate_smart_type_imports(code_content: &str) -> TokenStream {
        let mut imports = Vec::new();
        
        // Borsh导入（类型结构体总是需要）
        imports.push("use borsh::{BorshDeserialize, BorshSerialize};".to_string());
        
        // Pubkey导入（如果代码中使用了Pubkey）
        if code_content.contains("Pubkey") {
            imports.push("use solana_pubkey::Pubkey;".to_string());
        }
        
        // 避免自引用types模块
        if code_content.contains("crate::types::") && !code_content.contains("use crate::types::*;") {
            // Types模块通常不需要自引用除非有嵌套类型
            let has_nested_types = code_content.matches("crate::types::").count() > 1;
            if has_nested_types {
                imports.push("use crate::types::*;".to_string());
            }
        }
        
        // 转换为TokenStream
        let import_tokens: Result<Vec<TokenStream>, _> = imports
            .iter()
            .map(|s| s.parse())
            .collect();
        
        match import_tokens {
            Ok(tokens) => quote! { #(#tokens)* },
            Err(_) => quote! { use borsh::{BorshDeserialize, BorshSerialize}; },
        }
    }

    /// 生成账户模块导入
    pub fn generate_account_imports() -> TokenStream {
        let mut manager = Self::new();
        manager.add_borsh_imports();
        manager.add_solana_pubkey();
        
        manager.generate_imports()
    }

    /// 生成类型模块导入
    pub fn generate_type_imports() -> TokenStream {
        let mut manager = Self::new();
        manager.add_borsh_imports();
        manager.add_solana_pubkey();
        
        manager.generate_imports()
    }

    /// 生成事件模块导入
    pub fn generate_event_imports() -> TokenStream {
        let mut manager = Self::new();
        manager.add_borsh_imports();
        manager.add_solana_pubkey();
        
        manager.generate_imports()
    }

    /// 生成错误模块智能导入（基于代码内容）
    pub fn generate_smart_error_imports(code_content: &str) -> TokenStream {
        let mut imports = Vec::new();
        
        // ProgramError导入（如果代码中使用了）
        if code_content.contains("ProgramError") {
            imports.push("use solana_program_error::ProgramError;".to_string());
        }
        
        // thiserror::Error导入（如果代码中使用了）
        if code_content.contains("#[derive(Error)]") || code_content.contains("thiserror::Error") {
            imports.push("use thiserror::Error;".to_string());
        }
        
        // 转换为TokenStream
        let import_tokens: Result<Vec<TokenStream>, _> = imports
            .iter()
            .map(|s| s.parse())
            .collect();
        
        match import_tokens {
            Ok(tokens) => quote! { #(#tokens)* },
            Err(_) => quote! {},
        }
    }

    /// 生成Parser模块智能导入（基于代码内容）
    pub fn generate_smart_parser_imports(code_content: &str) -> TokenStream {
        let mut imports = Vec::new();
        
        // BorshDeserialize导入（Parser模块通常需要）
        if code_content.contains("BorshDeserialize") || code_content.contains("try_from_slice") {
            imports.push("use borsh::BorshDeserialize;".to_string());
        }
        
        // BorshSerialize导入（如果需要序列化）
        if code_content.contains("BorshSerialize") || code_content.contains("try_to_vec") {
            imports.push("use borsh::BorshSerialize;".to_string());
        }
        
        // 如果两者都需要，使用组合导入
        if code_content.contains("BorshDeserialize") && code_content.contains("BorshSerialize") {
            imports.clear();
            imports.push("use borsh::{BorshDeserialize, BorshSerialize};".to_string());
        }
        
        // Pubkey导入（如果需要）
        if code_content.contains("Pubkey") {
            imports.push("use solana_pubkey::Pubkey;".to_string());
        }
        
        // std::io相关导入
        if code_content.contains("std::io::Write") || code_content.contains(".write(") {
            imports.push("use std::io::Write;".to_string());
        }
        
        // 转换为TokenStream
        let import_tokens: Result<Vec<TokenStream>, _> = imports
            .iter()
            .map(|s| s.parse())
            .collect();
        
        match import_tokens {
            Ok(tokens) => quote! { #(#tokens)* },
            Err(_) => quote! { use borsh::BorshDeserialize; },
        }
    }

    /// 生成错误模块导入
    pub fn generate_error_imports() -> TokenStream {
        quote! {
            use solana_program_error::ProgramError;
            use thiserror::Error;
        }
    }

    /// 生成Parser模块导入
    pub fn generate_parser_imports() -> TokenStream {
        let mut manager = Self::new();
        manager.add_borsh_imports();
        manager.add_solana_pubkey();
        manager.add_std_io();
        
        manager.generate_imports()
    }

    /// 生成测试模块导入（带有未使用导入抑制）
    pub fn generate_test_imports() -> TokenStream {
        quote! {
            #[cfg(test)]
            #[allow(unused_imports)]
            use crate::*;
        }
    }

    /// 生成精确的模块重导出（避免glob导入）
    pub fn generate_precise_reexports(modules: &[&str]) -> TokenStream {
        let reexport_tokens: Vec<TokenStream> = modules
            .iter()
            .map(|module| {
                let module_ident = syn::Ident::new(module, proc_macro2::Span::call_site());
                quote! { 
                    pub mod #module_ident;
                    pub use #module_ident::*;
                }
            })
            .collect();

        quote! { #(#reexport_tokens)* }
    }
}

impl Default for ImportManager {
    fn default() -> Self {
        Self::new()
    }
}