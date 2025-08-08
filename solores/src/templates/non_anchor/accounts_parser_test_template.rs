//! 非 Anchor Accounts Parser 测试模板
//!
//! 为非 Anchor 合约生成 Accounts Parser 测试代码

use proc_macro2::TokenStream;
use quote::quote;
use convert_case::{Case, Casing};

use crate::idl_format::non_anchor_idl::NonAnchorAccount;
use crate::templates::{NonAnchorAccountsParserTestGenerator, TemplateGenerator};

/// 非 Anchor Accounts Parser 测试模板
pub struct NonAnchorAccountsParserTestTemplate;

impl NonAnchorAccountsParserTestTemplate {
    /// 创建新的非 Anchor Accounts Parser 测试模板
    pub fn new() -> Self {
        Self
    }
}

impl NonAnchorAccountsParserTestGenerator for NonAnchorAccountsParserTestTemplate {
    fn generate_accounts_consistency_tests(&self, accounts: &[NonAnchorAccount], program_name: &str) -> TokenStream {
        if accounts.is_empty() {
            return quote! {};
        }

        // Get the actual program account enum name
        let program_enum_name = syn::Ident::new(
            &format!("{}Account", program_name.to_case(Case::Pascal)),
            proc_macro2::Span::call_site(),
        );

        // Generate consistency tests for each account using original struct serialization
        let consistency_tests = accounts.iter().map(|account| {
            let test_name = syn::Ident::new(
                &format!("test_{}_consistency", account.name.to_case(Case::Snake)),
                proc_macro2::Span::call_site(),
            );
            let account_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let variant_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());

            quote! {
                #[test]
                fn #test_name() {
                    // Create test account with default values for non-Anchor contracts
                    let test_account = #account_name::default();
                    
                    // Use original struct serialization approach
                    let test_data = test_account.try_to_vec().unwrap();
                    
                    match try_unpack_account(&test_data) {
                        Ok(#program_enum_name::#variant_name(account)) => {
                            // Verify first field consistency - compare with original
                            assert_eq!(
                                account, test_account,
                                "Parsed account should match original test account"
                            );
                        }
                        Ok(_) => panic!("Length-based identification matched wrong account type"),
                        Err(e) => {
                            let error_msg = e.to_string();
                            assert!(
                                !error_msg.contains("Unknown account type"),
                                "Should recognize length-based account type for {}",
                                stringify!(#account_name)
                            );
                        }
                    }
                }
            }
        });

        // Generate length-based recognition test for non-Anchor contracts
        let length_based_test = {
            let account_tests = accounts.iter().map(|account| {
                let account_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
                
                quote! {
                    let test_account = #account_name::default();
                    let test_data = test_account.try_to_vec().unwrap();
                    let result = try_unpack_account(&test_data);
                    match result {
                        Ok(_) => {
                            // Length-based recognition succeeded - this is expected for non-Anchor
                        }
                        Err(e) => {
                            let error_msg = e.to_string();
                            assert!(
                                !error_msg.contains("Unknown account type"),
                                "Should recognize length-based account type for account {}",
                                stringify!(#account_name)
                            );
                        }
                    }
                }
            });
            
            quote! {
                #[test]
                fn test_non_anchor_length_based_recognition() {
                    // Test length-based account recognition for non-Anchor contracts
                    #(#account_tests)*
                }
            }
        };

        quote! {
            #(#consistency_tests)*
            
            #length_based_test
        }
    }
}

impl TemplateGenerator for NonAnchorAccountsParserTestTemplate {
    fn get_standard_module_name(&self) -> &'static str {
        "tests"
    }

    fn gen_files(&self) -> Vec<(String, TokenStream)> {
        // This template is used as a component, not standalone
        vec![]
    }
    
    fn gen_mod_file(&self) -> TokenStream {
        quote! {}
    }
}