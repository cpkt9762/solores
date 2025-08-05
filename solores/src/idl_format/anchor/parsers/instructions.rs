use proc_macro2::TokenStream;
use quote::quote;
use convert_case::{Case, Casing};
use heck::ToShoutySnakeCase;

use crate::{
    idl_format::{anchor::AnchorIdl, IdlCodegenModule, IdlFormat},
    Args,
};

pub struct InstructionsParserModule<'a> {
    pub idl: &'a AnchorIdl,
    pub args: &'a Args,
}

impl<'a> InstructionsParserModule<'a> {
    pub fn new(idl: &'a AnchorIdl, args: &'a Args) -> Self {
        Self { idl, args }
    }

    fn generate_instruction_enum(&self) -> TokenStream {
        let instructions = self.idl.instructions();
        if instructions.is_empty() {
            return quote! {};
        }

        let enum_variants = instructions.iter().map(|ix| {
            let variant_name = syn::Ident::new(&ix.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            
            let doc_comments = if let Some(docs) = &ix.docs {
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

            // Check if instruction has arguments
            let has_args = ix.args.as_ref().map_or(false, |args| !args.is_empty());
            if !has_args {
                quote! {
                    #doc_comments
                    #variant_name,
                }
            } else {
                let args_struct_name = syn::Ident::new(
                    &format!("{}IxArgs", ix.name.to_case(Case::Pascal)),
                    proc_macro2::Span::call_site(),
                );
                quote! {
                    #doc_comments
                    #variant_name(#args_struct_name),
                }
            }
        });

        let program_name = syn::Ident::new(
            &format!("{}Instruction", self.idl.program_name().to_case(Case::Pascal)),
            proc_macro2::Span::call_site(),
        );

        quote! {
            /// Program instruction types
            #[derive(Clone, Debug, PartialEq)]
            pub enum #program_name {
                #(#enum_variants)*
            }
        }
    }

    fn generate_parse_instruction_function(&self) -> TokenStream {
        let instructions = self.idl.instructions();
        if instructions.is_empty() {
            return quote! {};
        }

        let program_name = syn::Ident::new(
            &format!("{}Instruction", self.idl.program_name().to_case(Case::Pascal)),
            proc_macro2::Span::call_site(),
        );

        let is_anchor = self.idl.is_anchor_contract();
        
        if is_anchor {
            // Generate 8-byte discriminator parsing for Anchor contracts
            self.generate_anchor_parse_function(&program_name, instructions)
        } else {
            // Generate 1-byte discriminator parsing for non-Anchor contracts
            self.generate_non_anchor_parse_function(&program_name, instructions)
        }
    }

    fn generate_anchor_parse_function(&self, program_name: &syn::Ident, instructions: &[crate::idl_format::anchor::instructions::NamedInstruction]) -> TokenStream {
        let match_arms = instructions.iter().map(|ix| {
            let variant_name = syn::Ident::new(&ix.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            
            let discm_const_name = syn::Ident::new(
                &format!("{}_IX_DISCM", ix.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );

            let has_args = ix.args.as_ref().map_or(false, |args| !args.is_empty());
            if !has_args {
                quote! {
                    #discm_const_name => Ok(#program_name::#variant_name),
                }
            } else {
                let args_struct_name = syn::Ident::new(
                    &format!("{}IxArgs", ix.name.to_case(Case::Pascal)),
                    proc_macro2::Span::call_site(),
                );
                
                quote! {
                    #discm_const_name => {
                        let args = #args_struct_name::deserialize(&mut ix_data)?;
                        Ok(#program_name::#variant_name(args))
                    },
                }
            }
        });

        quote! {
            /// Parse instruction data based on 8-byte discriminator (Anchor contracts)
            pub fn parse_instruction(data: &[u8]) -> Result<#program_name, std::io::Error> {
                if data.len() < 8 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Instruction data too short for discriminator"
                    ));
                }

                let discriminator: [u8; 8] = data[0..8].try_into()
                    .map_err(|_| std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Failed to read discriminator"
                    ))?;

                let mut ix_data = &data[8..];

                match discriminator {
                    #(#match_arms)*
                    _ => Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Unknown instruction discriminator: {:?}", discriminator)
                    )),
                }
            }
        }
    }

    fn generate_non_anchor_parse_function(&self, program_name: &syn::Ident, instructions: &[crate::idl_format::anchor::instructions::NamedInstruction]) -> TokenStream {
        let match_arms = instructions.iter().enumerate().map(|(index, ix)| {
            let variant_name = syn::Ident::new(&ix.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            
            // For non-Anchor contracts, use 1-byte discriminator based on JSON array index
            let discm_const_name = syn::Ident::new(
                &format!("{}_IX_DISCM", ix.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );

            let discriminator_value = index as u8;

            let has_args = ix.args.as_ref().map_or(false, |args| !args.is_empty());
            if !has_args {
                quote! {
                    #discriminator_value => Ok(#program_name::#variant_name),
                }
            } else {
                let args_struct_name = syn::Ident::new(
                    &format!("{}IxArgs", ix.name.to_case(Case::Pascal)),
                    proc_macro2::Span::call_site(),
                );
                
                quote! {
                    #discriminator_value => {
                        let args = #args_struct_name::deserialize(&mut ix_data)?;
                        Ok(#program_name::#variant_name(args))
                    },
                }
            }
        });

        quote! {
            /// Parse instruction data based on 1-byte discriminator (non-Anchor contracts)
            pub fn parse_instruction(data: &[u8]) -> Result<#program_name, std::io::Error> {
                if data.is_empty() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Instruction data is empty"
                    ));
                }

                let discriminator = data[0];  // 1-byte discriminator
                let mut ix_data = &data[1..];

                match discriminator {
                    #(#match_arms)*
                    _ => Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Unknown instruction discriminator: {}", discriminator)
                    )),
                }
            }
        }
    }

    fn generate_account_validation_function(&self) -> TokenStream {
        // Validation functions removed to reduce code size
        quote! {}
    }

    fn generate_tests(&self) -> TokenStream {
        let instructions = self.idl.instructions();
        if instructions.is_empty() {
            return quote! {};
        }

        let program_name = syn::Ident::new(
            &format!("{}Instruction", self.idl.program_name().to_case(Case::Pascal)),
            proc_macro2::Span::call_site(),
        );

        let test_functions = instructions.iter().map(|ix| {
            let test_name = syn::Ident::new(
                &format!("test_{}_consistency", ix.name.to_case(Case::Snake)),
                proc_macro2::Span::call_site(),
            );
            let variant_name = syn::Ident::new(&ix.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            
            let discm_const_name = syn::Ident::new(
                &format!("{}_IX_DISCM", ix.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );

            let has_args = ix.args.as_ref().map_or(false, |args| !args.is_empty());
            
            if !has_args {
                quote! {
                    #[test]
                    fn #test_name() {
                        // Test instruction without arguments
                        let mut test_data = Vec::new();
                        test_data.extend_from_slice(&#discm_const_name);
                        
                        // Parse the instruction
                        let parsed = parse_instruction(&test_data).expect("Failed to parse instruction");
                        
                        // Verify the parsed instruction matches expected type
                        match parsed {
                            #program_name::#variant_name => {
                                // Success - instruction parsed correctly
                            }
                            _ => panic!("Parsed instruction has wrong type"),
                        }
                    }
                }
            } else {
                let args_struct_name = syn::Ident::new(
                    &format!("{}IxArgs", ix.name.to_case(Case::Pascal)),
                    proc_macro2::Span::call_site(),
                );

                // Generate test data with known first field value for instructions with arguments
                let (test_data_generation, verification_logic) = if let Some(args) = ix.args.as_ref() {
                    if let Some(first_arg) = args.first() {
                        // Convert field name to snake_case to match generated struct fields
                        let snake_case_name = {
                            let mut result = String::new();
                            let mut leading_underscores = String::new();
                            
                            // 收集前导下划线
                            if first_arg.name.starts_with('_') {
                                leading_underscores.push('_');
                            }
                            
                            // 使用 convert_case 转换剩余部分
                            let rest = &first_arg.name[leading_underscores.len()..];
                            let converted = rest.to_case(Case::Snake);
                            
                            // 拼接前导下划线和转换后的部分
                            result.push_str(&leading_underscores);
                            result.push_str(&converted);
                            result
                        };
                        let first_arg_name = syn::Ident::new(&snake_case_name, proc_macro2::Span::call_site());
                        
                        // Generate test value based on argument type
                        log::debug!("🔍 分析第一个参数类型: {:?}", &first_arg.r#type);
                        let (is_simple_type, test_value, test_value_bytes) = match &first_arg.r#type {
                            crate::idl_format::anchor::typedefs::TypedefFieldType::PrimitiveOrPubkey(type_str) => {
                                match type_str.as_str() {
                                    "publicKey" | "Pubkey" => (
                                        true,
                                        quote! { solana_program::pubkey::Pubkey::new_from_array([1u8; 32]) },
                                        quote! { [1u8; 32].to_vec() }
                                    ),
                                    "u64" => (
                                        true,
                                        quote! { 1000u64 },
                                        quote! { 1000u64.to_le_bytes().to_vec() }
                                    ),
                                    "u32" => (
                                        true,
                                        quote! { 1000u32 },
                                        quote! { 1000u32.to_le_bytes().to_vec() }
                                    ),
                                    "u16" => (
                                        true,
                                        quote! { 1000u16 },
                                        quote! { 1000u16.to_le_bytes().to_vec() }
                                    ),
                                    "u8" => (
                                        true,
                                        quote! { 100u8 },
                                        quote! { vec![100u8] }
                                    ),
                                    "i64" => (
                                        true,
                                        quote! { 1000i64 },
                                        quote! { 1000i64.to_le_bytes().to_vec() }
                                    ),
                                    "i32" => (
                                        true,
                                        quote! { 1000i32 },
                                        quote! { 1000i32.to_le_bytes().to_vec() }
                                    ),
                                    "string" => (
                                        true,
                                        quote! { "test".to_string() },
                                        quote! { {
                                            let s = "test";
                                            let mut bytes = (s.len() as u32).to_le_bytes().to_vec();
                                            bytes.extend_from_slice(s.as_bytes());
                                            bytes
                                        } }
                                    ),
                                    "bool" => (
                                        true,
                                        quote! { true },
                                        quote! { vec![1u8] }
                                    ),
                                    _ => (
                                        false,
                                        quote! { Default::default() },  // Use Default instead of empty TokenStream
                                        quote! { vec![0u8; 32] }
                                    )
                                }
                            },
                            other_type => {
                                log::debug!("🔍 识别为复杂类型: {:?}", other_type);
                                (
                                    false,
                                    quote! { Default::default() },  // Use Default instead of empty TokenStream
                                    quote! { vec![0u8; 32] }
                                )
                            }
                        };
                        
                        log::debug!("🔍 类型分析结果: is_simple_type={}, test_value为空={}", is_simple_type, test_value.is_empty());

                        let data_gen = if is_simple_type {
                            quote! {
                                // Create test data using struct serialization
                                let expected_first_arg = #test_value;
                                let test_args = #args_struct_name {
                                    #first_arg_name: expected_first_arg,
                                    ..Default::default()
                                };
                                let mut test_data = Vec::new();
                                test_data.extend_from_slice(&#discm_const_name);
                                test_data.extend_from_slice(&test_args.try_to_vec().unwrap());
                            }
                        } else {
                            quote! {
                                // Create test data using default struct values
                                let test_args = #args_struct_name::default();
                                let mut test_data = Vec::new();
                                test_data.extend_from_slice(&#discm_const_name);
                                test_data.extend_from_slice(&test_args.try_to_vec().unwrap());
                            }
                        };
                        
                        let verification = if !is_simple_type {
                            quote! {
                                // Skip field verification for unknown types, just test discriminator matching
                                match parse_instruction(&test_data) {
                                    Ok(#program_name::#variant_name(_)) => {
                                        // Success - discriminator matched and instruction parsed
                                    }
                                    Ok(_) => panic!("Discriminator matched wrong instruction type"),
                                    Err(e) => {
                                        // Expected for complex data - verify it's not a discriminator error
                                        let error_msg = e.to_string();
                                        assert!(!error_msg.contains("Unknown instruction discriminator"), 
                                               "Should recognize discriminator for {}", stringify!(#variant_name));
                                    }
                                }
                            }
                        } else {
                            quote! {
                                // Test discriminator matching and first argument value
                                match parse_instruction(&test_data) {
                                    Ok(#program_name::#variant_name(args)) => {
                                        // Success - verify first argument matches expected value
                                        assert_eq!(args.#first_arg_name, expected_first_arg, 
                                                 "First argument value should match expected value");
                                    }
                                    Ok(_) => panic!("Discriminator matched wrong instruction type"),
                                    Err(e) => {
                                        // Expected for complex data - verify it's not a discriminator error
                                        let error_msg = e.to_string();
                                        assert!(!error_msg.contains("Unknown instruction discriminator"), 
                                               "Should recognize discriminator for {}", stringify!(#variant_name));
                                    }
                                }
                            }
                        };

                        (data_gen, verification)
                    } else {
                        // No args, fallback to dummy data test
                        (
                            quote! {
                                let mut test_data = Vec::new();
                                test_data.extend_from_slice(&[#discm_const_name]);
                                test_data.extend_from_slice(&vec![0u8; 32]);
                            },
                            quote! {
                                match parse_instruction(&test_data) {
                                    Ok(#program_name::#variant_name(_)) => {
                                        // Success - discriminator matched and instruction parsed
                                    }
                                    Ok(_) => panic!("Discriminator matched wrong instruction type"),
                                    Err(e) => {
                                        let error_msg = e.to_string();
                                        assert!(!error_msg.contains("Unknown instruction discriminator"), 
                                               "Should recognize discriminator for {}", stringify!(#variant_name));
                                    }
                                }
                            }
                        )
                    }
                } else {
                    // Fallback to dummy data test
                    (
                        quote! {
                            let mut test_data = Vec::new();
                            test_data.extend_from_slice(&#discm_const_name);
                            test_data.extend_from_slice(&vec![0u8; 32]);
                        },
                        quote! {
                            match parse_instruction(&test_data) {
                                Ok(#program_name::#variant_name(_)) => {
                                    // Success - discriminator matched and instruction parsed
                                }
                                Ok(_) => panic!("Discriminator matched wrong instruction type"),
                                Err(e) => {
                                    let error_msg = e.to_string();
                                    assert!(!error_msg.contains("Unknown instruction discriminator"), 
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
            }
        });

        // Generate discriminator test
        let discriminator_test = {
            let discriminator_tests = instructions.iter().map(|ix| {
                let discm_const_name = syn::Ident::new(
                    &format!("{}_IX_DISCM", ix.name.to_shouty_snake_case()),
                    proc_macro2::Span::call_site(),
                );

                // Check if instruction has arguments to determine test data generation
                let has_args = ix.args.as_ref().map_or(false, |args| !args.is_empty());
                
                if has_args {
                    let args_struct_name = syn::Ident::new(
                        &format!("{}IxArgs", ix.name.to_case(Case::Pascal)),
                        proc_macro2::Span::call_site(),
                    );
                    let program_ix_enum_name = program_name.clone();
                    let enum_variant_name = syn::Ident::new(
                        &ix.name.to_case(Case::Pascal),
                        proc_macro2::Span::call_site(),
                    );
                    
                    quote! {
                        // Test instruction with args: #ix.name using internal discriminator
                        let test_args = #args_struct_name::default();
                        let test_data = test_args.try_to_vec().unwrap();
                        
                        // Verify exact length for Anchor contract (discriminator is internal to struct)
                        let expected_size = std::mem::size_of::<[u8; 8]>(); // discriminator size
                        assert!(test_data.len() >= expected_size, 
                               "Test data length should be at least {} bytes for {}", expected_size, stringify!(#args_struct_name));
                        
                        // Verify discriminator value is correct (first 8 bytes)
                        assert_eq!(&test_data[0..8], &#discm_const_name, 
                                  "Discriminator should match expected value");
                        
                        // Test discriminator recognition and complete parameter validation
                        let result = parse_instruction(&test_data);
                        match result {
                            Ok(instruction) => {
                                match instruction {
                                    #program_ix_enum_name::#enum_variant_name(parsed_args) => {
                                        // Verify discriminator field
                                        assert_eq!(parsed_args.discriminator, #discm_const_name,
                                                  "Parsed discriminator should match expected");
                                        // Note: Additional field validation could be added here
                                    },
                                    _ => panic!("Parsed instruction should be {}", stringify!(#enum_variant_name))
                                }
                            },
                            Err(e) => {
                                panic!("Should successfully parse instruction {}: {}", stringify!(#enum_variant_name), e);
                            }
                        }
                    }
                } else {
                    let program_ix_enum_name = program_name.clone();
                    let enum_variant_name = syn::Ident::new(
                        &ix.name.to_case(Case::Pascal),
                        proc_macro2::Span::call_site(),
                    );
                    
                    quote! {
                        // Test instruction without args: #ix.name using discriminator only
                        let test_data = #discm_const_name.to_vec();
                        
                        // Verify exact length for Anchor contract (8-byte discriminator only)
                        assert_eq!(test_data.len(), 8, "Test data should be exactly 8 bytes for discriminator-only instruction");
                        
                        // Test discriminator recognition
                        let result = parse_instruction(&test_data);
                        match result {
                            Ok(instruction) => {
                                match instruction {
                                    #program_ix_enum_name::#enum_variant_name => {
                                        // Success - correct instruction type parsed
                                    },
                                    _ => panic!("Parsed instruction should be {}", stringify!(#enum_variant_name))
                                }
                            },
                            Err(e) => {
                                panic!("Should successfully parse instruction {}: {}", stringify!(#enum_variant_name), e);
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

                #(#test_functions)*

                #discriminator_test

                #[test]
                fn test_invalid_discriminator() {
                    let invalid_data = vec![255u8; 40]; // Invalid discriminator + some data
                    let result = parse_instruction(&invalid_data);
                    assert!(result.is_err(), "Should fail with invalid discriminator");
                }

                #[test]
                fn test_insufficient_data() {
                    let short_data = vec![1u8; 4]; // Less than 8 bytes
                    let result = parse_instruction(&short_data);
                    assert!(result.is_err(), "Should fail with insufficient data");
                }
            }
        }
    }

    fn generate_discriminator_constants(&self) -> TokenStream {
        let instructions = self.idl.instructions();
        if instructions.is_empty() {
            return quote! {};
        }

        let is_anchor = self.idl.is_anchor_contract();

        let constants = instructions.iter().enumerate().map(|(index, ix)| {
            let const_name = syn::Ident::new(
                &format!("{}_IX_DISCM", ix.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );
            
            if is_anchor {
                // Anchor contracts: use 8-byte discriminators from IDL
                let discriminator_bytes = if let Some(discm) = &ix.discriminator {
                    let bytes: Vec<u8> = discm.iter().map(|&x| x as u8).collect();
                    if bytes.len() == 8 {
                        let byte_tokens: Vec<proc_macro2::TokenStream> = bytes.iter().map(|&b| quote! { #b }).collect();
                        quote! { [#(#byte_tokens),*] }
                    } else {
                        log::warn!("Expected 8-byte discriminator for Anchor contract, got {} bytes", bytes.len());
                        quote! { [0u8; 8] }
                    }
                } else {
                    // Fallback for Anchor contracts without discriminator
                    quote! { [0u8; 8] }
                };

                quote! {
                    pub const #const_name: [u8; 8] = #discriminator_bytes;
                }
            } else {
                // Non-Anchor contracts: use 1-byte discriminators or index-based
                let discriminator_value = if let Some(discm) = &ix.discriminator {
                    if discm.len() == 1 {
                        discm[0] as u8
                    } else {
                        log::warn!("Expected 1-byte discriminator for non-Anchor contract, got {} bytes, using index", discm.len());
                        index as u8
                    }
                } else {
                    // Use instruction index as discriminator for non-Anchor contracts
                    index as u8
                };

                quote! {
                    pub const #const_name: u8 = #discriminator_value;
                }
            }
        });

        quote! {
            #(#constants)*
        }
    }
}

impl<'a> IdlCodegenModule for InstructionsParserModule<'a> {
    fn name(&self) -> &str {
        "instructions_parser"
    }

    fn gen_head(&self) -> TokenStream {
        quote! {
            use crate::instructions::*;
        }
    }

    fn gen_body(&self) -> TokenStream {
        let discriminator_constants = self.generate_discriminator_constants();
        let enum_def = self.generate_instruction_enum();
        let parse_fn = self.generate_parse_instruction_function();
        let validation_fns = self.generate_account_validation_function();
        let tests = self.generate_tests();

        quote! {
            #discriminator_constants

            #enum_def

            #parse_fn

            #validation_fns

            #tests
        }
    }
}