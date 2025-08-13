//! 测试工具
//!
//! 提供测试数据生成、mock 创建等测试辅助功能

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// 测试工具集
pub struct TestUtils;

impl TestUtils {
    /// 生成 mock 账户数组
    ///
    /// # Arguments
    /// * `count` - 账户数量
    ///
    /// # Returns
    /// * 生成创建 mock 账户的代码
    pub fn generate_mock_accounts(count: usize) -> TokenStream {
        let count_lit = proc_macro2::Literal::usize_unsuffixed(count);
        quote! {
            let mock_accounts = vec![Pubkey::default(); #count_lit];
        }
    }
    
    /// 生成默认参数结构体实例化代码
    ///
    /// # Arguments
    /// * `struct_name` - 参数结构体名称
    ///
    /// # Returns
    /// * 生成创建默认参数的代码
    pub fn generate_default_args(struct_name: &Ident) -> TokenStream {
        quote! {
            let test_args = #struct_name::default();
        }
    }
    
    /// 生成测试数据序列化代码
    ///
    /// # Arguments
    /// * `var_name` - 要序列化的变量名
    ///
    /// # Returns
    /// * 生成序列化测试数据的代码
    pub fn generate_test_data_serialization(var_name: &str) -> TokenStream {
        let var_ident: TokenStream = var_name.parse().unwrap();
        quote! {
            let test_data = #var_ident.try_to_vec().unwrap();
        }
    }
    
    /// 生成 Anchor 模式的 discriminator 测试数据
    ///
    /// # Arguments
    /// * `discm_const_name` - discriminator 常量名
    ///
    /// # Returns
    /// * 生成添加 8 字节 discriminator 的代码
    pub fn generate_anchor_discriminator_test_data(discm_const_name: &Ident) -> TokenStream {
        quote! {
            let mut test_data = Vec::new();
            test_data.extend_from_slice(&#discm_const_name);
        }
    }
    
    /// 生成非 Anchor 模式的 discriminator 测试数据
    ///
    /// # Arguments
    /// * `discm_const_name` - discriminator 常量名
    ///
    /// # Returns
    /// * 生成添加 1 字节 discriminator 的代码
    pub fn generate_non_anchor_discriminator_test_data(discm_const_name: &Ident) -> TokenStream {
        quote! {
            let mut test_data = Vec::new();
            test_data.push(#discm_const_name);
        }
    }
    
    /// 生成解析指令的测试代码
    ///
    /// # Arguments
    /// * `expected_variant` - 期望的指令变体名
    /// * `keys_struct_name` - Keys 结构体名
    /// * `accounts_len_const` - 账户长度常量名
    ///
    /// # Returns
    /// * 生成解析和验证指令的代码
    pub fn generate_instruction_parse_test(
        expected_variant: &Ident,
        keys_struct_name: &Ident,
        accounts_len_const: &Ident,
    ) -> TokenStream {
        quote! {
            let parsed = parse_instruction(&test_data, &mock_accounts).expect("Failed to parse instruction");
            
            match parsed {
                ProgramInstruction::#expected_variant(keys) => {
                    assert_eq!(keys, #keys_struct_name::from(&mock_accounts[..#accounts_len_const]));
                }
                _ => panic!("Parsed instruction has wrong type"),
            }
        }
    }
    
    /// 生成带参数指令的解析测试代码
    ///
    /// # Arguments
    /// * `expected_variant` - 期望的指令变体名
    /// * `keys_struct_name` - Keys 结构体名
    /// * `accounts_len_const` - 账户长度常量名
    /// * `args_struct_name` - Args 结构体名
    ///
    /// # Returns
    /// * 生成解析和验证带参数指令的代码
    pub fn generate_instruction_with_args_parse_test(
        expected_variant: &Ident,
        keys_struct_name: &Ident,
        accounts_len_const: &Ident,
        _args_struct_name: &Ident,
    ) -> TokenStream {
        quote! {
            match parse_instruction(&test_data, &mock_accounts) {
                Ok(ProgramInstruction::#expected_variant(keys, args)) => {
                    assert_eq!(keys, #keys_struct_name::from(&mock_accounts[..#accounts_len_const]));
                    assert_eq!(args, test_args);
                }
                Ok(_) => panic!("Discriminator matched wrong instruction type"),
                Err(e) => {
                    let error_msg = e.to_string();
                    assert!(!error_msg.contains("Unknown instruction discriminator"), 
                           "Should recognize discriminator for {}", stringify!(#expected_variant));
                }
            }
        }
    }
    
    /// 生成账户不足测试代码
    ///
    /// # Arguments
    /// * `accounts_len_const` - 账户长度常量名
    ///
    /// # Returns
    /// * 生成测试账户数量不足的代码
    pub fn generate_insufficient_accounts_test(accounts_len_const: &Ident) -> TokenStream {
        quote! {
            let insufficient_accounts = if #accounts_len_const > 0 {
                vec![Pubkey::default(); #accounts_len_const - 1]
            } else {
                vec![]
            };
            
            let result = parse_instruction(&test_data, &insufficient_accounts);
            assert!(result.is_err(), "Should fail with insufficient accounts");
            
            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("Insufficient accounts"), "Error should mention insufficient accounts");
        }
    }
    
    /// 生成错误处理测试
    ///
    /// # Arguments
    /// * `is_anchor` - 是否为 Anchor 模式
    ///
    /// # Returns
    /// * 生成错误情况测试的代码
    pub fn generate_error_tests(is_anchor: bool) -> TokenStream {
        if is_anchor {
            quote! {
                #[test]
                fn test_invalid_discriminator() {
                    let mock_accounts = vec![Pubkey::default(); 10];
                    let invalid_data = vec![255u8; 40]; // Invalid discriminator + some data
                    let result = parse_instruction(&invalid_data, &mock_accounts);
                    assert!(result.is_err(), "Should fail with invalid discriminator");
                }

                #[test]
                fn test_insufficient_data() {
                    let mock_accounts = vec![Pubkey::default(); 10];
                    let short_data = vec![1u8; 4]; // Less than 8 bytes for Anchor
                    let result = parse_instruction(&short_data, &mock_accounts);
                    assert!(result.is_err(), "Should fail with insufficient data");
                }
            }
        } else {
            quote! {
                #[test]
                fn test_invalid_discriminator() {
                    let mock_accounts = vec![Pubkey::default(); 10];
                    let invalid_data = vec![255u8; 40]; // Invalid discriminator + some data
                    let result = parse_instruction(&invalid_data, &mock_accounts);
                    assert!(result.is_err(), "Should fail with invalid discriminator");
                }

                #[test]
                fn test_insufficient_data() {
                    let mock_accounts = vec![Pubkey::default(); 10];
                    let empty_data = vec![]; // Empty data
                    let result = parse_instruction(&empty_data, &mock_accounts);
                    assert!(result.is_err(), "Should fail with empty data");
                }
            }
        }
    }
}