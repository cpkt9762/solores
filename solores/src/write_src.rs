use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::quote;
use std::{io::Write, path::Path};

use crate::{idl_format::IdlFormat, utils::open_file_create_overwrite, Args};

const DEFAULT_PROGRAM_ID_STR: &str = "TH1S1SNoTAVAL1DPUBKEYDoNoTUSE11111111111111";

const MAX_BASE58_LEN: usize = 44;
const PUBKEY_BYTES_SIZE: usize = 32;

/// Copied from solana_program::Pubkey::from_str()
/// so that we dont have to have solana_program as a dep
fn is_valid_pubkey(s: &str) -> bool {
    if s.len() > MAX_BASE58_LEN {
        return false;
    }
    let pubkey_vec = match bs58::decode(s).into_vec() {
        Ok(v) => v,
        Err(_) => return false,
    };
    if pubkey_vec.len() != PUBKEY_BYTES_SIZE {
        return false;
    }
    true
}

pub fn write_lib(args: &Args, idl: &dyn IdlFormat) -> std::io::Result<()> {
    let user_provided_id_opt =
        args.program_id
            .as_ref()
            .and_then(|s| if is_valid_pubkey(s) { Some(s) } else { None });
    let id = user_provided_id_opt
        .map(|string| string.as_ref())
        .unwrap_or_else(|| {
            idl.program_address().unwrap_or_else(|| {
                log::warn!(
                    "program address not in IDL, setting to default: {}",
                    DEFAULT_PROGRAM_ID_STR
                );
                DEFAULT_PROGRAM_ID_STR
            })
        });

    let mut contents = quote! {
        solana_program::declare_id!(#id);
    };

    for module in idl.modules(args) {
        let module_name = module.name();
        let module_ident = Ident::new(module.name(), Span::call_site());
        contents.extend(quote! {
            pub mod #module_ident;
            pub use #module_ident::*;
        });
        let mut module_contents = module.gen_head();
        module_contents.extend(module.gen_body());

        write_src_file(args, &format!("src/{module_name}.rs"), module_contents)?;
        println!("write_src_file done: {}", module_name);
    }

    write_src_file(args, "src/lib.rs", contents)
}

fn write_src_file<P: AsRef<Path>>(
    args: &Args,
    src_file_path: P,
    mut contents: TokenStream,
) -> std::io::Result<()> {
    let sanitized_contents = sanitize_tokens(contents);
    let unpretty = match syn::parse2(sanitized_contents) {
        Ok(unpretty) => unpretty,
        Err(e) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                e.to_string(),
            ));
        }
    };
    let formatted = prettyplease::unparse(&unpretty);
    let path = args.output_dir.join(src_file_path);
    let mut file = open_file_create_overwrite(path)?;
    file.write_all(formatted.as_bytes())?;
    file.flush()
}

fn sanitize_tokens(input: TokenStream) -> TokenStream {
    input.into_iter().map(sanitize_token).collect()
}

fn sanitize_token(token: TokenTree) -> TokenTree {
    match token {
        TokenTree::Group(group) => {
            let content = sanitize_tokens(group.stream());
            TokenTree::Group(proc_macro2::Group::new(group.delimiter(), content))
        }
        TokenTree::Ident(ident) if ident == "type" => {
            let raw_type = quote! {  type };
            raw_type.into_iter().next().unwrap()
        }
        _ => token,
    }
}
