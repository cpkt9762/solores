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
        tokens.extend(quote! {
            #[error(#msg)]
            #variant_ident = #code_literal,
        })
    }
}
