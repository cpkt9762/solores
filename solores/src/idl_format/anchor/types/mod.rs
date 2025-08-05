use proc_macro2::TokenStream;
use quote::quote;
use heck::ToSnakeCase;

use crate::idl_format::IdlCodegenModule;

mod types;
pub use types::*;

pub struct TypesCodegenModule<'a> {
    pub cli_args: &'a crate::Args,
    pub named_types: &'a [NamedType],
}

impl TypesCodegenModule<'_> {
    /// Filter out account types and event types to avoid duplication
    fn filtered_named_types(&self) -> Vec<&NamedType> {
        const ACCOUNT_TYPE_NAMES: &[&str] = &[
            "GlobalConfig",
            "PlatformConfig", 
            "PoolState",
            "VestingRecord"
        ];
        
        const EVENT_TYPE_NAMES: &[&str] = &[
            "ClaimVestedEvent",
            "CreateVestingEvent",
            "PoolCreateEvent", 
            "TradeEvent"
        ];
        
        self.named_types
            .iter()
            .filter(|named_type| {
                !ACCOUNT_TYPE_NAMES.contains(&named_type.name.as_str()) &&
                !EVENT_TYPE_NAMES.contains(&named_type.name.as_str())
            })
            .collect()
    }
}

impl IdlCodegenModule for TypesCodegenModule<'_> {
    fn name(&self) -> &str {
        "types"
    }

    fn has_multiple_files(&self) -> bool {
        true
    }

    fn gen_head(&self) -> TokenStream {
        // Not used for multi-file modules
        TokenStream::new()
    }

    fn gen_body(&self) -> TokenStream {
        // Not used for multi-file modules
        TokenStream::new()
    }

    fn gen_mod_file(&self) -> TokenStream {
        let filtered_types = self.filtered_named_types();
        let module_declarations: Vec<TokenStream> = filtered_types
            .iter()
            .map(|named_type| {
                let module_name = named_type.name.to_snake_case();
                let module_ident = quote::format_ident!("{}", module_name);
                quote! {
                    pub mod #module_ident;
                }
            })
            .collect();

        let exports: Vec<TokenStream> = filtered_types
            .iter()
            .map(|named_type| {
                let module_name = named_type.name.to_snake_case();
                let module_ident = quote::format_ident!("{}", module_name);
                quote! {
                    pub use #module_ident::*;
                }
            })
            .collect();

        quote! {
            #(#module_declarations)*
            #(#exports)*
        }
    }

    fn gen_files(&self) -> Vec<(String, TokenStream)> {
        let filtered_types = self.filtered_named_types();
        
        filtered_types
            .iter()
            .map(|named_type| {
                let filename = format!("{}.rs", named_type.name.to_snake_case());
                let content = self.gen_type_file(named_type);
                (filename, content)
            })
            .collect()
    }
}

impl TypesCodegenModule<'_> {
    fn gen_type_file(&self, named_type: &NamedType) -> TokenStream {
        let mut imports = quote! {
            use borsh::{BorshDeserialize, BorshSerialize};
        };

        // Add zero-copy imports if needed
        if self.cli_args.zero_copy.iter().any(|e| e == &named_type.name) {
            imports.extend(quote! {
                use bytemuck::{Pod, Zeroable};
            });
        }

        // Add Pubkey import if needed
        if let Some(r#type) = &named_type.r#type {
            if r#type.has_pubkey_field() {
                imports.extend(quote! {
                    use solana_program::pubkey::Pubkey;
                });
            }
        }

        // Add imports for referenced types from the same module
        if let Some(referenced_types) = self.get_referenced_types(named_type) {
            for ref_type in referenced_types {
                let ref_type_ident = quote::format_ident!("{}", ref_type);
                imports.extend(quote! {
                    use crate::#ref_type_ident;
                });
            }
        }

        let type_definition = named_type.to_token_stream(self.cli_args);

        quote! {
            #imports
            #type_definition
        }
    }

    /// Extract referenced type names from a named type
    fn get_referenced_types(&self, named_type: &NamedType) -> Option<Vec<String>> {
        let mut referenced_types = Vec::new();
        
        if let Some(r#type) = &named_type.r#type {
            self.collect_type_references(r#type, &mut referenced_types);
        }
        
        if referenced_types.is_empty() {
            None
        } else {
            Some(referenced_types)
        }
    }

    /// Recursively collect all type references from a TypedefType
    fn collect_type_references(&self, typedef_type: &crate::idl_format::anchor::typedefs::TypedefType, referenced_types: &mut Vec<String>) {
        use crate::idl_format::anchor::typedefs::*;
        
        match typedef_type {
            TypedefType::r#struct(s) => {
                for field in &s.fields {
                    self.collect_field_type_references(&field.r#type, referenced_types);
                }
            }
            TypedefType::r#enum(e) => {
                for variant in &e.variants {
                    if let Some(fields) = &variant.fields {
                        match fields {
                            EnumVariantFields::Struct(fields) => {
                                for field in fields {
                                    self.collect_field_type_references(&field.r#type, referenced_types);
                                }
                            }
                            EnumVariantFields::Tuple(fields) => {
                                for wrapper in fields {
                                    self.collect_field_type_references(&wrapper.0, referenced_types);
                                }
                            }
                        }
                    }
                }
            }
            TypedefType::r#alias(a) => {
                self.collect_field_type_references(&a.value, referenced_types);
            }
        }
    }

    /// Collect type references from a TypedefFieldType
    fn collect_field_type_references(&self, field_type: &crate::idl_format::anchor::typedefs::TypedefFieldType, referenced_types: &mut Vec<String>) {
        use crate::idl_format::anchor::typedefs::*;
        
        match field_type {
            TypedefFieldType::defined(value) => {
                if let Some(type_name) = value.as_str() {
                    // Check if this is a type defined in our module
                    if self.named_types.iter().any(|nt| nt.name == type_name) {
                        if !referenced_types.contains(&type_name.to_string()) {
                            referenced_types.push(type_name.to_string());
                        }
                    }
                } else if let Some(obj) = value.as_object() {
                    if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
                        if self.named_types.iter().any(|nt| nt.name == name) {
                            if !referenced_types.contains(&name.to_string()) {
                                referenced_types.push(name.to_string());
                            }
                        }
                    }
                }
            }
            TypedefFieldType::array(a) => {
                self.collect_field_type_references(&a.0, referenced_types);
            }
            TypedefFieldType::vec(v) => {
                self.collect_field_type_references(v, referenced_types);
            }
            TypedefFieldType::option(o) => {
                self.collect_field_type_references(o, referenced_types);
            }
            TypedefFieldType::PrimitiveOrPubkey(_) => {
                // No references to collect for primitive types
            }
        }
    }
}