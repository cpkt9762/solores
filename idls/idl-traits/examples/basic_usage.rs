//! idl-traits 基本使用示例

use idl_traits::*;

// 模拟程序ID
pub const ID: Pubkey = solana_pubkey::pubkey!("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");

// 模拟生成的指令类型
#[derive(Debug, Clone)]
pub enum ProgramInstruction {
    Buy { amount: u64 },
    Sell { amount: u64 },
}

// 模拟生成的账户类型  
#[derive(Debug, Clone)]
pub enum ProgramAccount {
    Global { authority: Pubkey },
    BondingCurve { reserves: u64 },
}

// 模拟 parsers 模块
pub mod parsers {
    pub mod instructions {
        use super::super::ProgramInstruction;
        
        pub fn parse_instruction(_data: &[u8], _accounts: &[crate::Pubkey]) -> Result<ProgramInstruction, String> {
            // 模拟解析逻辑
            Ok(ProgramInstruction::Buy { amount: 1000 })
        }
    }
    
    pub mod accounts {
        use super::super::ProgramAccount;
        
        pub fn try_unpack_account(_data: &[u8]) -> Result<ProgramAccount, std::io::Error> {
            // 模拟解析逻辑
            Ok(ProgramAccount::Global { authority: crate::ID })
        }
    }
}

// 使用派生宏自动实现解析器
#[derive(InstructionParser, Debug, Clone, Copy)]
pub struct MyInstructionParser;

#[derive(AccountParser, Debug, Clone, Copy)]  
pub struct MyAccountParser;

fn main() {
    println!("🎯 idl-traits 基本使用示例");
    
    // 创建解析器实例
    let instruction_parser = MyInstructionParser;
    let account_parser = MyAccountParser;
    
    // 显示解析器信息
    println!("📋 指令解析器:");
    println!("  ID: {}", instruction_parser.id());
    println!("  程序ID: {}", instruction_parser.program_id());
    
    println!("📋 账户解析器:");
    println!("  ID: {}", account_parser.id());
    println!("  程序ID: {}", account_parser.program_id());
    
    // 测试事件解析 (默认空实现)
    let test_data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    match instruction_parser.try_parse_any_event(&test_data) {
        Some(_event) => println!("✅ 解析到事件"),
        None => println!("📭 无事件解析支持 (默认行为)"),
    }
    
    println!("🎉 示例完成！");
}