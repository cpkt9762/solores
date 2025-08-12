//! Anchor Parsers 模板
//!
//! 为 Anchor 合约生成 Parsers 相关代码，使用 8 字节 discriminator

use proc_macro2::TokenStream;
use quote::quote;

use crate::idl_format::anchor_idl::AnchorIdl;
use crate::idl_format::IdlFormat;
use crate::Args;
use crate::templates::{TemplateGenerator, ParsersTemplateGenerator};
use crate::templates::common::import_manager::ImportManager;
use crate::templates::anchor::instructions_template::AnchorInstructionsTemplate;
use crate::templates::anchor::accounts_template::AnchorAccountsTemplate;
use crate::templates::anchor::instructions_parser_test_template::AnchorInstructionsParserTestTemplate;
use crate::templates::anchor::accounts_parser_test_template::AnchorAccountsParserTestTemplate;
use crate::templates::{InstructionsParserTestGenerator, AccountsParserTestGenerator};

/// Anchor Parsers 模板
pub struct AnchorParsersTemplate<'a> {
    pub idl: &'a AnchorIdl,
    pub args: &'a Args,
}

impl<'a> AnchorParsersTemplate<'a> {
    /// 创建新的 Anchor Parsers 模板
    pub fn new(idl: &'a AnchorIdl, args: &'a Args) -> Self {
        Self { idl, args }
    }
}

impl<'a> ParsersTemplateGenerator for AnchorParsersTemplate<'a> {
    fn generate_instructions_parser(&self) -> TokenStream {
        // Reuse the instructions template logic with proper imports
        let instructions_template = AnchorInstructionsTemplate::new(self.idl, self.args);
        let imports = ImportManager::generate_parser_imports();
        let enum_def = instructions_template.generate_instruction_enum();
        let parse_function = instructions_template.generate_parse_function();
        
        // Generate helper function for account validation
        let helper_functions = quote! {
            /// Helper function to check minimum accounts requirement
            fn check_min_accounts_req(
                accounts: &[Pubkey],
                required_len: usize,
                instruction_name: &str,
            ) -> Result<(), std::io::Error> {
                if accounts.len() < required_len {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Insufficient accounts for instruction {}", instruction_name),
                    ));
                }
                Ok(())
            }
        };

        // Generate tests using the test template (only if --test flag is enabled)
        let instructions = self.idl.instructions.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        let tests = if !instructions.is_empty() && self.args.test {
            let test_generator = AnchorInstructionsParserTestTemplate::new();
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
            //! Instructions parser for Anchor contracts with 8-byte discriminators
            
            #imports
            
            // Import discriminator constants and other items from instructions module
            use crate::instructions::*;
            
            #enum_def
            
            #helper_functions
            
            #parse_function
            
            #tests
        }
    }

    fn generate_accounts_parser(&self) -> TokenStream {
        let accounts = self.idl.accounts.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if accounts.is_empty() {
            return quote! {};
        }

        // 使用NamingConverter来确保命名规范一致性
        use crate::templates::common::naming_converter::NamingConverter;
        use std::cell::RefCell;
        let naming_converter = RefCell::new(NamingConverter::new());
        
        // Generate account enum and parsing logic
        let account_variants = accounts.iter().map(|account| {
            let variant_name = syn::Ident::new(
                &naming_converter.borrow_mut().to_pascal_case(&account.name), 
                proc_macro2::Span::call_site()
            );
            let struct_name = syn::Ident::new(
                &naming_converter.borrow_mut().to_pascal_case(&account.name), 
                proc_macro2::Span::call_site()
            );
            
            quote! {
                #variant_name(#struct_name),
            }
        });

        let program_account_name = format!("{}Account", naming_converter.borrow_mut().to_pascal_case(&self.idl.program_name()));
        let program_name = syn::Ident::new(
            &program_account_name,
            proc_macro2::Span::call_site(),
        );

        // Generate match arms for account parsing
        let match_arms = accounts.iter().map(|account| {
            let variant_name = syn::Ident::new(
                &naming_converter.borrow_mut().to_pascal_case(&account.name), 
                proc_macro2::Span::call_site()
            );
            let struct_name = syn::Ident::new(
                &naming_converter.borrow_mut().to_pascal_case(&account.name), 
                proc_macro2::Span::call_site()
            );
            
            quote! {
                if let Ok(account) = #struct_name::from_bytes(data) {
                    return Ok(#program_name::#variant_name(account));
                }
            }
        });

        // Generate tests using the test template (only if --test flag is enabled)
        let tests = if !accounts.is_empty() && self.args.test {
            let test_generator = AnchorAccountsParserTestTemplate::new();
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

        let imports = ImportManager::generate_parser_imports();
        
        quote! {
            //! Account parser for Anchor contracts with 8-byte discriminators
            
            #imports
            use crate::accounts::*;
            // 移除导入，使用完整路径 std::io::Error
            
            /// Program account types
            #[derive(Clone, Debug, PartialEq)]
            pub enum #program_name {
                #(#account_variants)*
            }
            
            /// Try to parse account data into one of the known account types
            pub fn try_unpack_account(data: &[u8]) -> Result<#program_name, std::io::Error> {
                #(#match_arms)*
                
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Unable to parse account data into any known account type"
                ))
            }
            
            #tests
        }
    }
}

impl<'a> TemplateGenerator for AnchorParsersTemplate<'a> {
    fn get_standard_module_name(&self) -> &'static str {
        "parsers"
    }

    fn gen_files(&self) -> Vec<(String, TokenStream)> {
        let instructions_parser = self.generate_instructions_parser();
        let accounts_parser = self.generate_accounts_parser();
        
        vec![
            ("instructions.rs".to_string(), instructions_parser),
            ("accounts.rs".to_string(), accounts_parser),
        ]
    }

    fn gen_mod_file(&self) -> TokenStream {
        quote! {
            //! Anchor parser module
            //! Generated parsers for instructions and accounts with 8-byte discriminator support
            
            pub mod instructions;
            pub mod accounts;
            
            pub use instructions::*;
            pub use accounts::*;
        }
    }
}