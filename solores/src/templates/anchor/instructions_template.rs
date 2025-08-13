//! Anchor Instructions 模板
//!
//! 为 Anchor 合约生成 Instructions 相关代码，使用 8 字节 discriminator

use proc_macro2::TokenStream;
use quote::quote;
use convert_case::{Case, Casing};
use heck::ToShoutySnakeCase;
use std::cell::RefCell;

use crate::idl_format::anchor_idl::{AnchorIdl, AnchorInstruction, AnchorFieldType};
use crate::Args;
use crate::templates::{TemplateGenerator, ContractModeTemplate};
use crate::templates::common::{
    doc_generator::DocGenerator, 
    import_manager::{ImportManager, ImportType, SolanaImport},
    naming_converter::NamingConverter
};
use crate::utils::{to_snake_case_with_serde, generate_pubkey_serde_attr};

/// Anchor Instructions 模板
pub struct AnchorInstructionsTemplate<'a> {
    pub idl: &'a AnchorIdl,
    pub args: &'a Args,
    naming_converter: RefCell<NamingConverter>,
}

impl<'a> AnchorInstructionsTemplate<'a> {
    /// 创建新的 Anchor Instructions 模板
    pub fn new(idl: &'a AnchorIdl, args: &'a Args) -> Self {
        Self { 
            idl, 
            args,
            naming_converter: RefCell::new(NamingConverter::new()),
        }
    }

    /// 转换 AnchorFieldType 为 Rust 类型
    fn convert_anchor_field_type_to_rust(field_type: &AnchorFieldType) -> TokenStream {
        match field_type {
            AnchorFieldType::Basic(type_str) => {
                // 处理基础类型字符串
                let type_ident = match type_str.as_str() {
                    "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "u64" | "i64" | "u128" | "i128" => {
                        syn::Ident::new(type_str, proc_macro2::Span::call_site())
                    },
                    "bool" => syn::Ident::new("bool", proc_macro2::Span::call_site()),
                    "string" => syn::Ident::new("String", proc_macro2::Span::call_site()),
                    "publicKey" | "pubkey" => syn::Ident::new("Pubkey", proc_macro2::Span::call_site()),
                    "bytes" => return quote! { Vec<u8> },
                    // 自定义类型
                    _ => syn::Ident::new(type_str, proc_macro2::Span::call_site()),
                };
                quote! { #type_ident }
            },
            AnchorFieldType::Complex { kind, params } => {
                match kind.as_str() {
                    "vec" => {
                        if let Some(params) = params {
                            if let Some(inner_param) = params.get(0) {
                                if let Ok(inner_type_str) = serde_json::from_value::<String>(inner_param.clone()) {
                                    let inner_type = Self::convert_anchor_field_type_to_rust(&AnchorFieldType::Basic(inner_type_str));
                                    return quote! { Vec<#inner_type> };
                                }
                            }
                        }
                        quote! { Vec<u8> } // Fallback
                    },
                    "array" => {
                        if let Some(params) = params {
                            if params.len() >= 2 {
                                if let (Ok(inner_type_str), Ok(size)) = (
                                    serde_json::from_value::<String>(params[0].clone()),
                                    serde_json::from_value::<u32>(params[1].clone())
                                ) {
                                    let inner_type = Self::convert_anchor_field_type_to_rust(&AnchorFieldType::Basic(inner_type_str));
                                    return quote! { [#inner_type; #size] };
                                }
                            }
                        }
                        quote! { [u8; 32] } // Fallback
                    },
                    "option" => {
                        if let Some(params) = params {
                            if let Some(inner_param) = params.get(0) {
                                if let Ok(inner_type_str) = serde_json::from_value::<String>(inner_param.clone()) {
                                    let inner_type = Self::convert_anchor_field_type_to_rust(&AnchorFieldType::Basic(inner_type_str));
                                    return quote! { Option<#inner_type> };
                                }
                            }
                        }
                        quote! { Option<u8> } // Fallback
                    },
                    _ => {
                        // 未知的复合类型，当作自定义类型处理
                        let type_name = syn::Ident::new(kind, proc_macro2::Span::call_site());
                        quote! { #type_name }
                    }
                }
            },
            // This pattern is handled by the Basic variant above
            AnchorFieldType::defined(type_name) => {
                // 使用完整路径引用types模块中的类型
                let type_path = format!("crate::types::{}", type_name);
                let type_path: syn::Path = syn::parse_str(&type_path).unwrap();
                quote! { #type_path }
            },
            AnchorFieldType::array(inner_type, size) => {
                let inner_token = Self::convert_anchor_field_type_to_rust(inner_type);
                let size_literal = proc_macro2::Literal::usize_unsuffixed(*size);
                quote! { [#inner_token; #size_literal] }
            },
            AnchorFieldType::vec(inner_type) => {
                let inner_token = Self::convert_anchor_field_type_to_rust(inner_type);
                quote! { Vec<#inner_token> }
            },
            AnchorFieldType::option(inner_type) => {
                let inner_token = Self::convert_anchor_field_type_to_rust(inner_type);
                quote! { Option<#inner_token> }
            },
            AnchorFieldType::PrimitiveOrPubkey(type_str) => {
                let type_ident = match type_str.as_str() {
                    "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "u64" | "i64" | "u128" | "i128" => {
                        syn::Ident::new(type_str, proc_macro2::Span::call_site())
                    },
                    "bool" => syn::Ident::new("bool", proc_macro2::Span::call_site()),
                    "string" => syn::Ident::new("String", proc_macro2::Span::call_site()),
                    "publicKey" | "pubkey" => syn::Ident::new("Pubkey", proc_macro2::Span::call_site()),
                    "bytes" => return quote! { Vec<u8> },
                    _ => syn::Ident::new(type_str, proc_macro2::Span::call_site()),
                };
                quote! { #type_ident }
            },
        }
    }

    /// 生成 discriminator 常量和账户长度常量
    pub fn generate_discriminator_constants(&self) -> TokenStream {
        let instructions = self.idl.instructions.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if instructions.is_empty() {
            return quote! {};
        }

        let constants = instructions.iter().map(|ix| {
            // 使用统一的命名转换器确保常量名称一致性
            let mut naming_converter = self.naming_converter.borrow_mut();
            let const_base_name = naming_converter.to_screaming_snake_case(&ix.name);
            
            let discm_const_name = syn::Ident::new(
                &format!("{}_IX_DISCM", const_base_name),
                proc_macro2::Span::call_site(),
            );
            let accounts_len_const = syn::Ident::new(
                &format!("{}_IX_ACCOUNTS_LEN", const_base_name),
                proc_macro2::Span::call_site(),
            );
            
            // Use the discriminator from IDL (8-byte array for Anchor)
            let discriminator = {
                let bytes = ix.discriminator.iter().map(|&b| b).collect::<Vec<_>>();
                quote! { [#(#bytes),*] }
            };

            // Calculate accounts length
            let accounts_len = ix.accounts.as_ref().map_or(0, |accounts| accounts.len());
            let accounts_len_literal = proc_macro2::Literal::usize_unsuffixed(accounts_len);

            quote! {
                pub const #discm_const_name: [u8; 8] = #discriminator;
                pub const #accounts_len_const: usize = #accounts_len_literal;
            }
        });

        quote! {
            #(#constants)*
        }
    }

    /// 生成指令枚举
    pub fn generate_instruction_enum(&self) -> TokenStream {
        let instructions = self.idl.instructions.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if instructions.is_empty() {
            return quote! {};
        }

        let enum_variants = instructions.iter().map(|ix| {
            let variant_name = syn::Ident::new(&ix.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let keys_struct_name = syn::Ident::new(&format!("{}Keys", ix.name.to_case(Case::Pascal)), proc_macro2::Span::call_site());
            let args_struct_name = syn::Ident::new(
                &format!("{}IxData", ix.name.to_case(Case::Pascal)),
                proc_macro2::Span::call_site(),
            );
            
            let doc_comments = DocGenerator::generate_instruction_docs(&ix.docs);

            // 统一所有指令都包含 Keys 和 Args
            quote! {
                #doc_comments
                #variant_name(#keys_struct_name, #args_struct_name),
            }
        });

        let program_name = syn::Ident::new(
            &format!("{}Instruction", &self.idl.program_name().to_case(Case::Pascal)),
            proc_macro2::Span::call_site(),
        );

        quote! {
            /// Program instruction types for Anchor contract
            #[derive(Clone, Debug, PartialEq)]
            pub enum #program_name {
                #(#enum_variants)*
            }
        }
    }
    

    /// 生成解析函数
    pub fn generate_parse_function(&self) -> TokenStream {
        let instructions = self.idl.instructions.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if instructions.is_empty() {
            return quote! {};
        }

        let program_name = syn::Ident::new(
            &format!("{}Instruction", &self.idl.program_name().to_case(Case::Pascal)),
            proc_macro2::Span::call_site(),
        );

        let match_arms = self.generate_match_arms();
        // Helper functions removed for client SDK - not needed

        quote! {

            /// Parse instruction data based on 8-byte discriminator (Anchor contracts)
            pub fn parse_instruction(data: &[u8], accounts: &[Pubkey]) -> Result<#program_name, std::io::Error> {
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

                match discriminator {
                    #match_arms
                    _ => Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Unknown instruction discriminator: {:?}", discriminator)
                    )),
                }
            }
        }
    }

    /// 生成 match 分支
    pub fn generate_match_arms(&self) -> TokenStream {
        let instructions = self.idl.instructions.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        
        let match_arms = instructions.iter().map(|ix| {
            let variant_name = syn::Ident::new(&ix.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let keys_struct_name = syn::Ident::new(&format!("{}Keys", ix.name.to_case(Case::Pascal)), proc_macro2::Span::call_site());
            
            // 使用统一的命名转换器确保常量名称一致性
            let mut naming_converter = self.naming_converter.borrow_mut();
            let const_base_name = naming_converter.to_screaming_snake_case(&ix.name);
            
            let accounts_len_const = syn::Ident::new(&format!("{}_IX_ACCOUNTS_LEN", const_base_name), proc_macro2::Span::call_site());
            let discm_const_name = syn::Ident::new(
                &format!("{}_IX_DISCM", const_base_name),
                proc_macro2::Span::call_site(),
            );
            let instruction_name_str = ix.name.to_case(Case::Pascal);
            let program_name = syn::Ident::new(
                &format!("{}Instruction", &self.idl.program_name().to_case(Case::Pascal)),
                proc_macro2::Span::call_site(),
            );

            let args_struct_name = syn::Ident::new(
                &format!("{}IxData", ix.name.to_case(Case::Pascal)),
                proc_macro2::Span::call_site(),
            );
            
            // 统一所有指令都处理 Keys 和 Args
            quote! {
                #discm_const_name => {
                    check_min_accounts_req(accounts, #accounts_len_const, #instruction_name_str)?;
                    let ix_accounts = #keys_struct_name::from(&accounts[..#accounts_len_const]);
                    let args = #args_struct_name::from_bytes(&data[..])?;
                    Ok(#program_name::#variant_name(ix_accounts, args))
                },
            }
        });

        quote! {
            #(#match_arms)*
        }
    }
    
    /// 生成指令参数结构体
    pub fn generate_instruction_args(&self) -> TokenStream {
        let instructions = self.idl.instructions.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if instructions.is_empty() {
            return quote! {};
        }

        let arg_structs = instructions.iter().map(|ix| {
            let struct_name = syn::Ident::new(
                &format!("{}IxData", ix.name.to_case(Case::Pascal)),
                proc_macro2::Span::call_site(),
            );
            // 使用统一的命名转换器确保常量名称一致性
            let mut naming_converter = self.naming_converter.borrow_mut();
            let const_base_name = naming_converter.to_screaming_snake_case(&ix.name);
            
            let discm_const_name = syn::Ident::new(
                &format!("{}_IX_DISCM", const_base_name),
                proc_macro2::Span::call_site(),
            );
            
            let doc_comments = DocGenerator::generate_instruction_docs(&ix.docs);
            
            let args_fields = if let Some(args) = &ix.args {
                let fields = args.iter().map(|arg| {
                    let (snake_field_name, serde_attr) = to_snake_case_with_serde(&arg.name);
                    let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                    let field_type = Self::convert_anchor_field_type_to_rust(&arg.field_type);
                    
                    // 检查是否为 Pubkey 类型，如果是则添加特殊的 serde 属性
                    let pubkey_serde_attr = if Self::is_anchor_field_pubkey_type(&arg.field_type) {
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
                quote! {
                    pub discriminator: [u8; 8],
                    #(#fields)*
                }
            } else {
                quote! {
                    pub discriminator: [u8; 8],
                }
            };
            
            let default_impl = if let Some(args) = &ix.args {
                let default_fields = args.iter().map(|arg| {
                    let (snake_field_name, _) = to_snake_case_with_serde(&arg.name);
                    let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                    quote! { #field_name: Default::default(), }
                });
                quote! {
                    discriminator: #discm_const_name,
                    #(#default_fields)*
                }
            } else {
                quote! {
                    discriminator: #discm_const_name,
                }
            };

            // 生成 new() 方法的参数和赋值
            let (new_method_args, new_method_assignments) = if let Some(args) = &ix.args {
                let arg_params = args.iter().map(|arg| {
                    let (snake_field_name, _) = to_snake_case_with_serde(&arg.name);
                    let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                    let field_type = Self::convert_anchor_field_type_to_rust(&arg.field_type);
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

            quote! {
                #doc_comments
                #[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Clone, Debug, PartialEq)]
                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                pub struct #struct_name {
                    #args_fields
                }

                impl Default for #struct_name {
                    fn default() -> Self {
                        Self {
                            #default_impl
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
                    
                    pub fn discriminator() -> [u8; 8] {
                        #discm_const_name
                    }
                    
                    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
                        borsh::to_vec(self)
                    }
                }
            }
        });

        quote! {
                        
            #(#arg_structs)*
        }
    }

    /// 生成指令账户结构体
    pub fn generate_instruction_accounts(&self) -> TokenStream {
        let instructions = self.idl.instructions.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if instructions.is_empty() {
            return quote! {};
        }

        // 这里需要参考Legacy系统的账户生成逻辑
        // 由于篇幅限制，先返回空实现，稍后完成
        quote! {
            // TODO: Generate instruction accounts structs
        }
    }

    /// 生成客户端指令函数
    pub fn generate_client_instruction_functions(&self) -> TokenStream {
        let instructions = self.idl.instructions.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if instructions.is_empty() {
            return quote! {};
        }

        let client_functions = instructions.iter().map(|ix| {
            let ix_fn_name = syn::Ident::new(
                &format!("{}_ix", ix.name.to_case(Case::Snake)),
                proc_macro2::Span::call_site(),
            );
            let ix_with_program_id_fn_name = syn::Ident::new(
                &format!("{}_ix_with_program_id", ix.name.to_case(Case::Snake)),
                proc_macro2::Span::call_site(),
            );
            let keys_struct_name = syn::Ident::new(
                &format!("{}Keys", ix.name.to_case(Case::Pascal)),
                proc_macro2::Span::call_site(),
            );
            let args_struct_name = syn::Ident::new(
                &format!("{}IxData", ix.name.to_case(Case::Pascal)),
                proc_macro2::Span::call_site(),
            );
            let accounts_len_const = syn::Ident::new(
                &format!("{}_IX_ACCOUNTS_LEN", ix.name.to_shouty_snake_case()),
                proc_macro2::Span::call_site(),
            );

            let has_accounts = ix.accounts.as_ref().map_or(false, |accounts| !accounts.is_empty());
            let has_args = ix.args.as_ref().map_or(false, |args| !args.is_empty());

            let fn_params = if has_accounts && has_args {
                quote! { keys: #keys_struct_name, args: #args_struct_name }
            } else if has_accounts {
                quote! { keys: #keys_struct_name }
            } else if has_args {
                quote! { args: #args_struct_name }
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
                let args_creation = if !has_args {
                    quote! { let args = #args_struct_name::default(); }
                } else {
                    quote! {}
                };
                (
                    quote! {
                        let metas: [solana_instruction::AccountMeta; #accounts_len_const] = keys.into();
                        #args_creation
                    },
                    quote! { Vec::from(metas) },
                )
            } else {
                let args_creation = if !has_args {
                    quote! { let args = #args_struct_name::default(); }
                } else {
                    quote! {}
                };
                (
                    quote! {
                        #args_creation
                    },
                    quote! { Vec::new() },
                )
            };

            quote! {
                pub fn #ix_with_program_id_fn_name(
                    program_id: Pubkey, 
                    #fn_params
                ) -> Result<solana_instruction::Instruction, std::io::Error> {
                    #fn_body
                    Ok(solana_instruction::Instruction {
                        program_id,
                        accounts: #accounts_expr,
                        data: args.try_to_vec()?,
                    })
                }

                pub fn #ix_fn_name(#fn_params) -> Result<solana_instruction::Instruction, std::io::Error> {
                    #ix_with_program_id_fn_name(crate::ID, #fn_args)
                }
            }
        });

        quote! {
            // 使用绝对路径，不需要导入
            #[allow(unused_imports)]
            use solana_pubkey::Pubkey;
            
            #(#client_functions)*
        }
    }

    /// 生成PDA相关函数
    pub fn generate_pda_functions(&self) -> TokenStream {
        // TODO: Generate PDA derivation functions
        quote! {}
    }

    /// 生成针对特定指令优化的导入（基于代码内容分析）
    fn generate_optimized_imports(&self, _ix: &AnchorInstruction) -> TokenStream {
        let mut import_manager = ImportManager::new();
        
        // 添加基础导入
        import_manager.add_import(ImportType::Solana(SolanaImport::Pubkey));
        import_manager.mark_as_used("solana_pubkey");
        
        // 添加指令相关导入
        import_manager.add_import(ImportType::Solana(SolanaImport::Instruction));
        import_manager.mark_as_used("instruction");
        
        import_manager.generate_optimized_imports()
    }

    /// 为单个instruction生成完整的文件内容
    pub fn generate_single_instruction_file(&self, ix: &AnchorInstruction) -> TokenStream {
        log::debug!("🔍 开始生成指令文件: {}", ix.name);
        
        // 使用智能导入管理器
        let imports = self.generate_optimized_imports(ix);
        log::debug!("📦 ImportManager生成的导入: {}", imports);
        
        // 使用命名转换器生成优化的名称
        let mut naming_converter = self.naming_converter.borrow_mut();
        let instruction_const_name = naming_converter.convert_constant_name(&format!("{}_IX_DISCM", &ix.name));
        let accounts_len_const_name = naming_converter.convert_constant_name(&format!("{}_IX_ACCOUNTS_LEN", &ix.name));
        
        let discm_const_name = syn::Ident::new(&instruction_const_name, proc_macro2::Span::call_site());
        let accounts_len_const = syn::Ident::new(&accounts_len_const_name, proc_macro2::Span::call_site());
        
        let discriminator = {
            let bytes = ix.discriminator.iter().map(|&b| b).collect::<Vec<_>>();
            quote! { [#(#bytes),*] }
        };
        let accounts_len = ix.accounts.as_ref().map_or(0, |accounts| accounts.len());
        let accounts_len_literal = proc_macro2::Literal::usize_unsuffixed(accounts_len);

        // 生成IxData结构体名称
        let struct_name_str = naming_converter.convert_instruction_struct_name(&ix.name);
        log::debug!("🏷️ Instruction '{}' -> struct name: '{}'", ix.name, struct_name_str);
        let struct_name = syn::Ident::new(&struct_name_str, proc_macro2::Span::call_site());
        let doc_comments = DocGenerator::generate_instruction_docs(&ix.docs);
        log::debug!("📝 Doc comments (from IxData generation) TokenStream: {}", doc_comments);
        
        let args_fields = if let Some(args) = &ix.args {
            let fields = args.iter().map(|arg| {
                let (snake_field_name, serde_attr) = to_snake_case_with_serde(&arg.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                let field_type = Self::convert_anchor_field_type_to_rust(&arg.field_type);
                
                // 检查是否为 Pubkey 类型，如果是则添加特殊的 serde 属性
                let pubkey_serde_attr = if Self::is_anchor_field_pubkey_type(&arg.field_type) {
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
            quote! {
                pub discriminator: [u8; 8],
                #(#fields)*
            }
        } else {
            quote! {
                pub discriminator: [u8; 8],
            }
        };
        
        let default_impl = if let Some(args) = &ix.args {
            let default_fields = args.iter().map(|arg| {
                let (snake_field_name, _) = to_snake_case_with_serde(&arg.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                quote! { #field_name: Default::default(), }
            });
            quote! {
                discriminator: #discm_const_name,
                #(#default_fields)*
            }
        } else {
            quote! {
                discriminator: #discm_const_name,
            }
        };

        // 生成 new() 方法的参数和赋值
        let (new_method_args, new_method_assignments) = if let Some(args) = &ix.args {
            let arg_params = args.iter().map(|arg| {
                let (snake_field_name, _) = to_snake_case_with_serde(&arg.name);
                let field_name = syn::Ident::new(&snake_field_name, proc_macro2::Span::call_site());
                let field_type = Self::convert_anchor_field_type_to_rust(&arg.field_type);
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

        // 生成客户端函数名称
        let ix_fn_name_str = naming_converter.convert_function_name(&format!("{}_ix", &ix.name));
        let ix_with_program_id_fn_name_str = naming_converter.convert_function_name(&format!("{}_ix_with_program_id", &ix.name));
        let keys_struct_name_str = naming_converter.convert_struct_name(&format!("{}Keys", &ix.name));
        
        let ix_fn_name = syn::Ident::new(&ix_fn_name_str, proc_macro2::Span::call_site());
        let ix_with_program_id_fn_name = syn::Ident::new(&ix_with_program_id_fn_name_str, proc_macro2::Span::call_site());
        let keys_struct_name = syn::Ident::new(&keys_struct_name_str, proc_macro2::Span::call_site());

        let has_accounts = ix.accounts.as_ref().map_or(false, |accounts| !accounts.is_empty());
        let has_args = ix.args.as_ref().map_or(false, |args| !args.is_empty());

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
            let args_creation = if !has_args {
                quote! { let args = #struct_name::default(); }
            } else {
                quote! {}
            };
            (
                quote! {
                    let metas: [solana_instruction::AccountMeta; #accounts_len_const] = keys.into();
                    #args_creation
                },
                quote! { Vec::from(metas) },
            )
        } else {
            let args_creation = if !has_args {
                quote! { let args = #struct_name::default(); }
            } else {
                quote! {}
            };
            (
                quote! {
                    #args_creation
                },
                quote! { Vec::new() },
            )
        };

        // 生成Keys结构体字段
        let keys_fields = if let Some(accounts) = &ix.accounts {
            accounts.iter().map(|account| {
                let field_name = syn::Ident::new(&account.name.to_case(Case::Snake), proc_macro2::Span::call_site());
                quote! { pub #field_name: Pubkey, }
            }).collect()
        } else {
            vec![]
        };
        
        // 生成Keys Default实现字段
        let keys_default_fields = if let Some(accounts) = &ix.accounts {
            accounts.iter().map(|account| {
                let field_name = syn::Ident::new(&account.name.to_case(Case::Snake), proc_macro2::Span::call_site());
                quote! { #field_name: Pubkey::default(), }
            }).collect()
        } else {
            vec![]
        };
        
        // 生成to_vec字段列表
        let to_vec_fields = if let Some(accounts) = &ix.accounts {
            accounts.iter().map(|account| {
                let field_name = syn::Ident::new(&account.name.to_case(Case::Snake), proc_macro2::Span::call_site());
                quote! { self.#field_name, }
            }).collect()
        } else {
            vec![]
        };
        
        // 生成From [Pubkey] 实现字段
        let keys_from_array_fields = if let Some(accounts) = &ix.accounts {
            accounts.iter().enumerate().map(|(i, account)| {
                let field_name = syn::Ident::new(&account.name.to_case(Case::Snake), proc_macro2::Span::call_site());
                let index_literal = proc_macro2::Literal::usize_unsuffixed(i);
                quote! { #field_name: pubkeys[#index_literal], }
            }).collect()
        } else {
            vec![]
        };
        
        // 生成Into [AccountMeta] 实现字段
        let keys_into_metas_fields = if let Some(accounts) = &ix.accounts {
            accounts.iter().map(|account| {
                let field_name = syn::Ident::new(&account.name.to_case(Case::Snake), proc_macro2::Span::call_site());
                quote! { solana_instruction::AccountMeta::new(keys.#field_name, false), }
            }).collect()
        } else {
            vec![]
        };

        let doc_string = format!("Instruction: {}", ix.name);
        log::debug!("📝 Doc string生成: {}", doc_string);
        
        let result = quote! {
            #![doc = #doc_string]
            #doc_comments
            
            #imports
            
            // Constants
            pub const #discm_const_name: [u8; 8] = #discriminator;
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
                        #default_impl
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
                
                pub fn discriminator() -> [u8; 8] {
                    #discm_const_name
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

            impl #keys_struct_name {
                /// Convert Keys to Vec<Pubkey>
                pub fn to_vec(&self) -> Vec<Pubkey> {
                    vec![
                        #(#to_vec_fields)*
                    ]
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
                    data: args.try_to_vec()?,
                })
            }

            pub fn #ix_fn_name(#fn_params) -> Result<solana_instruction::Instruction, std::io::Error> {
                #ix_with_program_id_fn_name(crate::ID, #fn_args)
            }
        };
        
        log::debug!("🏗️ 完整生成的TokenStream (前500字符): {}", 
            result.to_string().chars().take(500).collect::<String>());
        log::debug!("🔍 TokenStream中use crate::*出现次数: {}", 
            result.to_string().matches("use crate::*").count());
        
        result
    }
}

impl<'a> ContractModeTemplate for AnchorInstructionsTemplate<'a> {
    type DiscriminatorType = [u8; 8];

    fn discriminator_size() -> usize {
        8
    }

    fn parse_discriminator_code() -> TokenStream {
        quote! {
            let discriminator: [u8; 8] = data[0..8].try_into()?;
        }
    }

    fn generate_constants(&self) -> TokenStream {
        self.generate_discriminator_constants()
    }

    fn generate_tests(&self) -> TokenStream {
        // TODO: Agent 2 实现
        quote! {}
    }
}

impl<'a> TemplateGenerator for AnchorInstructionsTemplate<'a> {
    fn get_standard_module_name(&self) -> &'static str {
        "instructions"
    }

    fn gen_files(&self) -> Vec<(String, TokenStream)> {
        let instructions = self.idl.instructions.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if instructions.is_empty() {
            return vec![("mod.rs".to_string(), quote! {
                //! Empty instructions module - no instructions found in IDL
            })];
        }
        
        let mut files = Vec::new();
        
        // 为每个instruction生成独立文件
        for ix in instructions {
            let file_name = format!("{}.rs", ix.name.to_case(Case::Snake));
            let file_content = self.generate_single_instruction_file(ix);
            files.push((file_name, file_content));
        }
        
        files
    }

    fn gen_mod_file(&self) -> TokenStream {
        let instructions = self.idl.instructions.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if instructions.is_empty() {
            return quote! {
                //! Instructions module - no instructions found in IDL
            };
        }
        
        // 使用智能导入管理器生成精确导入
        let imports = self.generate_mod_rs_imports();
        
        // 生成模块声明和精确重导出
        let mut naming_converter = self.naming_converter.borrow_mut();
        let module_declarations_and_exports: Vec<TokenStream> = instructions.iter().map(|ix| {
            let module_name_str = naming_converter.convert_function_name(&ix.name);
            let module_name = syn::Ident::new(&module_name_str, proc_macro2::Span::call_site());
            
            // 生成精确重导出，避免使用glob导入
            let ix_data_struct = naming_converter.convert_instruction_struct_name(&ix.name);
            let keys_struct = naming_converter.convert_struct_name(&format!("{}Keys", &ix.name));
            let discm_const = naming_converter.convert_constant_name(&format!("{}_IX_DISCM", &ix.name));
            let accounts_len_const = naming_converter.convert_constant_name(&format!("{}_IX_ACCOUNTS_LEN", &ix.name));
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
            //! Anchor instructions module
            //! Generated instruction definitions and client functions
            //! Each instruction is defined in its own file
            //!
            //! This module provides precise imports to avoid naming conflicts
            //! and improve compile times by avoiding glob imports.
            
            #imports
            
            #(#module_declarations_and_exports)*
        }
    }
    
    fn is_single_root_file(&self) -> bool {
        false // instructions使用目录结构
    }
}

impl<'a> AnchorInstructionsTemplate<'a> {
    /// 生成mod.rs文件的优化导入  
    fn generate_mod_rs_imports(&self) -> TokenStream {
        let mut import_manager = ImportManager::new();
        
        // mod.rs通常只需要基础类型导入
        import_manager.add_import(ImportType::Borsh);
        import_manager.mark_as_used("borsh");
        
        import_manager.add_import(ImportType::Solana(SolanaImport::Pubkey));
        import_manager.mark_as_used("solana_pubkey");
        
        import_manager.generate_optimized_imports()
    }
    
    /// 检查 Anchor 字段类型是否为 Pubkey
    fn is_anchor_field_pubkey_type(field_type: &AnchorFieldType) -> bool {
        match field_type {
            AnchorFieldType::Basic(s) => {
                matches!(s.as_str(), "publicKey" | "pubkey" | "Pubkey")
            },
            AnchorFieldType::PrimitiveOrPubkey(s) => {
                matches!(s.as_str(), "publicKey" | "pubkey" | "Pubkey")
            },
            _ => false
        }
    }
}