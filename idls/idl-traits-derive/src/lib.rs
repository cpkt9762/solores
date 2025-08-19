//! IDL Traits 派生宏库
//!
//! 提供自动实现 Parser 和 ProgramParser trait 的派生宏

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// #[derive(InstructionParser)] 派生宏
/// 
/// 自动为结构体实现 Parser + ProgramParser trait，用于解析指令数据
/// 
/// # 要求
/// - 结构体所在的 crate 必须有 `ID` 常量 (程序ID)
/// - 结构体所在的 crate 必须有 `parsers::instructions` 模块
/// 
/// # 示例
/// ```rust
/// #[derive(InstructionParser)]
/// pub struct MyInstructionParser;
/// ```
#[proc_macro_derive(InstructionParser)]
pub fn derive_instruction_parser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    let expanded = quote! {
        impl ::idl_traits::Parser for #name {
            type Input = ::idl_traits::InstructionUpdate;
            type Output = ::idl_traits::ParsedInstruction;
            
            fn id(&self) -> ::std::borrow::Cow<str> {
                concat!(module_path!(), "::", stringify!(#name)).into()
            }
            
            fn prefilter(&self) -> ::idl_traits::Prefilter {
                ::idl_traits::Prefilter::builder()
                    .transaction_accounts([crate::ID])
                    .build()
                    .unwrap()
            }
            
            fn parse(&self, ix_update: &Self::Input) -> ::idl_traits::ParseResult<Self::Output> {
                if ix_update.program == crate::ID {
                    // 使用生成的解析函数
                    let parsed = crate::parsers::instructions::parse_instruction(&ix_update.data, &ix_update.accounts)
                        .map_err(|e| ::idl_traits::ParseError::InvalidInstructionData(e.to_string()))?;
                    ::std::result::Result::Ok(::idl_traits::ParsedInstruction::custom(parsed))
                } else {
                    ::error_stack::bail!(::idl_traits::ParseError::Filtered)
                }
            }
        }
        
        impl ::idl_traits::ProgramParser for #name {
            fn program_id(&self) -> ::idl_traits::Pubkey {
                crate::ID
            }
            
            // 默认空实现 - 可以通过手动实现覆盖
            fn try_parse_any_event(&self, _data: &[u8]) -> ::std::option::Option<::idl_traits::ParsedEvent> {
                ::std::option::Option::None
            }
        }
    };
    
    TokenStream::from(expanded)
}

/// #[derive(AccountParser)] 派生宏
/// 
/// 自动为结构体实现 Parser + ProgramParser trait，用于解析账户数据
/// 
/// # 要求
/// - 结构体所在的 crate 必须有 `ID` 常量 (程序ID)
/// - 结构体所在的 crate 必须有 `parsers::accounts` 模块
/// 
/// # 示例
/// ```rust
/// #[derive(AccountParser)]
/// pub struct MyAccountParser;
/// ```
#[proc_macro_derive(AccountParser)]
pub fn derive_account_parser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    let expanded = quote! {
        impl ::idl_traits::Parser for #name {
            type Input = ::idl_traits::AccountUpdate;
            type Output = ::idl_traits::ParsedInstruction;
            
            fn id(&self) -> ::std::borrow::Cow<str> {
                concat!(module_path!(), "::", stringify!(#name)).into()
            }
            
            fn prefilter(&self) -> ::idl_traits::Prefilter {
                ::idl_traits::Prefilter::builder()
                    .account_owners([crate::ID])
                    .build()
                    .unwrap()
            }
            
            fn parse(&self, account_update: &Self::Input) -> ::idl_traits::ParseResult<Self::Output> {
                if account_update.owner == crate::ID {
                    // 使用生成的账户解析函数
                    let parsed = crate::parsers::accounts::try_unpack_account(&account_update.data)
                        .map_err(|e| ::idl_traits::ParseError::DeserializationFailed(e.to_string()))?;
                    ::std::result::Result::Ok(::idl_traits::ParsedInstruction::custom(parsed))
                } else {
                    ::error_stack::bail!(::idl_traits::ParseError::Filtered)
                }
            }
        }
        
        impl ::idl_traits::ProgramParser for #name {
            fn program_id(&self) -> ::idl_traits::Pubkey {
                crate::ID
            }
            
            // 默认空实现
            fn try_parse_any_event(&self, _data: &[u8]) -> ::std::option::Option<::idl_traits::ParsedEvent> {
                ::std::option::Option::None
            }
        }
    };
    
    TokenStream::from(expanded)
}