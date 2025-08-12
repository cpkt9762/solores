//! 非 Anchor Instructions 模板
//!
//! 为非 Anchor 合约生成 Instructions 相关代码，使用 1 字节 discriminator

use proc_macro2::TokenStream;
use quote::quote;
use convert_case::{Case, Casing};
use heck::ToShoutySnakeCase;
use std::cell::RefCell;

use crate::idl_format::non_anchor_idl::NonAnchorIdl;
use crate::idl_format::non_anchor_idl::{NonAnchorType, NonAnchorTypeKind, NonAnchorFieldType};
use crate::Args;
use crate::templates::TemplateGenerator;
use crate::templates::common::{
    doc_generator::DocGenerator,
    import_manager::ImportManager,
    naming_converter::NamingConverter
};
use crate::utils::{to_snake_case_with_serde, generate_pubkey_serde_attr};

/// 非 Anchor Instructions 模板
pub struct NonAnchorInstructionsTemplate<'a> {
    pub idl: &'a NonAnchorIdl,
    pub args: &'a Args,
    pub named_types: &'a [NonAnchorType],
    naming_converter: RefCell<NamingConverter>,
}

impl<'a> NonAnchorInstructionsTemplate<'a> {
    /// 创建新的非 Anchor Instructions 模板
    pub fn new(idl: &'a NonAnchorIdl, args: &'a Args, named_types: &'a [NonAnchorType]) -> Self {
        Self { 
            idl, 
            args, 
            named_types,
            naming_converter: RefCell::new(NamingConverter::new()),
        }
    }

    /// 检查 typedef 字段类型是否为 Pubkey
    fn is_typedef_field_pubkey_type(field_type: &NonAnchorFieldType) -> bool {
        match field_type {
            NonAnchorFieldType::Basic(s) => {
                matches!(s.as_str(), "publicKey" | "pubkey" | "Pubkey")
            },
            _ => false
        }
    }

    /// 生成指令参数结构体
    pub fn generate_instruction_args(&self) -> TokenStream {
        let instructions = self.idl.instructions.as_deref().unwrap_or(&[]);
        if instructions.is_empty() {
            return quote! {};
        }

        let arg_structs = instructions.iter().filter_map(|instruction| {
            let empty_args = vec![];
            let args = instruction.args.as_ref().unwrap_or(&empty_args);
            if args.is_empty() {
                return None;
            }

            let mut naming_converter = self.naming_converter.borrow_mut();
            let struct_name_str = naming_converter.convert_instruction_struct_name(&instruction.name);
            log::debug!("🏷️ Non-Anchor Instruction '{}' -> struct name: '{}'", instruction.name, struct_name_str);
            let struct_name = syn::Ident::new(&struct_name_str, proc_macro2::Span::call_site());
            drop(naming_converter); // 释放借用
            
            let doc_comments = DocGenerator::generate_instruction_docs(&instruction.docs);
            
            // Try to find corresponding type definition in named_types
            let args_fields = if let Some(named_type) = self.named_types.iter()
                .find(|t| t.name == format!("{}Args", instruction.name.to_case(Case::Pascal)) || 
                         t.name == struct_name_str) {
                
                self.generate_fields_from_named_type(named_type)
            } else {
                // Fallback: generate from instruction.args directly
                self.generate_fields_from_instruction_args(args)
            };

            Some(quote! {
                #doc_comments
                #[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Clone, Debug, PartialEq)]
                pub struct #struct_name {
                    #args_fields
                }
            })
        });

        quote! {
            #(#arg_structs)*
        }
    }

    /// 从named_type生成字段
    fn generate_fields_from_named_type(&self, named_type: &NonAnchorType) -> TokenStream {
        if let NonAnchorTypeKind::Struct { fields } = &named_type.type_def {
            let field_tokens = fields.iter().map(|field| {
                let (snake_field_name, serde_attr) = to_snake_case_with_serde(&field.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                let field_type = Self::convert_typedef_field_type_to_rust(&field.field_type);
                let field_docs = DocGenerator::generate_field_docs(&field.docs);
                
                // 检查是否为 Pubkey 类型，如果是则添加特殊的 serde 属性
                let pubkey_serde_attr = if Self::is_typedef_field_pubkey_type(&field.field_type) {
                    generate_pubkey_serde_attr()
                } else {
                    quote! {}
                };
                
                quote! {
                    #field_docs
                    #serde_attr
                    #pubkey_serde_attr
                    pub #field_name: #field_type,
                }
            });
            quote! { #(#field_tokens)* }
        } else {
            quote! {}
        }
    }

    /// 从instruction.args生成字段
    fn generate_fields_from_instruction_args(&self, args: &[crate::idl_format::non_anchor_idl::NonAnchorField]) -> TokenStream {
        let fields = args.iter().map(|arg| {
            let (snake_field_name, serde_attr) = to_snake_case_with_serde(&arg.name);
            let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
            let field_type = Self::convert_typedef_field_type_to_rust(&arg.field_type);
            
            // 检查是否为 Pubkey 类型，如果是则添加特殊的 serde 属性
            let pubkey_serde_attr = if Self::is_typedef_field_pubkey_type(&arg.field_type) {
                generate_pubkey_serde_attr()
            } else {
                quote! {}
            };
            
            quote! {
                #serde_attr
                #pubkey_serde_attr
                pub #field_name: #field_type,
            }
        });
        quote! { #(#fields)* }
    }

    /// 转换 NonAnchorFieldType 为 Rust 类型
    fn convert_typedef_field_type_to_rust(field_type: &NonAnchorFieldType) -> TokenStream {
        match field_type {
            NonAnchorFieldType::Basic(type_str) => {
                let type_ident = match type_str.as_str() {
                    "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "u64" | "i64" | "u128" | "i128" => {
                        syn::Ident::new(type_str, proc_macro2::Span::call_site())
                    },
                    "bool" => syn::Ident::new("bool", proc_macro2::Span::call_site()),
                    "string" | "String" => syn::Ident::new("String", proc_macro2::Span::call_site()),
                    "publicKey" | "pubkey" | "Pubkey" => syn::Ident::new("Pubkey", proc_macro2::Span::call_site()),
                    "bytes" => return quote! { Vec<u8> }, // bytes类型映射为Vec<u8>
                    _ => syn::Ident::new(type_str, proc_macro2::Span::call_site()),
                };
                quote! { #type_ident }
            },
            NonAnchorFieldType::Option { option } => {
                let inner_type = Self::convert_typedef_field_type_to_rust(option);
                quote! { Option<#inner_type> }
            },
            NonAnchorFieldType::Vec { vec } => {
                let inner_type = Self::convert_typedef_field_type_to_rust(vec);
                quote! { Vec<#inner_type> }
            },
            NonAnchorFieldType::Array { array } => {
                let (inner_type, size) = array;
                let inner_type_token = Self::convert_typedef_field_type_to_rust(inner_type);
                let size_literal = proc_macro2::Literal::usize_unsuffixed(*size);
                quote! { [#inner_type_token; #size_literal] }
            },
            NonAnchorFieldType::Defined { defined } => {
                // 使用完整路径引用types模块中的类型
                let type_path = format!("crate::types::{}", defined);
                let type_path: syn::Path = syn::parse_str(&type_path).unwrap();
                quote! { #type_path }
            },
            NonAnchorFieldType::Complex { kind, params } => {
                // 处理复合类型，如 Vec<T>, Option<T>, [T; N] 等 (Legacy支持)
                match kind.as_str() {
                    "Vec" => {
                        if let Some(params) = params {
                            if let Some(inner_type) = params.get(0) {
                                if let Some(inner_str) = inner_type.as_str() {
                                    let inner_type_token = Self::convert_typedef_field_type_to_rust(&NonAnchorFieldType::Basic(inner_str.to_string()));
                                    quote! { Vec<#inner_type_token> }
                                } else {
                                    quote! { Vec<u8> } // fallback
                                }
                            } else {
                                quote! { Vec<u8> } // fallback
                            }
                        } else {
                            quote! { Vec<u8> } // fallback
                        }
                    },
                    "Option" => {
                        if let Some(params) = params {
                            if let Some(inner_type) = params.get(0) {
                                if let Some(inner_str) = inner_type.as_str() {
                                    let inner_type_token = Self::convert_typedef_field_type_to_rust(&NonAnchorFieldType::Basic(inner_str.to_string()));
                                    quote! { Option<#inner_type_token> }
                                } else {
                                    quote! { Option<u8> } // fallback
                                }
                            } else {
                                quote! { Option<u8> } // fallback
                            }
                        } else {
                            quote! { Option<u8> } // fallback
                        }
                    },
                    _ => {
                        let type_ident = syn::Ident::new(kind, proc_macro2::Span::call_site());
                        quote! { #type_ident }
                    }
                }
            },
        }
    }

    /// 生成指令账户结构体
    pub fn generate_instruction_accounts(&self) -> TokenStream {
        let instructions = self.idl.instructions.as_deref().unwrap_or(&[]);
        if instructions.is_empty() {
            return quote! {};
        }

        let account_structs = instructions.iter().map(|instruction| {
            let struct_name = syn::Ident::new(
                &format!("{}Accounts", instruction.name.to_case(Case::Pascal)),
                proc_macro2::Span::call_site(),
            );
            let keys_struct_name = syn::Ident::new(
                &format!("{}Keys", instruction.name.to_case(Case::Pascal)),
                proc_macro2::Span::call_site(),
            );
            
            let accounts_len_const = syn::Ident::new(
                &format!("{}_IX_ACCOUNTS_LEN", instruction.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );

            let empty_accounts = vec![];
            let accounts_raw = instruction.accounts.as_ref().unwrap_or(&empty_accounts);
            // Convert NonAnchorAccount to AnchorAccountConstraint for compatibility
            let accounts: Vec<crate::idl_format::anchor_idl::AnchorAccountConstraint> = accounts_raw.iter().map(|acc| {
                crate::idl_format::anchor_idl::AnchorAccountConstraint {
                    name: acc.name.clone(),
                    is_mut: false, // 默认值
                    is_signer: false, // 默认值
                    is_optional: None, // 默认值
                    docs: acc.docs.clone(),
                    constraints: None,
                }
            }).collect();
            
            // Generate account fields
            let account_fields = accounts.iter().map(|account| {
                let field_name = syn::Ident::new(&account.name.to_case(Case::Snake), proc_macro2::Span::call_site());
                let field_docs = DocGenerator::generate_instruction_account_docs(account);
                
                quote! {
                    #field_docs
                    pub #field_name: &'me AccountInfo<'info>,
                }
            });

            let key_fields = accounts.iter().map(|account| {
                let field_name = syn::Ident::new(&account.name.to_case(Case::Snake), proc_macro2::Span::call_site());
                let field_docs = DocGenerator::generate_instruction_account_docs(account);
                
                quote! {
                    #field_docs
                    pub #field_name: Pubkey,
                }
            });

            let accounts_len = accounts.len();

            quote! {
                pub const #accounts_len_const: usize = #accounts_len;

                /// Accounts struct for the #instruction.name instruction
                #[derive(Copy, Clone, Debug)]
                pub struct #struct_name<'me, 'info> {
                    #(#account_fields)*
                }

                /// Public keys struct for the #instruction.name instruction
                #[derive(Copy, Clone, Debug, PartialEq)]
                pub struct #keys_struct_name {
                    #(#key_fields)*
                }
            }
        });

        quote! {
            use solana_account_info::AccountInfo;
            use solana_pubkey::Pubkey;
            
            #(#account_structs)*
        }
    }

    /// 生成调用函数
    pub fn generate_invoke_functions(&self) -> TokenStream {
        let instructions = self.idl.instructions.as_deref().unwrap_or(&[]);
        if instructions.is_empty() {
            return quote! {};
        }

        let invoke_functions = instructions.iter().map(|instruction| {
            let mut naming_converter = self.naming_converter.borrow_mut();
            let func_name = syn::Ident::new(
                &format!("{}_invoke", instruction.name.to_case(Case::Snake)),
                proc_macro2::Span::call_site(),
            );
            let ix_func_name = syn::Ident::new(
                &format!("{}_ix", instruction.name.to_case(Case::Snake)),
                proc_macro2::Span::call_site(),
            );
            
            let accounts_struct = syn::Ident::new(
                &format!("{}Accounts", instruction.name.to_case(Case::Pascal)),
                proc_macro2::Span::call_site(),
            );
            let keys_struct = syn::Ident::new(
                &format!("{}Keys", instruction.name.to_case(Case::Pascal)),
                proc_macro2::Span::call_site(),
            );
            let args_struct_name_str = naming_converter.convert_instruction_struct_name(&instruction.name);
            let args_struct = syn::Ident::new(&args_struct_name_str, proc_macro2::Span::call_site());
            let discm_const = syn::Ident::new(
                &format!("{}_IX_DISCM", instruction.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );

            let has_args = !instruction.args.as_ref().unwrap_or(&vec![]).is_empty();
            let args_param = if has_args {
                quote! { args: #args_struct, }
            } else {
                quote! {}
            };
            let args_serialization = if has_args {
                quote! {
                    instruction_data.extend_from_slice(&borsh::to_vec(&args)?);
                }
            } else {
                quote! {}
            };

            quote! {
                pub fn #func_name(
                    accounts: #accounts_struct,
                    #args_param
                ) -> solana_program_entrypoint::ProgramResult {
                    let keys = accounts_to_keys(&accounts);
                    let ix = #ix_func_name(keys, #(args,)?)?;
                    solana_cpi::invoke(&ix, &accounts_to_account_infos(&accounts))
                }

                pub fn #ix_func_name(
                    keys: #keys_struct,
                    #args_param
                ) -> Result<solana_instruction::Instruction, std::io::Error> {
                    let mut instruction_data = Vec::new();
                    instruction_data.push(#discm_const);
                    #args_serialization
                    
                    let accounts = keys_to_account_metas(&keys);
                    Ok(solana_instruction::Instruction {
                        program_id: crate::ID,
                        accounts,
                        data: instruction_data,
                    })
                }
            }
        });

        quote! {
            // 使用绝对路径，不需要导入
            use solana_cpi::invoke;
            use solana_program_entrypoint::ProgramResult;
            use solana_pubkey::Pubkey;
            use solana_account_info::AccountInfo;
            
            #(#invoke_functions)*
        }
    }

    /// 生成验证函数  
    pub fn generate_verify_functions(&self) -> TokenStream {
        // Basic verification functions for non-Anchor contracts
        quote! {
            // TODO: Implement verify functions for non-Anchor contracts
            // These would validate account keys and privileges
        }
    }

    /// 生成 1 字节 discriminator 常量
    pub fn generate_discriminator_constants(&self) -> TokenStream {
        let instructions = self.idl.instructions.as_deref().unwrap_or(&[]);
        if instructions.is_empty() {
            return quote! {};
        }

        let constants = instructions.iter().enumerate().map(|(index, instruction)| {
            let const_name = syn::Ident::new(
                &format!("{}_IX_DISCM", instruction.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );
            
            // Use 1-byte discriminator (with fallback for missing discriminator)
            let discriminator_value = {
                let discriminator = instruction.get_discriminator_with_fallback(index);
                discriminator[0] as u8
            };

            quote! {
                pub const #const_name: u8 = #discriminator_value;
            }
        });

        quote! {
            #(#constants)*
        }
    }

    /// 为单个instruction生成完整的文件内容
    pub fn generate_single_instruction_file(&self, instruction: &crate::idl_format::non_anchor_idl::NonAnchorInstruction, discriminator: u8) -> TokenStream {
        let mut naming_converter = self.naming_converter.borrow_mut();
        let struct_name_str = naming_converter.convert_instruction_struct_name(&instruction.name);
        drop(naming_converter); // 释放借用
        
        log::debug!("🏷️ Non-Anchor Single Instruction '{}' -> struct name: '{}'", instruction.name, struct_name_str);
        let struct_name = syn::Ident::new(&struct_name_str, proc_macro2::Span::call_site());
        let const_name = syn::Ident::new(
            &format!("{}_IX_DISCM", instruction.name.to_shouty_snake_case()),
            proc_macro2::Span::call_site(),
        );
        let accounts_len_const = syn::Ident::new(
            &format!("{}_IX_ACCOUNTS_LEN", instruction.name.to_shouty_snake_case()),
            proc_macro2::Span::call_site(),
        );

        let doc_comments = DocGenerator::generate_instruction_docs(&instruction.docs);
        let instruction_name_str = &instruction.name;

        // 生成指令参数字段
        let args_fields = if let Some(args) = &instruction.args {
            let fields: Vec<_> = args.iter().map(|arg| {
                let (snake_field_name, serde_attr) = to_snake_case_with_serde(&arg.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                let field_type = self.convert_field_type_to_rust(&arg.field_type);
                
                // 检查是否为 Pubkey 类型，如果是则添加特殊的 serde 属性
                let pubkey_serde_attr = if Self::is_typedef_field_pubkey_type(&arg.field_type) {
                    generate_pubkey_serde_attr()
                } else {
                    quote! {}
                };
                
                quote! { 
                    #serde_attr
                    #pubkey_serde_attr
                    pub #field_name: #field_type, 
                }
            }).collect();
            quote! {
                pub discriminator: u8,
                #(#fields)*
            }
        } else {
            quote! {
                pub discriminator: u8,
            }
        };

        // 生成默认实现
        let default_fields = if let Some(args) = &instruction.args {
            let default_values: Vec<_> = args.iter().map(|arg| {
                let (snake_field_name, _) = to_snake_case_with_serde(&arg.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                quote! { #field_name: Default::default(), }
            }).collect();
            quote! {
                discriminator: #const_name,
                #(#default_values)*
            }
        } else {
            quote! {
                discriminator: #const_name,
            }
        };

        // 生成 new() 方法的参数和赋值
        let (new_method_args, new_method_assignments) = if let Some(args) = &instruction.args {
            let arg_params = args.iter().map(|arg| {
                let (snake_field_name, _) = to_snake_case_with_serde(&arg.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                let field_type = self.convert_field_type_to_rust(&arg.field_type);
                quote! { #field_name: #field_type }
            });
            let arg_assignments = args.iter().map(|arg| {
                let (snake_field_name, _) = to_snake_case_with_serde(&arg.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                quote! { #field_name, }
            });
            (
                quote! { #(#arg_params),* },
                quote! { #(#arg_assignments)* }
            )
        } else {
            (quote! {}, quote! {})
        };

        // 计算账户长度和生成账户字段
        let accounts_len = instruction.accounts.as_ref().map_or(0, |accounts| accounts.len());
        let accounts_len_literal = proc_macro2::Literal::usize_unsuffixed(accounts_len);
        
        // 生成Keys结构体字段
        let keys_fields = if let Some(accounts) = &instruction.accounts {
            accounts.iter().map(|account| {
                let field_name = syn::Ident::new(&account.name.to_case(Case::Snake), proc_macro2::Span::call_site());
                quote! { pub #field_name: Pubkey, }
            }).collect()
        } else {
            vec![]
        };
        
        // 生成Keys Default实现字段
        let keys_default_fields = if let Some(accounts) = &instruction.accounts {
            accounts.iter().map(|account| {
                let field_name = syn::Ident::new(&account.name.to_case(Case::Snake), proc_macro2::Span::call_site());
                quote! { #field_name: Pubkey::default(), }
            }).collect()
        } else {
            vec![]
        };
        
        // 生成From [Pubkey] 实现字段
        let keys_from_array_fields = if let Some(accounts) = &instruction.accounts {
            accounts.iter().enumerate().map(|(i, account)| {
                let field_name = syn::Ident::new(&account.name.to_case(Case::Snake), proc_macro2::Span::call_site());
                let index_literal = proc_macro2::Literal::usize_unsuffixed(i);
                quote! { #field_name: pubkeys[#index_literal], }
            }).collect()
        } else {
            vec![]
        };
        
        // 生成Into [AccountMeta] 实现字段
        let keys_into_metas_fields = if let Some(accounts) = &instruction.accounts {
            accounts.iter().map(|account| {
                let field_name = syn::Ident::new(&account.name.to_case(Case::Snake), proc_macro2::Span::call_site());
                quote! { solana_instruction::AccountMeta::new(keys.#field_name, false), }
            }).collect()
        } else {
            vec![]
        };

        // 生成客户端函数
        let ix_fn_name = syn::Ident::new(
            &format!("{}_ix", instruction.name.to_case(Case::Snake)),
            proc_macro2::Span::call_site(),
        );
        let ix_with_program_id_fn_name = syn::Ident::new(
            &format!("{}_ix_with_program_id", instruction.name.to_case(Case::Snake)),
            proc_macro2::Span::call_site(),
        );
        let keys_struct_name = syn::Ident::new(
            &format!("{}Keys", instruction.name.to_case(Case::Pascal)),
            proc_macro2::Span::call_site(),
        );

        let has_accounts = instruction.accounts.as_ref().map_or(false, |accounts| !accounts.is_empty());
        let has_args = instruction.args.as_ref().map_or(false, |args| !args.is_empty());

        let fn_params = if has_accounts && has_args {
            quote! { keys: #keys_struct_name, args: #struct_name }
        } else if has_accounts {
            quote! { keys: #keys_struct_name }
        } else if has_args {
            quote! { args: #struct_name }
        } else {
            quote! {}
        };

        let fn_args = if has_accounts && has_args {
            quote! { keys, args }
        } else if has_accounts {
            quote! { keys }
        } else if has_args {
            quote! { args }
        } else {
            quote! {}
        };

        let (fn_body, accounts_expr) = if has_accounts {
            (
                quote! {
                    let metas: [solana_instruction::AccountMeta; #accounts_len_const] = keys.into();
                },
                quote! { Vec::from(metas) },
            )
        } else {
            (
                quote! {},
                quote! { Vec::new() },
            )
        };

        // Generate data expression based on whether instruction has args
        let data_expr = if has_args {
            quote! { args.try_to_vec()? }
        } else {
            quote! { #struct_name::default().try_to_vec()? }
        };

        let doc_string = format!("Instruction: {} (NonAnchor)", instruction.name);
        
        // 生成完整的代码内容以分析所需的导入
        let full_code_for_analysis = quote! {
            // Constants
            pub const #const_name: u8 = #discriminator;
            pub const #accounts_len_const: usize = #accounts_len_literal;
            
            // Instruction Data Structure
            #[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Clone, Debug, PartialEq)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            pub struct #struct_name {
                #args_fields
            }

            impl Default for #struct_name {
                fn default() -> Self {
                    Self {
                        #default_fields
                    }
                }
            }
            
            
            // Client functions
            pub fn #ix_fn_name(#fn_params) -> Result<solana_instruction::Instruction, std::io::Error> {
                #fn_body
                let data = #data_expr;
                Ok(solana_instruction::Instruction {
                    program_id: crate::ID,
                    accounts: #accounts_expr,
                    data,
                })
            }

            pub fn #ix_with_program_id_fn_name(program_id: Pubkey, #fn_params) -> Result<solana_instruction::Instruction, std::io::Error> {
                #fn_body
                let data = #data_expr;
                Ok(solana_instruction::Instruction {
                    program_id,
                    accounts: #accounts_expr,
                    data,
                })
            }
        }.to_string();
        
        // 检查IDL是否有types字段
        let has_types_module = self.idl.types.as_ref().map_or(false, |types| !types.is_empty());
        
        // 基于代码内容和types存在性生成智能导入
        let optimized_imports = ImportManager::generate_optimized_instruction_imports_for_code_with_types_check(
            &full_code_for_analysis, 
            has_types_module
        );
        
        quote! {
            #![doc = #doc_string]
            #doc_comments
            
            #optimized_imports
            
            // Constants
            pub const #const_name: u8 = #discriminator;
            pub const #accounts_len_const: usize = #accounts_len_literal;
            
            // Instruction Data Structure
            #doc_comments
            #[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Clone, Debug, PartialEq)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            pub struct #struct_name {
                #args_fields
            }

            impl Default for #struct_name {
                fn default() -> Self {
                    Self {
                        #default_fields
                    }
                }
            }

            impl #struct_name {
                pub fn new(#new_method_args) -> Self {
                    Self {
                        discriminator: Self::discriminator(),
                        #new_method_assignments
                    }
                }
                
                pub fn from_bytes(buf: &[u8]) -> std::io::Result<Self> {
                    borsh::BorshDeserialize::deserialize(&mut &buf[..])
                }
                
                pub fn discriminator() -> u8 {
                    #const_name
                }
                
                pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
                    borsh::to_vec(self)
                }
            }

            // Keys Structure for accounts
            #[derive(Copy, Clone, Debug, PartialEq)]
            pub struct #keys_struct_name {
                #(#keys_fields)*
            }

            impl Default for #keys_struct_name {
                fn default() -> Self {
                    Self {
                        #(#keys_default_fields)*
                    }
                }
            }

            impl From<&[Pubkey]> for #keys_struct_name {
                fn from(pubkeys: &[Pubkey]) -> Self {
                    Self {
                        #(#keys_from_array_fields)*
                    }
                }
            }

            impl From<#keys_struct_name> for [solana_instruction::AccountMeta; #accounts_len_const] {
                fn from(keys: #keys_struct_name) -> Self {
                    [
                        #(#keys_into_metas_fields)*
                    ]
                }
            }

            // Client Functions
            pub fn #ix_with_program_id_fn_name(
                program_id: Pubkey, 
                #fn_params
            ) -> Result<solana_instruction::Instruction, std::io::Error> {
                #fn_body
                Ok(solana_instruction::Instruction {
                    program_id,
                    accounts: #accounts_expr,
                    data: #data_expr,
                })
            }

            pub fn #ix_fn_name(#fn_params) -> Result<solana_instruction::Instruction, std::io::Error> {
                #ix_with_program_id_fn_name(crate::ID, #fn_args)
            }
        }
    }

    /// 转换字段类型为Rust类型
    fn convert_field_type_to_rust(&self, field_type: &crate::idl_format::non_anchor_idl::NonAnchorFieldType) -> TokenStream {
        Self::convert_typedef_field_type_to_rust(field_type)
    }
}

impl<'a> TemplateGenerator for NonAnchorInstructionsTemplate<'a> {
    fn get_standard_module_name(&self) -> &'static str {
        "instructions"
    }

    fn gen_files(&self) -> Vec<(String, TokenStream)> {
        let instructions = self.idl.instructions.as_deref().unwrap_or(&[]);
        if instructions.is_empty() {
            return vec![("mod.rs".to_string(), quote! {
                //! Empty instructions module - no instructions found in IDL
            })];
        }
        
        let mut files = Vec::new();
        
        // 为每个instruction生成独立文件
        for (index, instruction) in instructions.iter().enumerate() {
            let file_name = format!("{}.rs", instruction.name.to_case(Case::Snake));
            let file_content = self.generate_single_instruction_file(instruction, index as u8);
            files.push((file_name, file_content));
        }
        
        files
    }

    fn gen_mod_file(&self) -> TokenStream {
        let instructions = self.idl.instructions.as_deref().unwrap_or(&[]);
        if instructions.is_empty() {
            return quote! {
                //! Instructions module - no instructions found in IDL
            };
        }
        
        // 生成模块声明和精确重导出
        let mut naming_converter = self.naming_converter.borrow_mut();
        let module_declarations_and_exports: Vec<TokenStream> = instructions.iter().map(|ix| {
            let module_name_str = naming_converter.convert_function_name(&ix.name);
            let module_name = syn::Ident::new(&module_name_str, proc_macro2::Span::call_site());
            
            // 生成精确重导出，避免使用glob导入
            let ix_data_struct = naming_converter.convert_instruction_struct_name(&ix.name);
            let keys_struct = naming_converter.convert_struct_name(&format!("{}Keys", &ix.name));
            let discm_const = format!("{}_IX_DISCM", ix.name.to_shouty_snake_case());
            let accounts_len_const = format!("{}_IX_ACCOUNTS_LEN", ix.name.to_shouty_snake_case());
            let ix_fn = naming_converter.convert_function_name(&format!("{}_ix", &ix.name));
            let ix_with_program_id_fn = naming_converter.convert_function_name(&format!("{}_ix_with_program_id", &ix.name));
            
            let ix_data_ident = syn::Ident::new(&ix_data_struct, proc_macro2::Span::call_site());
            let keys_ident = syn::Ident::new(&keys_struct, proc_macro2::Span::call_site());
            let discm_const_ident = syn::Ident::new(&discm_const, proc_macro2::Span::call_site());
            let accounts_len_const_ident = syn::Ident::new(&accounts_len_const, proc_macro2::Span::call_site());
            let ix_fn_ident = syn::Ident::new(&ix_fn, proc_macro2::Span::call_site());
            let ix_with_program_id_fn_ident = syn::Ident::new(&ix_with_program_id_fn, proc_macro2::Span::call_site());
            
            quote! {
                pub mod #module_name;
                pub use #module_name::{
                    #ix_data_ident,
                    #keys_ident,
                    #discm_const_ident,
                    #accounts_len_const_ident,
                    #ix_fn_ident,
                    #ix_with_program_id_fn_ident
                };
            }
        }).collect();
        
        quote! {
            //! Non-Anchor instructions module
            //! Generated instruction definitions with 1-byte discriminator support
            //! Each instruction is defined in its own file
            //!
            //! This module provides precise imports to avoid naming conflicts
            //! and improve compile times by avoiding glob imports.
            
            #(#module_declarations_and_exports)*
        }
    }
}