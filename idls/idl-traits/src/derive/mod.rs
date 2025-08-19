//! 派生宏实现模块

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// #[derive(InstructionParser)] 派生宏
pub fn derive_instruction_parser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    let expanded = quote! {
        impl ::idl_traits::Parser for #name {
            type Input = ::idl_traits::InstructionUpdate;
            type Output = ::idl_traits::ProgramInstruction;
            
            fn id(&self) -> ::std::borrow::Cow<str> {
                concat!(module_path!(), "::", stringify!(#name)).into()
            }
            
            fn prefilter(&self) -> ::idl_traits::Prefilter {
                ::idl_traits::Prefilter::builder()
                    .transaction_accounts([crate::ID])
                    .build()
                    .unwrap()
            }
            
            fn parse(&self, ix_update: &::idl_traits::InstructionUpdate) -> ::idl_traits::ParseResult<Self::Output> {
                if ix_update.program == crate::ID {
                    // 使用生成的解析函数
                    crate::parsers::instructions::parse_instruction(&ix_update.data, &ix_update.accounts)
                        .map_err(|e| ::idl_traits::ParseError::InvalidInstructionData(e.to_string()))
                        .map(Into::into)
                } else {
                    ::error_stack::bail!(::idl_traits::ParseError::Filtered)
                }
            }
        }
        
        impl ::idl_traits::ProgramParser for #name {
            fn program_id(&self) -> ::idl_traits::Pubkey {
                crate::ID
            }
            
            // 默认空实现 - 可以被手动覆盖
            fn try_parse_any_event(&self, _data: &[u8]) -> ::std::option::Option<::idl_traits::ParsedEvent> {
                ::std::option::Option::None
            }
        }
    };
    
    TokenStream::from(expanded)
}

/// #[derive(AccountParser)] 派生宏
pub fn derive_account_parser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    let expanded = quote! {
        impl ::idl_traits::Parser for #name {
            type Input = ::idl_traits::AccountUpdate;
            type Output = ::idl_traits::ProgramAccount;
            
            fn id(&self) -> ::std::borrow::Cow<str> {
                concat!(module_path!(), "::", stringify!(#name)).into()
            }
            
            fn prefilter(&self) -> ::idl_traits::Prefilter {
                ::idl_traits::Prefilter::builder()
                    .account_owners([crate::ID])
                    .build()
                    .unwrap()
            }
            
            fn parse(&self, account_update: &::idl_traits::AccountUpdate) -> ::idl_traits::ParseResult<Self::Output> {
                if account_update.owner == crate::ID {
                    // 使用生成的账户解析函数
                    crate::parsers::accounts::try_unpack_account(&account_update.data)
                        .map_err(|e| ::idl_traits::ParseError::DeserializationFailed(e.to_string()))
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