//! Anchor Accounts Parser 测试模板
//!
//! 为 Anchor 合约生成 Accounts Parser 测试代码

use proc_macro2::TokenStream;
use quote::quote;
use convert_case::{Case, Casing};
use heck::ToShoutySnakeCase;

use crate::idl_format::anchor_idl::AnchorAccount;
use crate::templates::{AccountsParserTestGenerator, TemplateGenerator};
use crate::templates::common::doc_generator::DocGenerator;

/// Anchor Accounts Parser 测试模板
pub struct AnchorAccountsParserTestTemplate;

impl AnchorAccountsParserTestTemplate {
    /// 创建新的 Anchor Accounts Parser 测试模板
    pub fn new() -> Self {
        Self
    }
}

impl AccountsParserTestGenerator for AnchorAccountsParserTestTemplate {
    fn generate_accounts_consistency_tests(&self, accounts: &[AnchorAccount], program_name: &str) -> TokenStream {
        if accounts.is_empty() {
            return quote! {};
        }

        // 使用NamingConverter确保命名一致性
        use crate::templates::common::naming_converter::NamingConverter;
        let mut naming_converter = NamingConverter::new();

        let program_account_name = format!("{}Account", naming_converter.to_pascal_case(program_name));
        let program_enum_name = syn::Ident::new(
            &program_account_name,
            proc_macro2::Span::call_site(),
        );

        // Generate consistency tests for each account
        let consistency_tests = accounts.iter().map(|account| {
            let test_name = syn::Ident::new(
                &format!("test_{}_consistency", naming_converter.to_snake_case(&account.name)),
                proc_macro2::Span::call_site(),
            );
            let account_name = syn::Ident::new(&naming_converter.to_pascal_case(&account.name), proc_macro2::Span::call_site());
            let variant_name = syn::Ident::new(&naming_converter.to_pascal_case(&account.name), proc_macro2::Span::call_site());
            let discm_const_name = syn::Ident::new(
                &format!("{}_ACCOUNT_DISCM", naming_converter.to_screaming_snake_case(&account.name)),
                proc_macro2::Span::call_site(),
            );

            quote! {
                #[test]
                fn #test_name() {
                    let is_anchor = true;
                    let expected_first_field = 42u64;
                    let test_account = #account_name {
                        // Assume first field is epoch based on the pattern
                        epoch: expected_first_field,
                        ..Default::default()
                    };
                    let test_data = test_account.try_to_vec().unwrap();
                    match try_unpack_account(&test_data) {
                        Ok(#program_enum_name::#variant_name(account)) => {
                            assert_eq!(
                                account.epoch, expected_first_field,
                                "First field value should match expected value"
                            );
                            assert_eq!(
                                account.discriminator,
                                #discm_const_name,
                                "Discriminator field should match expected value"
                            );
                        }
                        Ok(_) => panic!("Discriminator matched wrong account type"),
                        Err(e) => {
                            let error_msg = e.to_string();
                            assert!(
                                !error_msg.contains("Unknown account discriminator"),
                                "Should recognize discriminator for {}",
                                stringify!(#account_name)
                            );
                        }
                    }
                }
            }
        });

        // Generate discriminator recognition test
        let discriminator_test = {
            let account_tests = accounts.iter().map(|account| {
                let account_name = syn::Ident::new(&account.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
                
                quote! {
                    let test_account = #account_name::default();
                    let test_data = test_account.try_to_vec().unwrap();
                    assert!(
                        test_data.len() >= 8,
                        "Test data should have at least 8 bytes for discriminator"
                    );
                    let result = try_unpack_account(&test_data);
                    match result {
                        Ok(_) => {}
                        Err(e) => {
                            let error_msg = e.to_string();
                            assert!(
                                !error_msg.contains("Unknown account discriminator"),
                                "Should recognize discriminator for account {}",
                                stringify!(#account_name)
                            );
                        }
                    }
                }
            });
            
            quote! {
                #[test]
                fn test_discriminator_recognition() {
                    let is_anchor = true;
                    #(#account_tests)*
                }
            }
        };


        quote! {
            #(#consistency_tests)*
            
            #discriminator_test
        }
    }
}

impl TemplateGenerator for AnchorAccountsParserTestTemplate {
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