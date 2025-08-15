//! 非 Anchor Parsers 模板
//!
//! 为非 Anchor 合约生成 Parsers 相关代码，使用 1 字节 discriminator 和基于长度的识别

use proc_macro2::TokenStream;
use quote::quote;
use convert_case::{Case, Casing};
use heck::ToShoutySnakeCase;

use crate::idl_format::non_anchor_idl::NonAnchorIdl;
use crate::idl_format::IdlFormat;
use crate::Args;
use crate::templates::{TemplateGenerator, ParsersTemplateGenerator};
use crate::templates::non_anchor::instructions_parser_test_template::NonAnchorInstructionsParserTestTemplate;
use crate::templates::non_anchor::accounts_parser_test_template::NonAnchorAccountsParserTestTemplate;
use crate::templates::{NonAnchorInstructionsParserTestGenerator, NonAnchorAccountsParserTestGenerator};

/// 非 Anchor Parsers 模板
pub struct NonAnchorParsersTemplate<'a> {
    pub idl: &'a NonAnchorIdl,
    pub args: &'a Args,
}

impl<'a> NonAnchorParsersTemplate<'a> {
    /// 创建新的非 Anchor Parsers 模板
    pub fn new(idl: &'a NonAnchorIdl, args: &'a Args) -> Self {
        Self { idl, args }
    }

    // 生成非Anchor序列化match arms（1字节discriminator）
}

impl<'a> ParsersTemplateGenerator for NonAnchorParsersTemplate<'a> {
    fn generate_instructions_parser(&self) -> TokenStream {
        let instructions = self.idl.instructions();
        if instructions.is_empty() {
            return quote! {};
        }


        // Generate instruction enum variants
        let enum_variants = instructions.iter().map(|ix| {
            let variant_name = syn::Ident::new(&ix.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let keys_struct_name = syn::Ident::new(
                &format!("{}Keys", ix.name.to_case(Case::Pascal)),
                proc_macro2::Span::call_site(),
            );
            let args_struct_name = syn::Ident::new(
                &format!("{}IxData", ix.name.to_case(Case::Pascal)),
                proc_macro2::Span::call_site(),
            );

            // 统一所有指令都包含 Keys 和 Args
            quote! {
                #variant_name(#keys_struct_name, #args_struct_name),
            }
        });


        // Generate to_json match arms for ProgramInstruction enum
        let to_json_match_arms = instructions.iter().map(|ix| {
            let variant_name = syn::Ident::new(&ix.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let ix_name = &ix.name;
            quote! {
                Self::#variant_name(keys, data) => {
                    format!("{{\"instruction\":\"{}\",\"keys\":{},\"data\":{}}}", 
                        #ix_name, 
                        keys.to_json(), 
                        data.to_json()
                    )
                }
            }
        });

        // Generate match arms for parsing instructions with 1-byte discriminator
        let match_arms = instructions.iter().enumerate().map(|(_index, ix)| {
            let variant_name = syn::Ident::new(&ix.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let const_name = syn::Ident::new(
                &format!("{}_IX_DISCM", ix.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );
            let keys_struct_name = syn::Ident::new(
                &format!("{}Keys", ix.name.to_case(Case::Pascal)),
                proc_macro2::Span::call_site(),
            );
            let accounts_len_const = syn::Ident::new(
                &format!("{}_IX_ACCOUNTS_LEN", ix.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );

            let args_struct_name = syn::Ident::new(
                &format!("{}IxData", ix.name.to_case(Case::Pascal)),
                proc_macro2::Span::call_site(),
            );
            
            // 统一所有指令都处理 Keys 和 Args
            quote! {
                #const_name => {
                    if accounts.len() < #accounts_len_const {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Insufficient accounts for instruction {}", stringify!(#variant_name)),
                        ));
                    }
                    let keys = #keys_struct_name::from(&accounts[..#accounts_len_const]);
                    let args = #args_struct_name::from_bytes(&data[..])?;
                    Ok(ProgramInstruction::#variant_name(keys, args))
                },
            }
        });

        // Generate tests using the test template (only if --test flag is enabled)
        let tests = if !instructions.is_empty() && self.args.test {
            let test_generator = NonAnchorInstructionsParserTestTemplate::new();
            let test_content = test_generator.generate_instructions_consistency_tests(instructions, self.idl.program_name());
            
            quote! {
                #[cfg(test)]
                mod tests {
                    use super::*;
                    #test_content
                }
            }
        } else {
            quote! {}
        };

        quote! {
            //! Instructions parser for non-Anchor contracts with 1-byte discriminators
            
            // Import discriminator constants and other items from instructions module  
            use crate::instructions::*;
            use solana_pubkey::Pubkey;
            // 移除导入，使用完整路径 std::io::Write
            
            /// Program instruction types for non-Anchor contracts
            #[derive(Clone, Debug, PartialEq)]
            pub enum ProgramInstruction {
                #(#enum_variants)*
            }
            
            impl ProgramInstruction {
                /// Manual JSON serialization
                #[cfg(feature = "serde")]
                pub fn to_json(&self) -> String {
                    match self {
                        #(#to_json_match_arms)*
                    }
                }
            }
            
            /// Parse instruction data using 1-byte discriminator for non-Anchor contracts
            pub fn parse_instruction(data: &[u8], accounts: &[Pubkey]) -> Result<ProgramInstruction, std::io::Error> {
                if data.is_empty() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Empty instruction data"
                    ));
                }
                
                match data[0] {
                    #(#match_arms)*
                    _ => Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Unknown instruction discriminator: {}", data[0])
                    )),
                }
            }
            
            #tests
        }
    }

    fn generate_accounts_parser(&self) -> TokenStream {
        let accounts = self.idl.accounts.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if accounts.is_empty() {
            return quote! {};
        }

        // Generate account enum variants
        let account_variants = accounts.iter().map(|account| {
            let variant_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let struct_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            
            quote! {
                #variant_name(#struct_name),
            }
        });

        let program_name = syn::Ident::new(
            &format!("{}Account", self.idl.program_name().to_case(Case::Pascal)),
            proc_macro2::Span::call_site(),
        );

        // Generate length-based matching for non-Anchor accounts
        let match_arms = accounts.iter().map(|account| {
            let variant_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let struct_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            
            quote! {
                // Try to deserialize as this account type based on length
                if let Ok(account) = #struct_name::from_bytes(data) {
                    return Ok(#program_name::#variant_name(account));
                }
            }
        });

        // Generate tests using the test template (only if --test flag is enabled)
        let tests = if !accounts.is_empty() && self.args.test {
            let test_generator = NonAnchorAccountsParserTestTemplate::new();
            let test_content = test_generator.generate_accounts_consistency_tests(accounts, &self.idl.program_name());
            
            quote! {
                #[cfg(test)]
                mod tests {
                    use super::*;
                            #test_content
                }
            }
        } else {
            quote! {}
        };

        quote! {
            //! Account parser for non-Anchor contracts using length-based identification
            
            use crate::accounts::*;
            // 移除导入，使用完整路径 std::io::Error
            
            /// Program account types for non-Anchor contracts
            #[derive(Clone, Debug, PartialEq)]
            pub enum #program_name {
                #(#account_variants)*
            }
            
            /// Try to parse account data into one of the known account types using length-based identification
            pub fn try_unpack_account(data: &[u8]) -> Result<#program_name, std::io::Error> {
                if data.is_empty() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Empty account data"
                    ));
                }
                
                #(#match_arms)*
                
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unable to parse account data into any known account type (data length: {})", data.len())
                ))
            }
            
            #tests
        }
    }
}

impl<'a> TemplateGenerator for NonAnchorParsersTemplate<'a> {
    fn get_standard_module_name(&self) -> &'static str {
        "parsers"
    }

    fn gen_files(&self) -> Vec<(String, TokenStream)> {
        let instructions_parser = self.generate_instructions_parser();
        
        // 检查是否存在顶级accounts字段，只有存在时才生成accounts.rs
        let has_accounts = self.idl.accounts.as_ref().map_or(false, |accounts| !accounts.is_empty());
        
        let mut files = vec![
            ("instructions.rs".to_string(), instructions_parser),
        ];
        
        if has_accounts {
            let accounts_parser = self.generate_accounts_parser();
            files.push(("accounts.rs".to_string(), accounts_parser));
        }
        
        files
    }

    fn gen_mod_file(&self) -> TokenStream {
        // 检查是否存在顶级accounts字段，只有存在时才生成accounts模块声明和导入
        let has_accounts = self.idl.accounts.as_ref().map_or(false, |accounts| !accounts.is_empty());
        
        if has_accounts {
            quote! {
                //! Non-Anchor parser module
                //! Generated parsers for instructions and accounts with 1-byte discriminator and length-based identification
                
                pub mod instructions;
                pub mod accounts;
                
                pub use instructions::*;
                pub use accounts::*;
            }
        } else {
            quote! {
                //! Non-Anchor parser module
                //! Generated parsers for instructions with 1-byte discriminator
                
                pub mod instructions;
                
                pub use instructions::*;
            }
        }
    }
}