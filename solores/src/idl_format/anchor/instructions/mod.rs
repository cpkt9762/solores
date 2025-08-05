use heck::{ToPascalCase, ToSnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::idl_format::IdlCodegenModule;

mod instruction;
pub use instruction::*;

pub struct IxCodegenModule<'a> {
    pub program_name: &'a str,
    pub instructions: &'a [NamedInstruction],
    pub is_anchor_contract: bool,  // Contract type information
}

impl IdlCodegenModule for IxCodegenModule<'_> {
    fn name(&self) -> &str {
        "instructions"
    }

    fn has_multiple_files(&self) -> bool {
        true
    }

    fn gen_files(&self) -> Vec<(String, TokenStream)> {
        let mut files = Vec::new();
        
        // Generate individual instruction files
        for instruction in self.instructions.iter() {
            let filename = format!("{}.rs", instruction.name.to_snake_case());
            let content = self.gen_individual_instruction_file(instruction);
            files.push((filename, content));
        }
        
        files
    }

    fn gen_mod_file(&self) -> TokenStream {
        let program_ix_enum_content = self.gen_program_ix_enum();
        let common_imports = self.gen_common_imports();
        let module_imports = self.instructions.iter().map(|ix| {
            let module_name = format_ident!("{}", ix.name.to_snake_case());
            quote! {
                pub mod #module_name;
                pub use #module_name::*;
            }
        });

        quote! {
            #common_imports
            #(#module_imports)*
            #program_ix_enum_content
        }
    }

    fn gen_head(&self) -> TokenStream {
        let mut res = quote! {};
        let has_args = self
            .instructions
            .iter()
            .map(|ix| ix.has_ix_args())
            .any(|b| b);
        if has_args {
            res.extend(quote! {
                use borsh::{BorshDeserialize, BorshSerialize};
            });
        }
        let has_accounts = self
            .instructions
            .iter()
            .map(|ix| ix.has_accounts())
            .any(|b| b);

        let mut solana_program_imports = if has_accounts {
            quote! {
                account_info::AccountInfo,
                entrypoint::ProgramResult,
                instruction::{AccountMeta, Instruction},
                program::{invoke, invoke_signed},
                pubkey::Pubkey,
            }
        } else {
            quote! {
                entrypoint::ProgramResult,
                instruction::Instruction,
                program::{invoke, invoke_signed},
                pubkey::Pubkey,
            }
        };

        res.extend(quote! {
            use solana_program::{#solana_program_imports};
        });
        
        // Only import Read for Anchor contracts
        if self.is_anchor_contract {
            res.extend(quote! {
                use std::io::Read;
            });
        }
        
        let has_defined_type = self
            .instructions
            .iter()
            .map(|ix| ix.args_has_defined_type())
            .any(|b| b);
        if has_defined_type {
            res.extend(quote! {
                use crate::*;
            });
        }

        // program ix enum
        let program_ix_enum_ident =
            format_ident!("{}ProgramIx", self.program_name.to_pascal_case());
        let program_ix_enum_variants = self.instructions.iter().map(enum_variant);
        let serialize_variant_match_arms =
            self.instructions.iter().map(serialize_variant_match_arm);
        let deserialize_variant_match_arms =
            self.instructions.iter().map(deserialize_variant_match_arm);

        res.extend(quote! {
            #[derive(Clone, Debug, PartialEq)]
            pub enum #program_ix_enum_ident {
                #(#program_ix_enum_variants),*
            }

            impl #program_ix_enum_ident {
                pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
                    let mut reader = buf;
                    let mut maybe_discm = [0u8; 8];
                    reader.read_exact(&mut maybe_discm)?;
                    match maybe_discm {
                        #(#deserialize_variant_match_arms),*,
                        _ => Err(
                            std::io::Error::new(
                                std::io::ErrorKind::Other, format!("discm {:?} not found", maybe_discm)
                            )
                        ),
                    }
                }

                pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
                    match self {
                        #(#serialize_variant_match_arms),*,
                    }
                }

                pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
                    let mut data = Vec::new();
                    self.serialize(&mut data)?;
                    Ok(data)
                }
            }
        });

        if has_accounts {
            res.extend(quote! {
                fn invoke_instruction<'info, A: Into<[AccountInfo<'info>; N]>, const N: usize>(
                    ix: &Instruction,
                    accounts: A,
                ) -> ProgramResult {
                    let account_info: [AccountInfo<'info>; N] = accounts.into();
                    invoke(ix, &account_info)
                }
                fn invoke_instruction_signed<'info, A: Into<[AccountInfo<'info>; N]>, const N: usize>(
                    ix: &Instruction,
                    accounts: A,
                    seeds: &[&[&[u8]]],
                ) -> ProgramResult {
                    let account_info: [AccountInfo<'info>; N] = accounts.into();
                    invoke_signed(ix, &account_info, seeds)
                }
            });
        }

        res
    }

    fn gen_body(&self) -> TokenStream {
        self.instructions
            .iter()
            .map(|e| e.into_token_stream())
            .collect()
    }
}

pub fn enum_variant(ix: &NamedInstruction) -> TokenStream {
    let variant_ident = format_ident!("{}", ix.name.to_pascal_case());
    let mut res = quote!(
        #variant_ident
    );
    if ix.has_ix_args() {
        let ix_args_ident = ix.ix_args_ident();
        res.extend(quote! {
            (#ix_args_ident)
        })
    }
    res
}

pub fn serialize_variant_match_arm(ix: &NamedInstruction) -> TokenStream {
    let variant_ident = format_ident!("{}", ix.name.to_pascal_case());
    let discm_ident = ix.discm_ident();
    let serialize_expr = if ix.has_ix_args() {
        quote! {{
            writer.write_all(&#discm_ident)?;
            args.serialize(&mut writer)
        }}
    } else {
        quote! { writer.write_all(&#discm_ident) }
    };
    let mut left_matched = quote! { Self::#variant_ident };
    if ix.has_ix_args() {
        left_matched.extend(quote! { (args) });
    }
    quote! {
        #left_matched => #serialize_expr
    }
}

pub fn deserialize_variant_match_arm(ix: &NamedInstruction) -> TokenStream {
    let variant_ident = format_ident!("{}", ix.name.to_pascal_case());
    let discm_ident = ix.discm_ident();
    let mut variant_expr = quote! {
        Self::#variant_ident
    };
    if ix.has_ix_args() {
        let ix_args_ident = ix.ix_args_ident();
        variant_expr.extend(quote! {
            (#ix_args_ident::deserialize(&mut reader)?)
        })
    }
    quote! {
        #discm_ident => Ok(#variant_expr)
    }
}

impl IxCodegenModule<'_> {
    fn gen_common_imports(&self) -> TokenStream {
        let has_args = self.instructions.iter().any(|ix| ix.has_ix_args());
        let has_accounts = self.instructions.iter().any(|ix| ix.has_accounts());
        let has_defined_type = self.instructions.iter().any(|ix| ix.args_has_defined_type());

        let mut res = quote! {};
        
        if has_args {
            res.extend(quote! {
                use borsh::{BorshDeserialize, BorshSerialize};
            });
        }

        let mut solana_program_imports = if has_accounts {
            quote! {
                account_info::AccountInfo,
                entrypoint::ProgramResult,
                instruction::{AccountMeta, Instruction},
                program::{invoke, invoke_signed},
                pubkey::Pubkey,
            }
        } else {
            quote! {
                entrypoint::ProgramResult,
                instruction::Instruction,
                program::{invoke, invoke_signed},
                pubkey::Pubkey,
            }
        };


        res.extend(quote! {
            use solana_program::{#solana_program_imports};
        });
        
        // Add std::io::Read import for Anchor contracts that need discriminator parsing
        if self.is_anchor_contract {
            res.extend(quote! {
                use std::io::Read;
            });
        }

        if has_defined_type {
            res.extend(quote! {
                use crate::*;
            });
        }

        if has_accounts {
            res.extend(quote! {
                fn invoke_instruction<'info, A: Into<[AccountInfo<'info>; N]>, const N: usize>(
                    ix: &Instruction,
                    accounts: A,
                ) -> ProgramResult {
                    let account_info: [AccountInfo<'info>; N] = accounts.into();
                    invoke(ix, &account_info)
                }
                fn invoke_instruction_signed<'info, A: Into<[AccountInfo<'info>; N]>, const N: usize>(
                    ix: &Instruction,
                    accounts: A,
                    seeds: &[&[&[u8]]],
                ) -> ProgramResult {
                    let account_info: [AccountInfo<'info>; N] = accounts.into();
                    invoke_signed(ix, &account_info, seeds)
                }
            });
        }

        res
    }

    fn gen_program_ix_enum(&self) -> TokenStream {
        if self.is_anchor_contract {
            self.gen_anchor_program_ix_enum()
        } else {
            self.gen_non_anchor_program_ix_enum()
        }
    }

    fn gen_anchor_program_ix_enum(&self) -> TokenStream {
        let program_ix_enum_ident = format_ident!("{}ProgramIx", self.program_name.to_pascal_case());
        let program_ix_enum_variants = self.instructions.iter().map(enum_variant);
        let serialize_variant_match_arms = self.instructions.iter().map(serialize_variant_match_arm);
        let deserialize_variant_match_arms = self.instructions.iter().map(deserialize_variant_match_arm);

        quote! {
            #[derive(Clone, Debug, PartialEq)]
            pub enum #program_ix_enum_ident {
                #(#program_ix_enum_variants),*
            }

            impl #program_ix_enum_ident {
                pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
                    let mut reader = buf;
                    let mut maybe_discm = [0u8; 8];
                    reader.read_exact(&mut maybe_discm)?;
                    match maybe_discm {
                        #(#deserialize_variant_match_arms),*,
                        _ => Err(
                            std::io::Error::new(
                                std::io::ErrorKind::Other, format!("discm {:?} not found", maybe_discm)
                            )
                        ),
                    }
                }

                pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
                    match self {
                        #(#serialize_variant_match_arms),*,
                    }
                }

                pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
                    let mut data = Vec::new();
                    self.serialize(&mut data)?;
                    Ok(data)
                }
            }
        }
    }

    fn gen_non_anchor_program_ix_enum(&self) -> TokenStream {
        let program_ix_enum_ident = format_ident!("{}ProgramIx", self.program_name.to_pascal_case());
        let program_ix_enum_variants = self.instructions.iter().map(enum_variant);
        
        // Generate serialize match arms for non-Anchor (u8 discriminator)
        let serialize_variant_match_arms = self.instructions.iter().map(|ix| {
            let variant_ident = format_ident!("{}", ix.name.to_pascal_case());
            let discm_ident = ix.discm_ident();
            let serialize_expr = if ix.has_ix_args() {
                quote! {{
                    writer.write_all(&[#discm_ident])?;
                    args.serialize(&mut writer)
                }}
            } else {
                quote! { writer.write_all(&[#discm_ident]) }
            };
            let mut left_matched = quote! { Self::#variant_ident };
            if ix.has_ix_args() {
                left_matched.extend(quote! { (args) });
            }
            quote! {
                #left_matched => #serialize_expr
            }
        });

        // Generate deserialize match arms for non-Anchor (u8 discriminator)
        let deserialize_variant_match_arms = self.instructions.iter().map(|ix| {
            let variant_ident = format_ident!("{}", ix.name.to_pascal_case());
            let discm_ident = ix.discm_ident();
            let mut variant_expr = quote! {
                Self::#variant_ident
            };
            if ix.has_ix_args() {
                let ix_args_ident = ix.ix_args_ident();
                variant_expr.extend(quote! {
                    (#ix_args_ident::deserialize(&mut reader)?)
                })
            }
            quote! {
                #discm_ident => Ok(#variant_expr)
            }
        });

        quote! {
            #[derive(Clone, Debug, PartialEq)]
            pub enum #program_ix_enum_ident {
                #(#program_ix_enum_variants),*
            }

            impl #program_ix_enum_ident {
                pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
                    if buf.is_empty() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Instruction data is empty",
                        ));
                    }
                    let maybe_discm = buf[0];
                    let mut reader = &buf[1..];
                    match maybe_discm {
                        #(#deserialize_variant_match_arms),*,
                        _ => Err(
                            std::io::Error::new(
                                std::io::ErrorKind::Other, format!("discm {} not found", maybe_discm)
                            )
                        ),
                    }
                }

                pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
                    match self {
                        #(#serialize_variant_match_arms),*,
                    }
                }

                pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
                    let mut data = Vec::new();
                    self.serialize(&mut data)?;
                    Ok(data)
                }
            }
        }
    }

    fn gen_individual_instruction_file(&self, instruction: &NamedInstruction) -> TokenStream {
        let base_imports = quote! {
            use super::*;
        };
        
        // Generate documentation comments for the instruction if present
        let doc_comments = if let Some(docs) = &instruction.docs {
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
        
        // Generate instruction content with context information
        let instruction_content = self.gen_instruction_content_with_context(instruction);
        let ix_args_methods = self.gen_ix_args_methods(instruction);
        
        quote! {
            #base_imports
            #doc_comments
            #instruction_content
            #ix_args_methods
        }
    }

    fn gen_instruction_content_with_context(&self, instruction: &NamedInstruction) -> TokenStream {
        // Find the instruction index in the array for discriminator generation
        let instruction_index = self.instructions
            .iter()
            .position(|ix| ix.name == instruction.name)
            .unwrap_or(0);

        // Generate instruction content with context
        let mut tokens = TokenStream::new();
        
        // Process accounts
        let accounts = instruction
            .accounts
            .as_ref()
            .map_or(Vec::new(), |v| to_ix_accounts(v));
        let n_accounts = accounts.len();

        // Check for duplicate accounts (same as ToTokens implementation)
        let crate::utils::UniqueByReportDupsResult { duplicates, .. } =
            crate::utils::unique_by_report_dups(accounts.iter(), |acc| acc.name.clone());

        if !duplicates.is_empty() {
            log::error!(
                "Found duplicate accounts for instruction {}: {}",
                &instruction.name,
                duplicates.iter().map(|acc| acc.name.as_str()).collect::<Vec<_>>().join(", ")
            );
            panic!();
        }

        // Generate all the instruction components
        instruction.write_accounts_len(&mut tokens, n_accounts);
        instruction.write_accounts_struct(&mut tokens, &accounts);
        instruction.write_keys_struct(&mut tokens, &accounts);
        instruction.write_from_accounts_for_keys(&mut tokens, &accounts);
        instruction.write_from_keys_for_meta_arr(&mut tokens, &accounts);
        instruction.write_from_pubkey_arr_for_keys(&mut tokens, &accounts);
        instruction.write_from_string_arr_for_keys(&mut tokens, &accounts);
        instruction.write_from_accounts_for_account_info_arr(&mut tokens, &accounts);
        instruction.write_from_account_info_arr_for_accounts(&mut tokens, &accounts);

        // Generate discriminator with context
        instruction.write_discm_with_context(&mut tokens, self.is_anchor_contract, instruction_index);
        
        // Generate instruction arguments struct with context (internal discriminator)
        instruction.write_ix_args_struct_with_context(&mut tokens, self.is_anchor_contract, instruction_index);
        instruction.write_ix_data_struct(&mut tokens);
        instruction.write_from_ix_args_for_ix_data(&mut tokens);
        instruction.write_ix_data_impl_with_context(&mut tokens, self.is_anchor_contract, instruction_index);

        instruction.write_ix_fn(&mut tokens);
        instruction.write_invoke_fn(&mut tokens);
        instruction.write_invoke_signed_fn(&mut tokens);

        instruction.write_verify_account_keys_fn(&mut tokens, &accounts);
        
        // Generate PDA-related code
        instruction.write_pda_seeds_constants(&mut tokens, &accounts);
        instruction.write_pda_derivation_functions(&mut tokens, &accounts);
        instruction.write_fixed_address_constants(&mut tokens, &accounts);

        tokens
    }

    fn gen_ix_args_methods(&self, instruction: &NamedInstruction) -> TokenStream {
        if !instruction.has_ix_args() {
            return quote! {};
        }

        let ix_args_ident = instruction.ix_args_ident();
        let program_ix_enum_ident = format_ident!("{}ProgramIx", self.program_name.to_pascal_case());
        let variant_ident = format_ident!("{}", instruction.name.to_pascal_case());

        quote! {
            impl #ix_args_ident {
                pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
                    BorshSerialize::serialize(self, &mut writer)
                }
                
                pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
                    BorshDeserialize::deserialize(&mut &buf[..])
                }
                
                pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
                    borsh::to_vec(self)
                }
            }

            impl From<#ix_args_ident> for #program_ix_enum_ident {
                fn from(args: #ix_args_ident) -> Self {
                    Self::#variant_ident(args)
                }
            }
        }
    }
}
