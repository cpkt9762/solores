// TODO: determine borsh version for more efficient implementations of deserialize_reader
// that makes use of ix_args' deserialize_reader method if available

use heck::{ToPascalCase, ToShoutySnakeCase, ToSnakeCase};
use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use syn::{LitBool, LitInt};
use convert_case::{Case, Casing};

use crate::{
    idl_format::anchor::typedefs::TypedefField,
    utils::{unique_by_report_dups, UniqueByReportDupsResult},
};

// PDA support data structures
#[derive(Clone, Deserialize, Debug)]
pub struct PdaConfig {
    pub seeds: Vec<PdaSeed>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum PdaSeed {
    #[serde(rename = "const")]
    Const { value: Vec<u8> },
    #[serde(rename = "account")]
    Account { path: String },
    #[serde(rename = "arg")]
    Arg { path: String },
}

#[derive(Deserialize, Debug)]
pub struct NamedInstruction {
    pub name: String,
    #[serde(default)]
    pub docs: Option<Vec<String>>,
    pub discriminator: Option<Vec<u8>>,  // 8-byte discriminator for Anchor contracts
    pub accounts: Option<Vec<IxAccountEntry>>,
    pub args: Option<Vec<TypedefField>>,
}

impl NamedInstruction {
    pub fn ix_args_ident(&self) -> Ident {
        format_ident!("{}IxArgs", self.name.to_pascal_case())
    }

    pub fn ix_data_ident(&self) -> Ident {
        format_ident!("{}IxData", self.name.to_pascal_case())
    }

    pub fn ix_fn_ident(&self) -> Ident {
        format_ident!("{}_ix", self.name.to_snake_case())
    }

    pub fn ix_fn_with_program_id_ident(&self) -> Ident {
        format_ident!("{}_ix_with_program_id", self.name.to_snake_case())
    }

    pub fn discm_ident(&self) -> Ident {
        format_ident!("{}_IX_DISCM", &self.name.to_shouty_snake_case())
    }

    pub fn accounts_ident(&self) -> Ident {
        format_ident!("{}Accounts", self.name.to_pascal_case())
    }

    pub fn keys_ident(&self) -> Ident {
        format_ident!("{}Keys", self.name.to_pascal_case())
    }

    pub fn accounts_len_ident(&self) -> Ident {
        format_ident!("{}_IX_ACCOUNTS_LEN", self.name.to_shouty_snake_case())
    }

    pub fn has_ix_args(&self) -> bool {
        let args = match &self.args {
            Some(a) => a,
            None => return false,
        };
        !args.is_empty()
    }

    pub fn has_accounts(&self) -> bool {
        let accounts = match &self.accounts {
            Some(a) => a,
            None => return false,
        };
        !accounts.is_empty()
    }

    pub fn args_has_defined_type(&self) -> bool {
        let args = if !self.has_ix_args() {
            return false;
        } else {
            self.args.as_ref().unwrap()
        };
        args.iter().map(|a| a.r#type.is_or_has_defined()).any(|b| b)
    }

    pub fn args_has_pubkeys(&self) -> bool {
        let args = if !self.has_ix_args() {
            return false;
        } else {
            self.args.as_ref().unwrap()
        };
        args.iter().map(|a| a.r#type.is_or_has_pubkey()).any(|b| b)
    }

    /// Check if this instruction has discriminator (indicates Anchor contract)
    pub fn has_discriminator(&self) -> bool {
        self.discriminator.is_some()
    }


    /// export accounts_len as const
    pub fn write_accounts_len(&self, tokens: &mut TokenStream, accounts_len: usize) {
        if !self.has_accounts() {
            return;
        }
        let accounts_len_ident = self.accounts_len_ident();
        let n_accounts_lit = LitInt::new(&accounts_len.to_string(), Span::call_site());
        tokens.extend(quote! {
            pub const #accounts_len_ident: usize = #n_accounts_lit;
        });
    }

    pub fn write_accounts_struct(&self, tokens: &mut TokenStream, accounts: &[IxAccount]) {
        if !self.has_accounts() {
            return;
        }
        let accounts_ident = self.accounts_ident();
        let accounts_fields = accounts.iter().map(|acc| {
            let account_name = format_ident!("{}", &acc.name.to_snake_case());
            
            // Generate field documentation from IDL account docs
            let mut doc_lines = Vec::new();
            
            // Add account docs if present
            if let Some(docs) = &acc.docs {
                doc_lines.extend(docs.iter().filter(|doc| !doc.trim().is_empty()).cloned());
            }
            
            // Add relations info if present
            if let Some(relations) = &acc.relations {
                if !relations.is_empty() {
                    let relations_str = format!("Relations: {}", relations.join(", "));
                    doc_lines.push(relations_str);
                }
            }
            
            let field_docs = if !doc_lines.is_empty() {
                let doc_tokens: Vec<TokenStream> = doc_lines
                    .iter()
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
                #field_docs
                pub #account_name: &'me AccountInfo<'info>
            }
        });
        
        // Generate documentation comments for the accounts struct
        let doc_str = format!("Accounts struct for the {} instruction", self.name);
        let doc_comments = quote! { #[doc = #doc_str] };
        
        tokens.extend(quote! {
            #doc_comments
            #[derive(Copy, Clone, Debug)]
            pub struct #accounts_ident<'me, 'info> {
                #(#accounts_fields),*
            }
        });
    }

    pub fn write_keys_struct(&self, tokens: &mut TokenStream, accounts: &[IxAccount]) {
        if !self.has_accounts() {
            return;
        }
        let keys_ident = self.keys_ident();
        let keys_fields = accounts.iter().map(|acc| {
            let account_ident = format_ident!("{}", &acc.name.to_snake_case());
            
            // Generate field documentation from IDL account docs
            let mut doc_lines = Vec::new();
            
            // Add account docs if present
            if let Some(docs) = &acc.docs {
                doc_lines.extend(docs.iter().filter(|doc| !doc.trim().is_empty()).cloned());
            }
            
            // Add relations info if present
            if let Some(relations) = &acc.relations {
                if !relations.is_empty() {
                    let relations_str = format!("Relations: {}", relations.join(", "));
                    doc_lines.push(relations_str);
                }
            }
            
            let field_docs = if !doc_lines.is_empty() {
                let doc_tokens: Vec<TokenStream> = doc_lines
                    .iter()
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
                #field_docs
                pub #account_ident: Pubkey
            }
        });
        
        // Generate documentation comments for the keys struct
        let doc_str = format!("Public keys struct for the {} instruction", self.name);
        let doc_comments = quote! { #[doc = #doc_str] };
        
        tokens.extend(quote! {
            #doc_comments
            #[derive(Copy, Clone, Debug, PartialEq)]
            pub struct #keys_ident {
                #(#keys_fields),*
            }
        });
    }

    /// From<XAccounts> for XKeys
    pub fn write_from_accounts_for_keys(&self, tokens: &mut TokenStream, accounts: &[IxAccount]) {
        if !self.has_accounts() {
            return;
        }
        let accounts_ident = self.accounts_ident();
        let keys_ident = self.keys_ident();
        let from_keys_fields = accounts.iter().map(|acc| {
            let account_ident = format_ident!("{}", &acc.name.to_snake_case());
            quote! {
                #account_ident: *accounts.#account_ident.key
            }
        });
        tokens.extend(quote! {
            impl From<#accounts_ident<'_, '_>> for #keys_ident {
                fn from(accounts: #accounts_ident) -> Self {
                    Self {
                        #(#from_keys_fields),*
                    }
                }
            }
        });
    }

    /// From <&XKeys> for [AccountMeta]
    pub fn write_from_keys_for_meta_arr(&self, tokens: &mut TokenStream, accounts: &[IxAccount]) {
        if !self.has_accounts() {
            return;
        }
        let keys_ident = self.keys_ident();
        let accounts_len_ident = self.accounts_len_ident();
        let from_keys_meta = accounts.iter().map(|acc| acc.to_keys_account_meta_tokens());
        tokens.extend(quote! {
            impl From<#keys_ident> for [AccountMeta; #accounts_len_ident] {
                fn from(keys: #keys_ident) -> Self {
                    [
                        #(#from_keys_meta),*
                    ]
                }
            }
        });
    }

    /// From <[Pubkey]> for XKeys
    pub fn write_from_pubkey_arr_for_keys(&self, tokens: &mut TokenStream, accounts: &[IxAccount]) {
        if !self.has_accounts() {
            return;
        }
        let accounts_len_ident = self.accounts_len_ident();
        let keys_ident = self.keys_ident();
        let from_pubkey_arr_fields = accounts.iter().enumerate().map(|(i, acc)| {
            let account_ident = format_ident!("{}", &acc.name.to_snake_case());
            let index_lit = LitInt::new(&i.to_string(), Span::call_site());
            quote! {
                #account_ident: pubkeys[#index_lit]
            }
        });
        tokens.extend(quote! {
            impl From<[Pubkey; #accounts_len_ident]> for #keys_ident {
                fn from(pubkeys: [Pubkey; #accounts_len_ident]) -> Self {
                    Self {
                        #(#from_pubkey_arr_fields),*
                    }
                }
            }
        });
    }
    /// From <&[String]> for XKeys
    pub fn write_from_string_arr_for_keys(&self, tokens: &mut TokenStream, accounts: &[IxAccount]) {
        if !self.has_accounts() {
            return;
        }
        let keys_ident = self.keys_ident();
        let from_pubkey_arr_fields = accounts.iter().enumerate().map(|(i, acc)| {
            let account_ident = format_ident!("{}", &acc.name.to_snake_case());
            let index_lit = LitInt::new(&i.to_string(), Span::call_site());
            quote! {
                #account_ident: pubkeys[#index_lit].parse().unwrap()
            }
        });
        tokens.extend(quote! {
            impl From<&[String]> for #keys_ident {
                fn from(pubkeys: &[String]) -> Self {
                    Self {
                        #(#from_pubkey_arr_fields),*
                    }
                }
            }
        });
    }
    /// From <XAccounts> for [AccountInfo]
    pub fn write_from_accounts_for_account_info_arr(
        &self,
        tokens: &mut TokenStream,
        accounts: &[IxAccount],
    ) {
        if !self.has_accounts() {
            return;
        }
        let accounts_ident = self.accounts_ident();
        let accounts_len_ident = self.accounts_len_ident();
        let account_info_clone = accounts.iter().map(|acc| {
            let account_ident = format_ident!("{}", &acc.name.to_snake_case());
            quote! {
               accounts.#account_ident.clone()
            }
        });
        tokens.extend(quote! {
            impl<'info> From<#accounts_ident<'_, 'info>> for [AccountInfo<'info>; #accounts_len_ident] {
                fn from(accounts: #accounts_ident<'_, 'info>) -> Self {
                    [
                        #(#account_info_clone),*
                    ]
                }
            }
        });
    }

    /// From <&[AccountInfo]> for XAccounts
    pub fn write_from_account_info_arr_for_accounts(
        &self,
        tokens: &mut TokenStream,
        accounts: &[IxAccount],
    ) {
        if !self.has_accounts() {
            return;
        }
        let accounts_ident = self.accounts_ident();
        let accounts_len_ident = self.accounts_len_ident();
        let from_account_info_fields = accounts.iter().enumerate().map(|(i, acc)| {
            let account_ident = format_ident!("{}", &acc.name.to_snake_case());
            let index_lit = LitInt::new(&i.to_string(), Span::call_site());
            quote! {
               #account_ident: &arr[#index_lit]
            }
        });
        tokens.extend(quote! {
            impl<'me, 'info> From<&'me [AccountInfo<'info>; #accounts_len_ident]> for #accounts_ident<'me, 'info> {
                fn from(arr: &'me [AccountInfo<'info>; #accounts_len_ident]) -> Self {
                    Self {
                        #(#from_account_info_fields),*
                    }
                }
            }
        });
    }

    pub fn write_ix_args_struct(&self, tokens: &mut TokenStream) {
        self.write_ix_args_struct_with_context(tokens, true, 0);
    }

    pub fn write_ix_args_struct_with_context(&self, tokens: &mut TokenStream, is_anchor: bool, instruction_index: usize) {
        let args = if !self.has_ix_args() {
            return;
        } else {
            self.args.as_ref().unwrap()
        };
        let ix_args_ident = self.ix_args_ident();
        let args_fields = args;
        
        // Generate documentation comments for the ix args struct
        let doc_comments = if let Some(docs) = &self.docs {
            let doc_tokens: Vec<TokenStream> = docs
                .iter()
                .filter(|doc| !doc.trim().is_empty())
                .map(|doc| {
                    let doc_str = doc.trim();
                    quote! { #[doc = #doc_str] }
                })
                .collect();
            if doc_tokens.is_empty() {
                let doc_str = format!("Arguments for the {} instruction", self.name);
                quote! { #[doc = #doc_str] }
            } else {
                quote! { #(#doc_tokens)* }
            }
        } else {
            let doc_str = format!("Arguments for the {} instruction", self.name);
            quote! { #[doc = #doc_str] }
        };
        
        // Add discriminator field based on contract type
        let discriminator_field = if is_anchor {
            quote! { pub discriminator: [u8; 8], }
        } else {
            quote! { pub discriminator: u8, }
        };
        
        tokens.extend(quote! {
            #doc_comments
            #[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            pub struct #ix_args_ident {
                #discriminator_field
                #(#args_fields),*
            }
        });
        
        // Generate custom Default implementation with correct discriminator value
        self.write_ix_args_default_impl(tokens, is_anchor, instruction_index);
    }

    pub fn write_ix_args_default_impl(&self, tokens: &mut TokenStream, is_anchor: bool, instruction_index: usize) {
        if !self.has_ix_args() {
            return;
        }

        let ix_args_ident = self.ix_args_ident();
        let args = self.args.as_ref().unwrap();
        
        // Generate discriminator value
        let discriminator_value = if is_anchor {
            if let Some(ref discriminator) = self.discriminator {
                // Use discriminator from IDL if available
                let discm_bytes: Vec<proc_macro2::TokenStream> = discriminator.iter()
                    .map(|&byte| quote! { #byte })
                    .collect();
                quote! { [#(#discm_bytes),*] }
            } else {
                // Fallback: generate from instruction name hash
                quote! { [0u8; 8] } // This should be computed properly
            }
        } else {
            // Non-Anchor contract: use discriminator from IDL if available, otherwise use instruction index
            let discm_byte = if let Some(ref discriminator) = self.discriminator {
                if discriminator.len() == 1 {
                    discriminator[0] as u8
                } else {
                    log::warn!("Expected 1-byte discriminator for non-Anchor contract, got {} bytes, using index", discriminator.len());
                    instruction_index as u8
                }
            } else {
                instruction_index as u8
            };
            quote! { #discm_byte }
        };
        
        // Generate default values for all fields
        let field_defaults: Vec<proc_macro2::TokenStream> = args.iter().map(|arg| {
            // Convert field name to snake_case to match generated struct fields
            let snake_case_name = {
                let mut result = String::new();
                let mut leading_underscores = String::new();
                
                // Collect leading underscores
                if arg.name.starts_with('_') {
                    leading_underscores.push('_');
                }
                
                // Special handling for X64, X128 patterns
                let rest = &arg.name[leading_underscores.len()..];
                let converted = if rest.contains("X64") || rest.contains("X128") || rest.contains("X256") {
                    // First convert to snake_case, then fix the X64/X128/X256 patterns
                    let mut temp = rest.to_case(Case::Snake);
                    // Fix patterns like "_x_64" -> "_x64"
                    temp = temp.replace("_x_64", "_x64");
                    temp = temp.replace("_x_128", "_x128");
                    temp = temp.replace("_x_256", "_x256");
                    temp
                } else {
                    rest.to_case(Case::Snake)
                };
                
                // Combine leading underscores and converted part
                result.push_str(&leading_underscores);
                result.push_str(&converted);
                result
            };
            let field_name = syn::Ident::new(&snake_case_name, proc_macro2::Span::call_site());
            quote! { #field_name: Default::default() }
        }).collect();
        
        tokens.extend(quote! {
            impl Default for #ix_args_ident {
                fn default() -> Self {
                    Self {
                        discriminator: #discriminator_value,
                        #(#field_defaults),*
                    }
                }
            }
        });
    }

    pub fn write_discm(&self, tokens: &mut TokenStream) {
        self.write_discm_with_context(tokens, true, 0)
    }

    pub fn write_discm_with_context(&self, tokens: &mut TokenStream, is_anchor: bool, instruction_index: usize) {
        let discm_ident = self.discm_ident();
        
        if is_anchor {
            // Generate 8-byte discriminator for Anchor contracts
            if let Some(ref discriminator) = self.discriminator {
                // Use discriminator from IDL if available
                let discm_value_tokens: TokenStream = format!("{:?}", discriminator).parse().unwrap();
                tokens.extend(quote! {
                    pub const #discm_ident: [u8; 8] = #discm_value_tokens;
                });
            } else {
                // Fallback to SHA256 hash method for Anchor contracts without discriminator field
                let discm = <[u8; 8]>::try_from(
                    &Sha256::digest(format!("global:{}", self.name.to_snake_case()).as_bytes()).as_slice()
                        [..8],
                )
                .unwrap();
                let discm_value_tokens: TokenStream = format!("{:?}", discm).parse().unwrap();
                tokens.extend(quote! {
                    pub const #discm_ident: [u8; 8] = #discm_value_tokens;
                });
            }
        } else {
            // Generate 1-byte discriminator for non-Anchor contracts
            let discm_value = instruction_index as u8;
            tokens.extend(quote! {
                pub const #discm_ident: u8 = #discm_value;
            });
        }
    }

    pub fn write_ix_data_struct(&self, tokens: &mut TokenStream) {
        let ix_data_ident = self.ix_data_ident();
        let struct_decl = if self.has_ix_args() {
            let ix_args_ident = self.ix_args_ident();
            quote! { pub struct #ix_data_ident(pub #ix_args_ident); }
        } else {
            quote! { pub struct #ix_data_ident; }
        };

        tokens.extend(quote! {
            #[derive(Clone, Debug, PartialEq)]
            #struct_decl
        });
    }

    pub fn write_from_ix_args_for_ix_data(&self, tokens: &mut TokenStream) {
        if !self.has_ix_args() {
            return;
        }
        let ix_data_ident = self.ix_data_ident();
        let ix_args_ident = self.ix_args_ident();
        tokens.extend(quote! {
            impl From<#ix_args_ident> for #ix_data_ident {
                fn from(args: #ix_args_ident) -> Self {
                    Self(args)
                }
            }
        });
    }

    pub fn write_ix_data_impl(&self, tokens: &mut TokenStream) {
        self.write_ix_data_impl_with_context(tokens, true, 0)
    }

    pub fn write_ix_data_impl_with_context(&self, tokens: &mut TokenStream, is_anchor: bool, _instruction_index: usize) {
        let discm_ident = self.discm_ident();
        let ix_data_ident = self.ix_data_ident();
        
        let (deserialize_body, serialize_body) = if is_anchor {
            // 8-byte discriminator logic for Anchor contracts
            let mut deserialize_body = quote! {
                let mut reader = buf;
                let mut maybe_discm = [0u8; 8];
                reader.read_exact(&mut maybe_discm)?;
                if maybe_discm != #discm_ident {
                    return Err(
                        std::io::Error::new(
                            std::io::ErrorKind::Other, format!("discm does not match. Expected: {:?}. Received: {:?}", #discm_ident, maybe_discm)
                        )
                    );
                }
            };
            if self.has_ix_args() {
                let ix_args_ident = self.ix_args_ident();
                deserialize_body.extend(quote! {
                    Ok(Self(#ix_args_ident::deserialize(&mut reader)?))
                })
            } else {
                deserialize_body.extend(quote! {
                    Ok(Self)
                })
            }
            
            let serialize_body = if self.has_ix_args() {
                quote! {
                    writer.write_all(&#discm_ident)?;
                    self.0.serialize(&mut writer)
                }
            } else {
                quote! {
                    writer.write_all(&#discm_ident)
                }
            };
            
            (deserialize_body, serialize_body)
        } else {
            // 1-byte discriminator logic for non-Anchor contracts
            let mut deserialize_body = quote! {
                if buf.is_empty() {
                    return Err(
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Instruction data is empty"
                        )
                    );
                }
                let maybe_discm = buf[0];
                if maybe_discm != #discm_ident {
                    return Err(
                        std::io::Error::new(
                            std::io::ErrorKind::Other, format!("discm does not match. Expected: {}. Received: {}", #discm_ident, maybe_discm)
                        )
                    );
                }
            };
            if self.has_ix_args() {
                let ix_args_ident = self.ix_args_ident();
                deserialize_body.extend(quote! {
                    let mut reader = &buf[1..];
                    Ok(Self(#ix_args_ident::deserialize(&mut reader)?))
                })
            } else {
                deserialize_body.extend(quote! {
                    Ok(Self)
                })
            }
            
            let serialize_body = if self.has_ix_args() {
                quote! {
                    writer.write_all(&[#discm_ident])?;
                    self.0.serialize(&mut writer)
                }
            } else {
                quote! {
                    writer.write_all(&[#discm_ident])
                }
            };
            
            (deserialize_body, serialize_body)
        };

        tokens.extend(quote! {
            impl #ix_data_ident {
                pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
                    #deserialize_body
                }

                pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
                    #serialize_body
                }

                pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
                    let mut data = Vec::new();
                    self.serialize(&mut data)?;
                    Ok(data)
                }
            }
        });
    }

    /// _ix()
    /// _ix_with_program_id()
    pub fn write_ix_fn(&self, tokens: &mut TokenStream) {
        let ix_fn_ident = self.ix_fn_ident();
        let ix_with_program_id_fn_ident = self.ix_fn_with_program_id_ident();
        let keys_ident = self.keys_ident();
        let ix_args_ident = self.ix_args_ident();
        let accounts_len_ident = self.accounts_len_ident();
        let ix_data_ident = self.ix_data_ident();

        let mut fn_params = quote! {};
        let mut fn_args = quote! {};
        if self.has_accounts() {
            fn_params.extend(quote! { keys: #keys_ident, });
            fn_args.extend(quote! { keys, });
        }
        if self.has_ix_args() {
            fn_params.extend(quote! { args: #ix_args_ident, });
            fn_args.extend(quote! { args, });
        }

        let (mut fn_body, accounts_expr) = if self.has_accounts() {
            (
                quote! {
                    let metas: [AccountMeta; #accounts_len_ident] = keys.into();
                },
                quote! {
                    Vec::from(metas)
                },
            )
        } else {
            (
                quote! {},
                quote! {
                    Vec::new()
                },
            )
        };
        if self.has_ix_args() {
            fn_body.extend(quote! {
                let data: #ix_data_ident = args.into();
            })
        }
        let data_expr = if self.has_ix_args() {
            quote! { data.try_to_vec()? }
        } else {
            quote! { #ix_data_ident.try_to_vec()? }
        };

        tokens.extend(quote! {
            pub fn #ix_with_program_id_fn_ident(program_id: Pubkey, #fn_params) -> std::io::Result<Instruction> {
                #fn_body
                Ok(Instruction {
                    program_id,
                    accounts: #accounts_expr,
                    data: #data_expr,
                })
            }

            pub fn #ix_fn_ident(#fn_params) -> std::io::Result<Instruction> {
                #ix_with_program_id_fn_ident(crate::ID, #fn_args)
            }
        });
    }

    fn invoke_fn_params_prefix(&self) -> TokenStream {
        let accounts_ident = self.accounts_ident();
        let ix_args_ident = self.ix_args_ident();
        let mut fn_params = quote! {};
        if self.has_accounts() {
            fn_params.extend(quote! { accounts: #accounts_ident<'_, '_>, });
        }
        if self.has_ix_args() {
            fn_params.extend(quote! { args: #ix_args_ident, })
        }
        fn_params
    }

    fn invoke_fn_args_prefix(&self) -> TokenStream {
        let mut fn_args = quote! {};
        if self.has_accounts() {
            fn_args.extend(quote! { accounts, });
        }
        if self.has_ix_args() {
            fn_args.extend(quote! { args, })
        }
        fn_args
    }

    fn ix_call_assign(&self) -> TokenStream {
        let ix_with_program_id_fn_ident = self.ix_fn_with_program_id_ident();
        let keys_ident = self.keys_ident();
        let mut res = quote! {};
        let mut args = quote! {};
        if self.has_accounts() {
            res.extend(quote! {
                let keys: #keys_ident = accounts.into();
            });
            args.extend(quote! { keys, });
        }
        if self.has_ix_args() {
            args.extend(quote! { args });
        }
        res.extend(quote! {
            let ix = #ix_with_program_id_fn_ident(program_id, #args)?;
        });
        res
    }

    /// _invoke()
    /// _invoke_with_program_id()
    pub fn write_invoke_fn(&self, tokens: &mut TokenStream) {
        let invoke_fn_ident = format_ident!("{}_invoke", self.name.to_snake_case());
        let invoke_with_program_id_fn_ident =
            format_ident!("{}_invoke_with_program_id", self.name.to_snake_case());
        let fn_params = self.invoke_fn_params_prefix();
        let fn_args = self.invoke_fn_args_prefix();
        let call_assign = self.ix_call_assign();
        let invoke = if self.has_accounts() {
            quote! {
                invoke_instruction(&ix, accounts)
            }
        } else {
            quote! {
                invoke(&ix, &[])
            }
        };
        tokens.extend(quote! {
            pub fn #invoke_with_program_id_fn_ident(program_id: Pubkey, #fn_params) -> ProgramResult {
                #call_assign
                #invoke
            }

            pub fn #invoke_fn_ident(#fn_params) -> ProgramResult {
                #invoke_with_program_id_fn_ident(crate::ID, #fn_args)
            }
        });
    }

    /// _invoke_signed()
    /// _invoke_signed_with_program_id()
    pub fn write_invoke_signed_fn(&self, tokens: &mut TokenStream) {
        let invoke_signed_fn_ident = format_ident!("{}_invoke_signed", self.name.to_snake_case());
        let invoke_signed_with_program_id_fn_ident = format_ident!(
            "{}_invoke_signed_with_program_id",
            self.name.to_snake_case()
        );
        let mut fn_params = self.invoke_fn_params_prefix();
        fn_params.extend(quote! { seeds: &[&[&[u8]]], });
        let mut fn_args = self.invoke_fn_args_prefix();
        fn_args.extend(quote! { seeds, });
        let call_assign = self.ix_call_assign();
        let invoke = if self.has_accounts() {
            quote! {
                invoke_instruction_signed(&ix, accounts, seeds)
            }
        } else {
            quote! {
                invoke_signed(&ix, &[], seeds)
            }
        };
        tokens.extend(quote! {
            pub fn #invoke_signed_with_program_id_fn_ident(program_id: Pubkey, #fn_params) -> ProgramResult {
                #call_assign
                #invoke
            }

            pub fn #invoke_signed_fn_ident(#fn_params) -> ProgramResult {
                #invoke_signed_with_program_id_fn_ident(crate::ID, #fn_args)
            }
        });
    }

    /// _verify_account_keys()
    pub fn write_verify_account_keys_fn(&self, tokens: &mut TokenStream, accounts: &[IxAccount]) {
        if !self.has_accounts() {
            return;
        }
        let verify_account_keys_fn_ident =
            format_ident!("{}_verify_account_keys", self.name.to_snake_case());
        let accounts_ident = self.accounts_ident();
        let keys_ident = self.keys_ident();
        let key_tups = accounts.iter().map(IxAccount::to_verify_account_keys_tuple);
        // edge-case of accounts and keys being empty
        let pubkeys_loop_check = if accounts.is_empty() {
            quote! {}
        } else {
            quote! {
                for (actual, expected) in [
                    #(#key_tups),*
                ] {
                    if actual != expected {
                        return Err((actual, expected));
                    }
                }
            }
        };
        tokens.extend(quote! {
            pub fn #verify_account_keys_fn_ident(
                accounts: #accounts_ident<'_, '_>,
                keys: #keys_ident
            ) -> Result<(), (Pubkey, Pubkey)> {
                #pubkeys_loop_check
                Ok(())
            }
        });
    }


    /// Generate PDA seed constants for accounts with PDA configuration
    pub fn write_pda_seeds_constants(&self, tokens: &mut TokenStream, accounts: &[IxAccount]) {
        for account in accounts {
            if let Some(pda_config) = account.get_pda_config() {
                // Generate seed constants for this PDA
                for (seed_idx, seed) in pda_config.seeds.iter().enumerate() {
                    if let PdaSeed::Const { value } = seed {
                        let constant_name = format!("{}_SEED", account.name.to_shouty_snake_case());
                        let constant_name = if pda_config.seeds.len() > 1 {
                            format!("{}_{}", constant_name, seed_idx)
                        } else {
                            constant_name
                        };
                        let constant_ident = format_ident!("{}", constant_name);
                        
                        // Convert byte array to readable string comment if possible
                        let byte_string = String::from_utf8(value.clone())
                            .map(|s| format!(": \"{}\"", s))
                            .unwrap_or_default();
                        
                        let doc_comment = format!("{} seed for {} PDA{}", 
                            constant_name, account.name, byte_string);
                        
                        tokens.extend(quote! {
                            #[doc = #doc_comment]
                            pub const #constant_ident: &[u8] = &[#(#value),*];
                        });
                    }
                }
            }
        }
    }

    /// Generate PDA derivation functions
    pub fn write_pda_derivation_functions(&self, tokens: &mut TokenStream, accounts: &[IxAccount]) {
        let keys_ident = self.keys_ident();
        let mut derivation_functions = Vec::new();
        
        for account in accounts {
            if let Some(pda_config) = account.get_pda_config() {
                let function_name = format!("derive_{}_pda", account.name.to_snake_case());
                let function_ident = format_ident!("{}", function_name);
                let account_name = &account.name;
                
                // Build seeds for derivation
                let mut seed_exprs = Vec::new();
                let mut function_params = vec![quote! { program_id: &Pubkey }];
                
                for (seed_idx, seed) in pda_config.seeds.iter().enumerate() {
                    match seed {
                        PdaSeed::Const { .. } => {
                            let constant_name = format!("{}_SEED", account.name.to_shouty_snake_case());
                            let constant_name = if pda_config.seeds.len() > 1 {
                                format!("{}_{}", constant_name, seed_idx)
                            } else {
                                constant_name
                            };
                            let constant_ident = format_ident!("{}", constant_name);
                            seed_exprs.push(quote! { #constant_ident });
                        }
                        PdaSeed::Account { path } => {
                            let param_name = format_ident!("{}", path.to_snake_case());
                            function_params.push(quote! { #param_name: &Pubkey });
                            seed_exprs.push(quote! { #param_name.as_ref() });
                        }
                        PdaSeed::Arg { path } => {
                            let param_name = format_ident!("{}", path.to_snake_case());
                            // For now, assume it's bytes - this could be enhanced based on the actual type
                            function_params.push(quote! { #param_name: &[u8] });
                            seed_exprs.push(quote! { #param_name });
                        }
                    }
                }

                let doc_comment = format!("Derive the {} PDA", account_name);
                
                derivation_functions.push(quote! {
                    #[doc = #doc_comment]
                    pub fn #function_ident(#(#function_params),*) -> (Pubkey, u8) {
                        Pubkey::find_program_address(&[#(#seed_exprs),*], program_id)
                    }
                });
            }
        }
        
        if !derivation_functions.is_empty() {
            tokens.extend(quote! {
                impl #keys_ident {
                    #(#derivation_functions)*
                }
            });
        }
    }

    /// Generate constants for fixed address accounts
    pub fn write_fixed_address_constants(&self, tokens: &mut TokenStream, accounts: &[IxAccount]) {
        for account in accounts {
            if let Some(address) = account.get_fixed_address() {
                let constant_name = format!("{}_ADDRESS", account.name.to_shouty_snake_case());
                let constant_ident = format_ident!("{}", constant_name);
                let doc_comment = format!("{} program address", account.name);
                
                tokens.extend(quote! {
                    #[doc = #doc_comment]
                    pub const #constant_ident: &str = #address;
                });
            }
        }
    }
}

impl NamedInstruction {
    /// Generate instruction tokens with context information
    pub fn to_token_stream_with_context(&self, is_anchor: bool, instruction_index: usize) -> TokenStream {
        let mut tokens = TokenStream::new();
        
        let accounts = self
            .accounts
            .as_ref()
            .map_or(Vec::new(), |v| to_ix_accounts(v));
        let n_accounts = accounts.len();

        let UniqueByReportDupsResult { duplicates, .. } =
            unique_by_report_dups(accounts.iter(), |acc| acc.name.clone());

        if !duplicates.is_empty() {
            log::error!(
                "Found duplicate accounts for instruction {}: {}",
                &self.name,
                duplicates.iter().map(|acc| &acc.name).format(", ")
            );
            panic!();
        }

        self.write_accounts_len(&mut tokens, n_accounts);
        self.write_accounts_struct(&mut tokens, &accounts);
        self.write_keys_struct(&mut tokens, &accounts);
        self.write_from_accounts_for_keys(&mut tokens, &accounts);
        self.write_from_keys_for_meta_arr(&mut tokens, &accounts);
        self.write_from_pubkey_arr_for_keys(&mut tokens, &accounts);
        self.write_from_string_arr_for_keys(&mut tokens, &accounts);
        self.write_from_accounts_for_account_info_arr(&mut tokens, &accounts);
        self.write_from_account_info_arr_for_accounts(&mut tokens, &accounts);

        // Use context-aware methods for discriminator and data impl
        self.write_discm_with_context(&mut tokens, is_anchor, instruction_index);
        self.write_ix_args_struct(&mut tokens);
        self.write_ix_data_struct(&mut tokens);
        self.write_from_ix_args_for_ix_data(&mut tokens);
        self.write_ix_data_impl_with_context(&mut tokens, is_anchor, instruction_index);

        self.write_ix_fn(&mut tokens);
        self.write_invoke_fn(&mut tokens);
        self.write_invoke_signed_fn(&mut tokens);

        self.write_verify_account_keys_fn(&mut tokens, &accounts);
        
        // Generate PDA-related code
        self.write_pda_seeds_constants(&mut tokens, &accounts);
        self.write_pda_derivation_functions(&mut tokens, &accounts);
        self.write_fixed_address_constants(&mut tokens, &accounts);

        tokens
    }
}

impl ToTokens for NamedInstruction {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let accounts = self
            .accounts
            .as_ref()
            .map_or(Vec::new(), |v| to_ix_accounts(v));
        let n_accounts = accounts.len();

        let UniqueByReportDupsResult { duplicates, .. } =
            unique_by_report_dups(accounts.iter(), |acc| acc.name.clone());

        if !duplicates.is_empty() {
            log::error!(
                "Found duplicate accounts for instruction {}: {}",
                &self.name,
                duplicates.iter().map(|acc| &acc.name).format(", ")
            );
            panic!();
        }

        self.write_accounts_len(tokens, n_accounts);
        self.write_accounts_struct(tokens, &accounts);
        self.write_keys_struct(tokens, &accounts);
        self.write_from_accounts_for_keys(tokens, &accounts);
        self.write_from_keys_for_meta_arr(tokens, &accounts);
        self.write_from_pubkey_arr_for_keys(tokens, &accounts);
        self.write_from_string_arr_for_keys(tokens, &accounts);
        self.write_from_accounts_for_account_info_arr(tokens, &accounts);
        self.write_from_account_info_arr_for_accounts(tokens, &accounts);

        self.write_discm(tokens);
        self.write_ix_args_struct(tokens);
        self.write_ix_data_struct(tokens);
        self.write_from_ix_args_for_ix_data(tokens);
        self.write_ix_data_impl(tokens);

        self.write_ix_fn(tokens);
        self.write_invoke_fn(tokens);
        self.write_invoke_signed_fn(tokens);

        self.write_verify_account_keys_fn(tokens, &accounts);
        
        // Generate PDA-related code
        self.write_pda_seeds_constants(tokens, &accounts);
        self.write_pda_derivation_functions(tokens, &accounts);
        self.write_fixed_address_constants(tokens, &accounts);
    }
}

#[derive(Deserialize, Debug)]
pub struct InnerAccountStruct {
    pub name: String,
    pub accounts: Vec<IxAccountEntry>,
}


#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum IxAccountEntry {
    Account(IxAccount),
    Struct(Box<InnerAccountStruct>),
}


#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IxAccount {
    pub name: String,
    #[serde(default)]
    pub docs: Option<Vec<String>>,
    #[serde(default)]
    pub address: Option<String>,  // Fixed address (e.g., program addresses)
    #[serde(default)]
    pub pda: Option<PdaConfig>,   // PDA configuration
    is_mut: Option<bool>,
    is_signer: Option<bool>,
    writable: Option<bool>,
    signer: Option<bool>,
    
    // Additional fields that are not yet fully supported
    #[serde(default)]
    pub relations: Option<Vec<String>>,  // Account relations (e.g., in pump_amm.json)
}

impl IxAccount {
    pub fn field_ident(&self) -> Ident {
        format_ident!("{}", self.name.to_snake_case())
    }

    pub fn is_mut(&self) -> bool {
        self.is_mut.unwrap_or(false) || self.writable.unwrap_or(false)
    }

    pub fn is_signer(&self) -> bool {
        self.is_signer.unwrap_or(false) || self.signer.unwrap_or(false)
    }


    pub fn to_keys_account_meta_tokens(&self) -> TokenStream {
        let is_writable_arg = LitBool::new(self.is_mut(), Span::call_site());
        let is_signer_arg = LitBool::new(self.is_signer(), Span::call_site());
        let name = self.field_ident();
        quote! {
            AccountMeta {
                pubkey: keys.#name,
                is_signer: #is_signer_arg,
                is_writable: #is_writable_arg,
            }
        }
    }

    pub fn to_verify_account_keys_tuple(&self) -> TokenStream {
        let name = self.field_ident();
        quote! {
            (*accounts.#name.key, keys.#name)
        }
    }

    /// Check if this account is a PDA
    pub fn is_pda(&self) -> bool {
        self.pda.is_some()
    }

    /// Check if this account has a fixed address
    pub fn has_fixed_address(&self) -> bool {
        self.address.is_some()
    }

    /// Get the PDA configuration if available
    pub fn get_pda_config(&self) -> Option<&PdaConfig> {
        self.pda.as_ref()
    }

    /// Get the fixed address if available
    pub fn get_fixed_address(&self) -> Option<&str> {
        self.address.as_deref()
    }
}

pub fn to_ix_accounts(accounts: &[IxAccountEntry]) -> Vec<IxAccount> {
    accounts.iter().fold(Vec::new(), |mut vec, entry| {
        match entry {
            IxAccountEntry::Account(a) => vec.push(a.clone()),
            IxAccountEntry::Struct(s) => {
                vec.extend(to_ix_accounts(&s.accounts).into_iter().map(|mut acc| {
                    acc.name = format!("{}_{}", s.name, acc.name.to_snake_case());
                    acc
                }))
            }
        };
        vec
    })
}
