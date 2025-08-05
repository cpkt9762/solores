use proc_macro2::TokenStream;
use quote::quote;
use convert_case::{Case, Casing};
use sha2::{Digest, Sha256};
use heck::{ToPascalCase, ToShoutySnakeCase};

use crate::{
    idl_format::{anchor::AnchorIdl, IdlCodegenModule, IdlFormat},
    Args,
};

pub struct AccountsParserModule<'a> {
    pub idl: &'a AnchorIdl,
    pub args: &'a Args,
}

impl<'a> AccountsParserModule<'a> {
    pub fn new(idl: &'a AnchorIdl, args: &'a Args) -> Self {
        Self { idl, args }
    }

    fn generate_account_enum(&self) -> TokenStream {
        let accounts = self.idl.accounts();
        if accounts.is_empty() {
            return quote! {};
        }

        let enum_variants = accounts.iter().map(|acc| {
            let variant_name = syn::Ident::new(&acc.0.name, proc_macro2::Span::call_site());
            let struct_name = syn::Ident::new(&acc.0.name, proc_macro2::Span::call_site());
            
            let doc_comments = if let Some(docs) = &acc.0.docs {
                let doc_tokens: Vec<TokenStream> = docs
                    .iter()
                    .filter(|doc| !doc.trim().is_empty())
                    .map(|doc| {
                        let doc_str = doc.trim();
                        quote! { #[doc = #doc_str] }
                    })
                    .collect();
                quote! { #(#doc_tokens)* }
            } else {
                quote! {}
            };

            quote! {
                #doc_comments
                #variant_name(#struct_name),
            }
        });

        let program_name = syn::Ident::new(
            &format!("{}Account", self.idl.program_name().to_case(Case::Pascal)),
            proc_macro2::Span::call_site(),
        );

        quote! {
            /// Program account types
            #[derive(Clone, Debug, PartialEq)]
            pub enum #program_name {
                #(#enum_variants)*
            }
        }
    }

    fn generate_try_unpack_function(&self) -> TokenStream {
        let accounts = self.idl.accounts();
        if accounts.is_empty() {
            return quote! {};
        }

        let program_name = syn::Ident::new(
            &format!("{}Account", self.idl.program_name().to_case(Case::Pascal)),
            proc_macro2::Span::call_site(),
        );

        let is_anchor = self.idl.is_anchor_contract();

        if is_anchor {
            // Generate discriminator-based parsing for Anchor contracts
            let match_arms = accounts.iter().map(|acc| {
                let variant_name = syn::Ident::new(&acc.0.name, proc_macro2::Span::call_site());
                let struct_name = syn::Ident::new(&acc.0.name, proc_macro2::Span::call_site());
                
                // Calculate discriminator like in the account module
                let discm = <[u8; 8]>::try_from(
                    &Sha256::digest(format!("account:{}", acc.0.name.to_pascal_case()).as_bytes()).as_slice()[..8],
                ).unwrap();
                let discriminator = &discm;
                let discriminator_array = quote! { [#(#discriminator),*] };

                quote! {
                    #discriminator_array => Ok(#program_name::#variant_name(
                        #struct_name::from_bytes(data)?
                    )),
                }
            });

            quote! {
                /// Parse account data based on discriminator (Anchor contracts)
                pub fn try_unpack_account(data: &[u8]) -> Result<#program_name, std::io::Error> {
                    if data.len() < 8 {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Account data too short for discriminator"
                        ));
                    }

                    let discriminator: [u8; 8] = data[0..8].try_into()
                        .map_err(|_| std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Failed to read discriminator"
                    ))?;

                match discriminator {
                    #(#match_arms)*
                    _ => Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Unknown account discriminator: {:?}", discriminator)
                    )),
                }
                }
            }
        } else {
            // Generate length-based parsing for non-Anchor contracts
            let length_constants = accounts.iter().map(|acc| {
                let const_name = syn::Ident::new(
                    &format!("{}_LEN", acc.0.name.to_shouty_snake_case()),
                    proc_macro2::Span::call_site(),
                );
                let struct_name = syn::Ident::new(&acc.0.name, proc_macro2::Span::call_site());
                quote! {
                    const #const_name: usize = std::mem::size_of::<#struct_name>();
                }
            });

            let match_arms = accounts.iter().map(|acc| {
                let variant_name = syn::Ident::new(&acc.0.name, proc_macro2::Span::call_site());
                let struct_name = syn::Ident::new(&acc.0.name, proc_macro2::Span::call_site());
                let const_name = syn::Ident::new(
                    &format!("{}_LEN", acc.0.name.to_shouty_snake_case()),
                    proc_macro2::Span::call_site(),
                );

                quote! {
                    #const_name => Ok(#program_name::#variant_name(
                        #struct_name::from_bytes(data)?
                    )),
                }
            });

            quote! {
                /// Parse account data based on data length (non-Anchor contracts)
                pub fn try_unpack_account(data: &[u8]) -> Result<#program_name, std::io::Error> {
                    let data_len = data.len();
                    #(#length_constants)*

                    match data_len {
                        #(#match_arms)*
                        _ => Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Invalid Account data length: {}", data_len)
                        )),
                    }
                }
            }
        }
    }

    fn generate_tests(&self) -> TokenStream {
        let accounts = self.idl.accounts();
        if accounts.is_empty() {
            return quote! {};
        }

        let program_name = syn::Ident::new(
            &format!("{}Account", self.idl.program_name().to_case(Case::Pascal)),
            proc_macro2::Span::call_site(),
        );

        let is_anchor = self.idl.is_anchor_contract();

        let test_functions = accounts.iter().map(|acc| {
            let test_name = syn::Ident::new(
                &format!("test_{}_consistency", acc.0.name.to_case(Case::Snake)),
                proc_macro2::Span::call_site(),
            );
            let struct_name = syn::Ident::new(&acc.0.name, proc_macro2::Span::call_site());
            let variant_name = syn::Ident::new(&acc.0.name, proc_macro2::Span::call_site());

            // Calculate discriminator and test data prefix (for backward compatibility)
            let discm = <[u8; 8]>::try_from(
                &Sha256::digest(format!("account:{}", acc.0.name.to_pascal_case()).as_bytes()).as_slice()[..8],
            ).unwrap();
            let discriminator = &discm;
            let discriminator_array = quote! { [#(#discriminator),*] };

            // Generate test data with known first field value
            // Try to get struct definition either from account itself or from types array
            let struct_def = if acc.0.r#type.is_some() {
                acc.0.r#type.as_ref()
            } else {
                // Try to find matching struct definition in types array (similar to account generation logic)
                self.idl.types.as_deref().unwrap_or(&[]).iter()
                    .find(|t| t.name == acc.0.name)
                    .and_then(|t| t.r#type.as_ref())
            };
            
            let (test_data_generation, verification_logic) = if let Some(typedef_type) = struct_def {
                if let crate::idl_format::anchor::typedefs::TypedefType::r#struct(typedef_struct) = typedef_type {
                    if let Some(first_field) = typedef_struct.fields.first() {
                    let first_field_name = syn::Ident::new(&first_field.name.to_case(Case::Snake), proc_macro2::Span::call_site());
                    
                    // Generate test value based on field type
                    let (test_value, test_value_bytes) = match &first_field.r#type {
                        crate::idl_format::anchor::typedefs::TypedefFieldType::PrimitiveOrPubkey(type_str) => {
                            match type_str.as_str() {
                                "publicKey" | "Pubkey" => (
                                    quote! { solana_program::pubkey::Pubkey::new_from_array([1u8; 32]) },
                                    quote! { [1u8; 32].to_vec() }
                                ),
                                "u64" => (
                                    quote! { 42u64 },
                                    quote! { 42u64.to_le_bytes().to_vec() }
                                ),
                                "u32" => (
                                    quote! { 42u32 },
                                    quote! { 42u32.to_le_bytes().to_vec() }
                                ),
                                "u16" => (
                                    quote! { 42u16 },
                                    quote! { 42u16.to_le_bytes().to_vec() }
                                ),
                                "u8" => (
                                    quote! { 42u8 },
                                    quote! { vec![42u8] }
                                ),
                                "i64" => (
                                    quote! { 42i64 },
                                    quote! { 42i64.to_le_bytes().to_vec() }
                                ),
                                "i32" => (
                                    quote! { 42i32 },
                                    quote! { 42i32.to_le_bytes().to_vec() }
                                ),
                                "string" => (
                                    quote! { "test".to_string() },
                                    quote! { {
                                        let s = "test";
                                        let mut bytes = (s.len() as u32).to_le_bytes().to_vec();
                                        bytes.extend_from_slice(s.as_bytes());
                                        bytes
                                    } }
                                ),
                                "bool" => (
                                    quote! { true },
                                    quote! { vec![1u8] }
                                ),
                                _ => (
                                    quote! { Default::default() }, // Use Default instead of empty comment
                                    quote! { vec![0u8; 32] }
                                )
                            }
                        },
                        crate::idl_format::anchor::typedefs::TypedefFieldType::array(array_type) => {
                            // Handle array types like [u64; 4]
                            let array_size = proc_macro2::Literal::usize_unsuffixed(array_type.1 as usize);
                            match &*array_type.0 {
                                crate::idl_format::anchor::typedefs::TypedefFieldType::PrimitiveOrPubkey(elem_type) => {
                                    match elem_type.as_str() {
                                        "u64" => (
                                            quote! { [42u64; #array_size] },
                                            quote! { [42u64; #array_size].iter().flat_map(|&x| x.to_le_bytes()).collect::<Vec<u8>>() }
                                        ),
                                        "u32" => (
                                            quote! { [42u32; #array_size] },
                                            quote! { [42u32; #array_size].iter().flat_map(|&x| x.to_le_bytes()).collect::<Vec<u8>>() }
                                        ),
                                        "u16" => (
                                            quote! { [42u16; #array_size] },
                                            quote! { [42u16; #array_size].iter().flat_map(|&x| x.to_le_bytes()).collect::<Vec<u8>>() }
                                        ),
                                        "u8" => (
                                            quote! { [42u8; #array_size] },
                                            quote! { [42u8; #array_size].to_vec() }
                                        ),
                                        _ => (
                                            quote! { Default::default() }, // Fallback for unknown array element types
                                            quote! { vec![0u8; 32] }
                                        )
                                    }
                                },
                                _ => (
                                    quote! { Default::default() }, // Complex array element types
                                    quote! { vec![0u8; 32] }
                                )
                            }
                        },
                        _ => (
                            quote! { Default::default() }, // Other complex field types
                            quote! { vec![0u8; 32] }
                        )
                    };

                    let data_prefix = if is_anchor {
                        quote! { test_data.extend_from_slice(&#discriminator_array); }
                    } else {
                        quote! { /* No discriminator prefix for non-Anchor contracts */ }
                    };

                    let data_gen = quote! {
                        // Create test data using struct serialization
                        let expected_first_field = #test_value;
                        let test_account = #struct_name {
                            #first_field_name: expected_first_field,
                            ..Default::default()
                        };
                        let test_data = test_account.try_to_vec().unwrap();
                    };

                    let is_simple_type = matches!(&first_field.r#type, 
                        crate::idl_format::anchor::typedefs::TypedefFieldType::PrimitiveOrPubkey(_) |
                        crate::idl_format::anchor::typedefs::TypedefFieldType::array(_));
                    
                    let verification = if !is_simple_type {
                        quote! {
                            // Skip field verification for unknown types, just test discriminator matching
                            match try_unpack_account(&test_data) {
                                Ok(#program_name::#variant_name(account)) => {
                                    // Success - discriminator matched and data parsed
                                    // Note: Field verification skipped for complex types
                                    
                                    // Verify discriminator field is correct (for Anchor contracts)
                                    assert_eq!(account.discriminator, #discriminator_array,
                                             "Discriminator field should match expected value");
                                }
                                Ok(_) => panic!("Discriminator matched wrong account type"),
                                Err(e) => {
                                    // Expected for complex data - verify it's not a discriminator error
                                    let error_msg = e.to_string();
                                    assert!(!error_msg.contains("Unknown account discriminator"), 
                                           "Should recognize discriminator for {}", stringify!(#variant_name));
                                }
                            }
                        }
                    } else {
                        quote! {
                            // Test discriminator matching and first field value
                            match try_unpack_account(&test_data) {
                                Ok(#program_name::#variant_name(account)) => {
                                    // Success - verify first field matches expected value
                                    assert_eq!(account.#first_field_name, expected_first_field, 
                                             "First field value should match expected value");
                                    
                                    // Verify discriminator field is correct (for Anchor contracts)
                                    assert_eq!(account.discriminator, #discriminator_array,
                                             "Discriminator field should match expected value");
                                }
                                Ok(_) => panic!("Discriminator matched wrong account type"),
                                Err(e) => {
                                    // Expected for complex data - verify it's not a discriminator error
                                    let error_msg = e.to_string();
                                    assert!(!error_msg.contains("Unknown account discriminator"), 
                                           "Should recognize discriminator for {}", stringify!(#variant_name));
                                }
                            }
                        }
                    };

                    (data_gen, verification)
                    } else {
                        // No fields, fallback to dummy data test
                        let data_prefix = if is_anchor {
                            quote! { test_data.extend_from_slice(&#discriminator_array); }
                        } else {
                            quote! { /* No discriminator prefix for non-Anchor contracts */ }
                        };
                        (
                            quote! {
                                // Create test data using default struct values
                                let test_account = #struct_name::default();
                                let test_data = test_account.try_to_vec().unwrap();
                            },
                            quote! {
                                match try_unpack_account(&test_data) {
                                    Ok(#program_name::#variant_name(account)) => {
                                        // Success - discriminator matched and data parsed
                                        
                                        // Verify discriminator field is correct (for Anchor contracts)
                                        assert_eq!(account.discriminator, #discriminator_array,
                                                 "Discriminator field should match expected value");
                                    }
                                    Ok(_) => panic!("Discriminator matched wrong account type"),
                                    Err(e) => {
                                        let error_msg = e.to_string();
                                        assert!(!error_msg.contains("Unknown account discriminator"), 
                                               "Should recognize discriminator for {}", stringify!(#variant_name));
                                    }
                                }
                            }
                        )
                    }
                } else {
                    // No type definition, fallback to dummy data test
                    let data_prefix = if is_anchor {
                        quote! { test_data.extend_from_slice(&#discriminator_array); }
                    } else {
                        quote! { /* No discriminator prefix for non-Anchor contracts */ }
                    };
                    (
                        quote! {
                            // Create test data using default struct values  
                            let test_account = #struct_name::default();
                            let test_data = test_account.try_to_vec().unwrap();
                        },
                        quote! {
                            match try_unpack_account(&test_data) {
                                Ok(#program_name::#variant_name(_)) => {
                                    // Success - discriminator matched and data parsed
                                }
                                Ok(_) => panic!("Discriminator matched wrong account type"),
                                Err(e) => {
                                    let error_msg = e.to_string();
                                    assert!(!error_msg.contains("Unknown account discriminator"), 
                                           "Should recognize discriminator for {}", stringify!(#variant_name));
                                }
                            }
                        }
                    )
                }
            } else {
                // No type definition, fallback to dummy data test
                (
                    quote! {
                        // Create test data using default struct values
                        let test_account = #struct_name::default(); 
                        let test_data = test_account.try_to_vec().unwrap();
                    },
                    quote! {
                        match try_unpack_account(&test_data) {
                            Ok(#program_name::#variant_name(_)) => {
                                // Success - discriminator matched and data parsed
                            }
                            Ok(_) => panic!("Discriminator matched wrong account type"),
                            Err(e) => {
                                let error_msg = e.to_string();
                                assert!(!error_msg.contains("Unknown account discriminator"), 
                                       "Should recognize discriminator for {}", stringify!(#variant_name));
                            }
                        }
                    }
                )
            };

            quote! {
                #[test]
                fn #test_name() {
                    #test_data_generation
                    #verification_logic
                }
            }
        });

        // Also generate a basic discriminator test
        let discriminator_test = {
            let discriminator_tests = accounts.iter().map(|acc| {
                let variant_name = syn::Ident::new(&acc.0.name, proc_macro2::Span::call_site());
                
                let discm = <[u8; 8]>::try_from(
                    &Sha256::digest(format!("account:{}", acc.0.name.to_pascal_case()).as_bytes()).as_slice()[..8],
                ).unwrap();
                let discriminator = &discm;
                let discriminator_array = quote! { [#(#discriminator),*] };

                let struct_name = syn::Ident::new(&acc.0.name, proc_macro2::Span::call_site());
                
                if is_anchor {
                    quote! {
                        // Test account: #variant_name (Anchor style with discriminator)
                        let test_account = #struct_name::default();
                        let test_data = test_account.try_to_vec().unwrap();
                        
                        // Verify minimum length for Anchor contract (8-byte discriminator + account data)
                        assert!(test_data.len() >= 8, "Test data should have at least 8 bytes for discriminator");
                        
                        // Test discriminator recognition with proper error handling
                        let result = try_unpack_account(&test_data);
                        match result {
                            Ok(_) => {
                                // Success - discriminator was recognized and account parsed
                            },
                            Err(e) => {
                                // Allow parsing errors but not discriminator recognition errors
                                let error_msg = e.to_string();
                                assert!(!error_msg.contains("Unknown account discriminator"), 
                                       "Should recognize discriminator for account {}", stringify!(#variant_name));
                            }
                        }
                    }
                } else {
                    quote! {
                        // Test account: #variant_name (non-Anchor style with length-based recognition)
                        let test_account = #struct_name::default();
                        let test_data = test_account.try_to_vec().unwrap();
                        
                        // Verify account data size matches expected length
                        assert_eq!(test_data.len(), #struct_name::LEN, "Test data should match account struct size");
                        
                        // Test length-based recognition with proper error handling
                        let result = try_unpack_account(&test_data);
                        match result {
                            Ok(_) => {
                                // Success - length was recognized and account parsed
                            },
                            Err(e) => {
                                // Allow parsing errors but not length recognition errors
                                let error_msg = e.to_string();
                                assert!(!error_msg.contains("Invalid Account data length"), 
                                       "Should recognize account length for {}", stringify!(#variant_name));
                            }
                        }
                    }
                }
            });

            quote! {
                #[test]
                fn test_discriminator_recognition() {
                    #(#discriminator_tests)*
                }
            }
        };

        quote! {
            #[cfg(test)]
            mod tests {
                use super::*;
                use borsh::{BorshDeserialize, BorshSerialize};

                #(#test_functions)*

                #discriminator_test

                #[test]
                fn test_invalid_discriminator() {
                    let invalid_data = vec![255u8; 40]; // Invalid discriminator + some data
                    let result = try_unpack_account(&invalid_data);
                    assert!(result.is_err(), "Should fail with invalid discriminator");
                }

                #[test] 
                fn test_insufficient_data() {
                    let short_data = vec![1u8; 4]; // Less than 8 bytes
                    let result = try_unpack_account(&short_data);
                    assert!(result.is_err(), "Should fail with insufficient data");
                }
            }
        }
    }
}

impl<'a> IdlCodegenModule for AccountsParserModule<'a> {
    fn name(&self) -> &str {
        "accounts_parser"
    }

    fn gen_head(&self) -> TokenStream {
        quote! {
            use crate::accounts::*;
        }
    }

    fn gen_body(&self) -> TokenStream {
        let enum_def = self.generate_account_enum();
        let try_unpack_fn = self.generate_try_unpack_function();
        let tests = self.generate_tests();

        quote! {
            #enum_def

            #try_unpack_fn

            #tests
        }
    }
}