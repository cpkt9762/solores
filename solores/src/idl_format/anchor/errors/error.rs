use heck::ToPascalCase;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use serde::Deserialize;
use syn::LitInt;

#[derive(Deserialize, Debug)]
pub struct ErrorEnumVariant {
    code: u32,
    name: String,
    msg: Option<String>,
    #[serde(default)]
    pub docs: Option<Vec<String>>,
}

impl ToTokens for ErrorEnumVariant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let variant_ident = format_ident!("{}", self.name.to_pascal_case());
        let msg = match &self.msg {
            Some(msg) => quote! { #msg },
            None => {
                let formatted_name = self.name.replace('_', " ");
                quote! { #formatted_name }
            }
        };
        let code_literal = LitInt::new(&self.code.to_string(), Span::call_site());
        
        // Generate documentation comments for the error variant
        // Format: /// {code} - {msg}
        let doc_comments = if let Some(docs) = &self.docs {
            // If IDL provides docs, use them
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
            // Generate standard format: /// {code} - {msg}
            let msg_str = match &self.msg {
                Some(msg) => msg.clone(),
                None => self.name.replace('_', " "),
            };
            let doc_str = format!("{} - {}", self.code, msg_str);
            quote! { #[doc = #doc_str] }
        };
        
        tokens.extend(quote! {
            #doc_comments
            #[error(#msg)]
            #variant_ident = #code_literal,
        })
    }
}
