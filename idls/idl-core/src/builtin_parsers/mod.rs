pub mod helpers;
pub mod system_program;
pub mod token_program;
pub mod token_extension_program;  // 重新启用
pub mod compute_budget_program;
pub mod memo_program;

// Re-export common parsers
pub use system_program::InstructionParser as SystemProgramParser;
pub use token_program::InstructionParser as SplTokenParser;
pub use token_extension_program::InstructionParser as SplToken2022Parser;  // 重新启用
pub use compute_budget_program::InstructionParser as ComputeBudgetParser;
pub use memo_program::InstructionParser as MemoParser;