use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::idl_format::IdlCodegenModule;

mod account;
pub use account::*;

pub struct AccountsCodegenModule<'a> {
    pub cli_args: &'a crate::Args,
    pub named_accounts: &'a [NamedAccount],
    pub typedefs: &'a [crate::idl_format::anchor::typedefs::NamedType],
    pub is_anchor_contract: bool,
}

impl AccountsCodegenModule<'_> {
    // 将 docs 数组转换为 Rust 文档注释
    fn generate_doc_comments(docs: &Option<Vec<String>>) -> TokenStream {
        if let Some(doc_lines) = docs {
            let doc_tokens: Vec<TokenStream> = doc_lines
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
        }
    }

    // 生成通用导入语句
    fn gen_common_imports(&self) -> TokenStream {
        let mut res = quote! {
            use borsh::{BorshDeserialize, BorshSerialize};
        };
        
        // Check if any account needs bytemuck
        for a in self.named_accounts {
            if self.cli_args.zero_copy.iter().any(|e| e == &a.0.name) {
                res.extend(quote! {
                    use bytemuck::{Pod, Zeroable};
                });
                break;
            }
        }
        
        // Always add Pubkey import for accounts
        res.extend(quote! {
            use solana_program::pubkey::Pubkey;
        });
        
        // Import common types from crate::types only if types exist
        if !self.typedefs.is_empty() {
            res.extend(quote! {
                use crate::types::*;
            });
        }
        
        res
    }

}

impl IdlCodegenModule for AccountsCodegenModule<'_> {
    fn name(&self) -> &str {
        "accounts"
    }

    fn has_multiple_files(&self) -> bool {
        true
    }

    fn gen_files(&self) -> Vec<(String, TokenStream)> {
        log::debug!("🗂️ 生成账户文件列表 (is_anchor_contract={})", self.is_anchor_contract);
        let mut files = Vec::new();
        
        for account in self.named_accounts.iter() {
            let filename = format!("{}.rs", account.0.name.to_lowercase());
            log::debug!("  ├─ 准备生成文件: {}", filename);
            let content = self.gen_individual_account_file(account);
            files.push((filename, content));
        }
        
        log::debug!("✅ 账户文件列表生成完成: {} 个文件", files.len());
        files
    }

    fn gen_mod_file(&self) -> TokenStream {
        let common_imports = self.gen_common_imports();
        let module_imports = self.named_accounts.iter().map(|account| {
            let module_name = format_ident!("{}", account.0.name.to_lowercase());
            quote! {
                pub mod #module_name;
                pub use #module_name::*;
            }
        });

        quote! {
            #common_imports
            #(#module_imports)*
        }
    }

    fn gen_head(&self) -> TokenStream {
        let mut res = quote! {
            use borsh::{BorshDeserialize, BorshSerialize};
        };
        for a in self.named_accounts {
            if self.cli_args.zero_copy.iter().any(|e| e == &a.0.name) {
                res.extend(quote! {
                    use bytemuck::{Pod, Zeroable};
                });
                break;
            }
        }
        let mut has_pubkey = false;
        let mut has_defined = false;
        for a in self.named_accounts {
            if let Some(r#type) = &a.0.r#type {
                if r#type.has_pubkey_field() && !has_pubkey {
                    has_pubkey = true;
                    res.extend(quote! {
                        use solana_program::pubkey::Pubkey;
                    });
                    if r#type.has_defined_field() && !has_defined {
                        has_defined = true;
                        res.extend(quote! {
                            use crate::*;
                        })
                    }
                }
            }

            if has_defined && has_pubkey {
                break;
            }
        }
        res
    }

    fn gen_body(&self) -> TokenStream {
        self.named_accounts
            .iter()
            .map(|e| e.to_token_stream_with_typedefs(self.cli_args, self.is_anchor_contract, self.typedefs))
            .collect()
    }
}

impl AccountsCodegenModule<'_> {
    // 根据名称查找对应的 typedef 结构体
    fn find_typedef_by_name(&self, name: &str) -> Option<&crate::idl_format::anchor::typedefs::NamedType> {
        self.typedefs.iter().find(|typedef| typedef.name == name)
    }
    
    // 从 typedef 中提取结构体字段
    fn extract_struct_fields<'a>(&self, typedef: &'a crate::idl_format::anchor::typedefs::NamedType) -> Option<&'a crate::idl_format::anchor::typedefs::TypedefStruct> {
        use crate::idl_format::anchor::typedefs::TypedefType;
        match &typedef.r#type {
            Some(TypedefType::r#struct(s)) => Some(s),
            _ => None,
        }
    }


    fn gen_individual_account_file(&self, account: &NamedAccount) -> TokenStream {
        log::debug!("📋 生成单个账户文件: {} (is_anchor_contract={})", account.0.name, self.is_anchor_contract);
        
        let base_imports = quote! {
            use super::*;
        };
        
        let account_content = account.to_token_stream_with_typedefs(self.cli_args, self.is_anchor_contract, self.typedefs);
        quote! {
            #base_imports
            #account_content
        }
    }

    fn gen_complete_account_struct(&self, account: &NamedAccount) -> TokenStream {
        use crate::utils::conditional_pascal_case;
        
        let name = &account.0.name;
        let name_ident = format_ident!("{}", conditional_pascal_case(name));
        
        // 生成完整的结构体定义
        let struct_content = if let Some(typedef_type) = &account.0.r#type {
            self.gen_struct_from_typedef(typedef_type, &name_ident)
        } else {
            // 如果没有类型定义，从 crate::typedefs 中引用对应的结构体
            // 生成一个使用 typedefs 结构体字段的完整结构体
            self.gen_struct_from_typedefs_reference(&name_ident)
        };
        
        // 生成基本序列化方法
        let methods = quote! {
            impl #name_ident {
                pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
                    BorshDeserialize::deserialize(&mut &buf[..])
                }
                
                pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
                    BorshSerialize::serialize(self, &mut writer)
                }
                
                pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
                    borsh::to_vec(self)
                }
            }
        };
        
        quote! {
            #struct_content
            #methods
        }
    }

    fn gen_struct_from_typedef(&self, typedef_type: &crate::idl_format::anchor::typedefs::TypedefType, name_ident: &proc_macro2::Ident) -> TokenStream {
        use crate::idl_format::anchor::typedefs::TypedefType;
        
        match typedef_type {
            TypedefType::r#struct(typedef_struct) => {
                let fields = typedef_struct.fields.iter().map(|field| {
                    let field_name = format_ident!("{}", field.name);
                    let field_type = field.r#type.to_token_stream();
                    quote! {
                        pub #field_name: #field_type,
                    }
                });
                
                quote! {
                    #[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
                    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                    pub struct #name_ident {
                        pub discriminator: [u8; 8],
                        #(#fields)*
                    }
                }
            }
            TypedefType::r#enum(_typedef_enum) => {
                // 如果是枚举类型，也生成简单结构体
                quote! {
                    #[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
                    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                    pub struct #name_ident {
                        pub discriminator: [u8; 8],
                    }
                }
            }
            TypedefType::alias(_typedef_alias) => {
                // 如果是别名类型，也生成简单结构体
                quote! {
                    #[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
                    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                    pub struct #name_ident {
                        pub discriminator: [u8; 8],
                    }
                }
            }
        }
    }

    fn gen_struct_from_typedefs_reference(&self, name_ident: &proc_macro2::Ident) -> TokenStream {
        let name = name_ident.to_string();
        
        // 动态查找对应的 typedef
        if let Some(typedef) = self.find_typedef_by_name(&name) {
            // 生成结构体级别的文档注释
            let struct_docs = Self::generate_doc_comments(&typedef.docs);
            
            if let Some(struct_def) = self.extract_struct_fields(typedef) {
                // 动态生成字段，包含文档注释和 serde_with 支持
                let fields = struct_def.fields.iter().map(|field| {
                    let field_name = format_ident!("{}", field.name);
                    let field_type = field.r#type.to_token_stream();
                    
                    // 生成字段级别的文档注释
                    let field_docs = Self::generate_doc_comments(&field.docs);
                    
                    // 检查是否是 Pubkey 类型，添加 serde_with 支持
                    let serde_attr = if field.r#type.is_or_has_pubkey() {
                        quote! {
                            #[cfg_attr(feature = "serde", serde(with = "serde_with::As::<serde_with::DisplayFromStr>"))]
                        }
                    } else {
                        quote! {}
                    };
                    
                    quote! {
                        #field_docs
                        #serde_attr
                        pub #field_name: #field_type,
                    }
                });
                
                return quote! {
                    #struct_docs
                    #[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
                    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                    pub struct #name_ident {
                        pub discriminator: [u8; 8],
                        #(#fields)*
                    }
                };
            }
        }
        
        // 如果找不到对应的 typedef，生成只有 discriminator 的简单结构体
        quote! {
            #[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            pub struct #name_ident {
                pub discriminator: [u8; 8],
            }
        }
    }
}
