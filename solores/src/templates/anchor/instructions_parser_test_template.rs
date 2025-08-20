//! Anchor Instructions Parser 测试模板
//!
//! 为 Anchor 合约生成 Instructions Parser 测试代码

use proc_macro2::TokenStream;
use quote::quote;
use convert_case::{Case, Casing};
use heck::ToShoutySnakeCase;

use crate::idl_format::anchor_idl::AnchorInstruction;
use crate::templates::{InstructionsParserTestGenerator, TemplateGenerator};

/// Anchor Instructions Parser 测试模板
pub struct AnchorInstructionsParserTestTemplate;

impl AnchorInstructionsParserTestTemplate {
    /// 创建新的 Anchor Instructions Parser 测试模板
    pub fn new() -> Self {
        Self
    }
}

impl InstructionsParserTestGenerator for AnchorInstructionsParserTestTemplate {
    fn generate_instructions_consistency_tests(&self, instructions: &[AnchorInstruction], program_name: &str) -> TokenStream {
        if instructions.is_empty() {
            return quote! {};
        }

        let program_enum_name = syn::Ident::new(
            &format!("{}Instruction", program_name.to_case(Case::Pascal)),
            proc_macro2::Span::call_site(),
        );

        // Generate consistency tests for each instruction
        let consistency_tests = instructions.iter().map(|ix| {
            let test_name = syn::Ident::new(
                &format!("test_{}_consistency", ix.name),
                proc_macro2::Span::call_site(),
            );
            let variant_name = syn::Ident::new(&ix.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let keys_struct_name = syn::Ident::new(&format!("{}Keys", ix.name.to_case(Case::Pascal)), proc_macro2::Span::call_site());
            let accounts_len_const = syn::Ident::new(&format!("{}_IX_ACCOUNTS_LEN", ix.name.to_shouty_snake_case()), proc_macro2::Span::call_site());
            let _discm_const_name = syn::Ident::new(
                &format!("{}_IX_DISCM", ix.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );

            // All instructions now have IxData struct (even if only contains discriminator)
            let args_struct_name = syn::Ident::new(
                &format!("{}IxData", ix.name.to_case(Case::Pascal)),
                proc_macro2::Span::call_site(),
            );
            
            quote! {
                #[test]
                fn #test_name() {
                    let mock_accounts = vec![Pubkey::default(); #accounts_len_const];
                    let test_keys = #keys_struct_name::from(&mock_accounts[..#accounts_len_const]);
                    let test_args = #args_struct_name::default();
                    
                    // Use direct Args serialization (discriminator included in Default impl)
                    let test_data = test_args.try_to_vec().unwrap();
                    
                    match parse_instruction(&test_data, &mock_accounts) {
                        Ok(#program_enum_name::#variant_name(keys, args)) => {
                            // Verify key consistency
                            assert_eq!(keys, test_keys);
                            // Verify args consistency  
                            assert_eq!(args, test_args);
                        }
                        Ok(_) => panic!("Discriminator matched wrong instruction type"),
                        Err(e) => {
                            let error_msg = e.to_string();
                            assert!(
                                !error_msg.contains("Unknown instruction discriminator"),
                                "Should recognize discriminator for {}",
                                stringify!(#variant_name)
                            );
                        }
                    }
                }
            }
        });

        // Generate insufficient accounts tests
        let insufficient_account_tests = instructions.iter().map(|ix| {
            let test_name = syn::Ident::new(
                &format!("test_{}_insufficient_accounts", ix.name),
                proc_macro2::Span::call_site(),
            );
            let _variant_name = syn::Ident::new(&ix.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let _keys_struct_name = syn::Ident::new(&format!("{}Keys", ix.name.to_case(Case::Pascal)), proc_macro2::Span::call_site());
            let accounts_len_const = syn::Ident::new(&format!("{}_IX_ACCOUNTS_LEN", ix.name.to_shouty_snake_case()), proc_macro2::Span::call_site());
            
            // Every instruction now has Args struct (even if only contains discriminator)
            let args_struct_name = syn::Ident::new(
                &format!("{}IxData", ix.name.to_case(Case::Pascal)),
                proc_macro2::Span::call_site(),
            );

            quote! {
                #[test]
                fn #test_name() {
                    let insufficient_accounts = if #accounts_len_const > 0 {
                        vec![Pubkey::default(); #accounts_len_const - 1]
                    } else {
                        vec![]
                    };
                    
                    // Create test data using Args (discriminator included in Default impl)
                    let test_args = #args_struct_name::default();
                    let test_data = test_args.try_to_vec().unwrap();
                    
                    // Test with insufficient accounts
                    let result = parse_instruction(&test_data, &insufficient_accounts);
                    assert!(result.is_err(), "Should fail with insufficient accounts");
                    let error_msg = result.unwrap_err().to_string();
                    assert!(
                        error_msg.contains("Insufficient accounts"),
                        "Error should mention insufficient accounts"
                    );
                }
            }
        });


        quote! {
            #(#consistency_tests)*
            
            #(#insufficient_account_tests)*
        }
    }
}

impl TemplateGenerator for AnchorInstructionsParserTestTemplate {
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