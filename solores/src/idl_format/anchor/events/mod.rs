use proc_macro2::TokenStream;
use quote::quote;
use heck::ToSnakeCase;

use crate::idl_format::IdlCodegenModule;
use crate::idl_format::anchor::typedefs::NamedType;

mod event;
pub use event::*;

pub struct EventsCodegenModule<'a> {
    pub events: &'a [Event],
    pub named_types: &'a [NamedType],
}

impl IdlCodegenModule for EventsCodegenModule<'_> {
    fn name(&self) -> &str {
        "events"
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
        let imports = quote! {
            use borsh::{BorshDeserialize, BorshSerialize};
            use solana_program::pubkey::Pubkey;
            use crate::*;
        };

        let module_declarations: Vec<TokenStream> = self.events
            .iter()
            .map(|event| {
                let module_name = event.0.name.to_snake_case();
                let module_ident = quote::format_ident!("{}", module_name);
                quote! {
                    pub mod #module_ident;
                }
            })
            .collect();

        let exports: Vec<TokenStream> = self.events
            .iter()
            .map(|event| {
                let module_name = event.0.name.to_snake_case();
                let module_ident = quote::format_ident!("{}", module_name);
                quote! {
                    pub use #module_ident::*;
                }
            })
            .collect();

        quote! {
            #imports
            #(#module_declarations)*
            #(#exports)*
        }
    }

    fn gen_files(&self) -> Vec<(String, TokenStream)> {
        self.events
            .iter()
            .map(|event| {
                let filename = format!("{}.rs", event.0.name.to_snake_case());
                let content = self.gen_event_file(event);
                (filename, content)
            })
            .collect()
    }
}

impl EventsCodegenModule<'_> {
    fn gen_event_file(&self, event: &Event) -> TokenStream {
        let imports = quote! {
            use super::*;
        };

        // Check if we need Pubkey import
        if let Some(fields) = &event.0.fields {
            for field in fields {
                if field.r#type.is_or_has_pubkey() {
                    // Pubkey will be available through super::* (from mod.rs)
                    break;
                }
            }
        }

        // Generate event data structure (like ClaimVestedEvent)
        let event_struct = self.gen_event_struct(&event.0);
        
        // Generate event wrapper (like ClaimVestedEventEvent)
        let event_wrapper = self.gen_event_wrapper(event);

        quote! {
            #imports
            #event_struct
            #event_wrapper
        }
    }

    fn gen_event_struct(&self, event_type: &EventType) -> TokenStream {
        let struct_ident = event_type.struct_ident();
        
        // Generate doc comments for the event
        let doc_comments = if let Some(docs) = &event_type.docs {
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
            // Generate default documentation
            let doc_str = format!("Event emitted by {}", event_type.name);
            quote! { #[doc = #doc_str] }
        };
        
        if let Some(struct_fields) = &event_type.fields {
            // Event has fields defined directly
            // Generate fields manually to avoid double pub
            let fields = struct_fields.iter().map(|f| f.to_struct_field_tokens());
            quote! {
                #doc_comments
                #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, Eq)]
                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                pub struct #struct_ident {
                    #(#fields),*
                }
            }
        } else {
            // Event has no fields, look for the type definition in named_types
            if let Some(named_type) = self.named_types.iter().find(|t| t.name == event_type.name) {
                // Use named_type docs instead of event_type docs to avoid duplication
                let override_doc_comments = if let Some(docs) = &named_type.docs {
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
                    doc_comments  // Fallback to event docs
                };
                
                // Generate the struct from named_type
                if let Some(typedef_type) = &named_type.r#type {
                    match typedef_type {
                        crate::idl_format::anchor::typedefs::TypedefType::r#struct(typedef_struct) => {
                            // Generate fields manually to avoid double pub
                            let fields = typedef_struct.fields.iter().map(|f| f.to_struct_field_tokens());
                            quote! {
                                #override_doc_comments
                                #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, Eq)]
                                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                                pub struct #struct_ident {
                                    #(#fields),*
                                }
                            }
                        },
                        _ => {
                            // For non-struct types, just generate empty struct with docs
                            quote! {
                                #override_doc_comments
                                #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, Eq)]
                                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                                pub struct #struct_ident;
                            }
                        }
                    }
                } else {
                    quote! {
                        #override_doc_comments
                        #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, Eq)]
                        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                        pub struct #struct_ident;
                    }
                }
            } else {
                // No type definition found, generate empty struct
                quote! {
                    #doc_comments
                    #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, Eq)]
                    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                    pub struct #struct_ident;
                }
            }
        }
    }

    fn gen_event_wrapper(&self, event: &Event) -> TokenStream {
        use heck::ToShoutySnakeCase;
        use sha2::{Digest, Sha256};
        
        // Generate discriminator
        let event_discm_ident = quote::format_ident!("{}_EVENT_DISCM", event.0.name.to_shouty_snake_case());
        let discm = <[u8; 8]>::try_from(
            &Sha256::digest(format!("event:{}", event.0.name).as_bytes()).as_slice()[..8],
        ).unwrap();
        let discm_tokens: TokenStream = format!("{:?}", discm).parse().unwrap();

        let struct_ident = event.0.struct_ident();
        let event_ident = quote::format_ident!("{}Event", struct_ident);
        
        // Add documentation for the event wrapper
        let wrapper_doc = format!("Event wrapper for {} with discriminator for serialization", struct_ident);

        quote! {
            pub const #event_discm_ident: [u8; 8] = #discm_tokens;

            #[doc = #wrapper_doc]
            #[derive(Clone, Debug, PartialEq)]
            pub struct #event_ident(pub #struct_ident);

            impl BorshSerialize for #event_ident {
                fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                    #event_discm_ident.serialize(writer)?;
                    self.0.serialize(writer)
                }
            }

            impl #event_ident {
                pub fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
                    let maybe_discm = <[u8; 8]>::deserialize(buf)?;
                    if maybe_discm != #event_discm_ident {
                        return Err(
                            std::io::Error::new(
                                std::io::ErrorKind::Other, 
                                format!("discm does not match. Expected: {:?}. Received: {:?}", #event_discm_ident, maybe_discm)
                            )
                        );
                    }
                    Ok(Self(#struct_ident::deserialize(buf)?))
                }
            }
        }
    }
}
