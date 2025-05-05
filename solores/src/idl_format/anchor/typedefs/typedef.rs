#![allow(non_camel_case_types)]

use std::str::FromStr;

use heck::{ToPascalCase, ToSnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use syn::Index;
use void::Void;

use crate::utils::{
    conditional_pascal_case, primitive_or_pubkey_to_token, string_or_struct, PUBKEY_TOKEN,
};

#[derive(Deserialize, Debug)]
pub struct NamedType {
    pub name: String,
    pub discriminator: Option<[u8; 8]>,
    pub r#type: Option<TypedefType>,
}

impl NamedType {
    pub fn to_token_stream(&self, cli_args: &crate::Args) -> TokenStream {
        let name = format_ident!("{}", conditional_pascal_case(&self.name));

        let token_stream = match &self.r#type {
            Some(TypedefType::r#struct(typedef_struct)) => {
                let derive = if cli_args.zero_copy.iter().any(|e| e == &self.name) {
                    quote! {
                        #[repr(C)]
                        #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, Pod, Copy, Zeroable)]
                    }
                } else {
                    quote! {
                        #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
                    }
                };

                // 在 `TypedefStruct` 分支中直接生成 TokenStream
                quote! {
                    #derive
                    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                    pub struct #name {
                        #typedef_struct
                    }
                }
            }

            Some(TypedefType::r#enum(typedef_enum)) => {
                // 为 enum 生成 TokenStream
                quote! {
                    #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
                    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                    pub enum #name {
                        #typedef_enum
                    }
                }
            }

            Some(TypedefType::r#alias(typedef_alias)) => {
                // 处理 alias 类型
                if let TypedefFieldType::array(TypedefFieldArray(inner, size)) =
                    &typedef_alias.value
                {
                    let inner_type = match &**inner {
                        TypedefFieldType::PrimitiveOrPubkey(_) => quote! { u64 }, // 假设处理 `u64` 或 `Pubkey` 类型
                        TypedefFieldType::defined(_) => quote! { String }, // 你可以按需替换为实际的类型
                        // 更多类型的支持可以在这里添加
                        _ => panic!("Unsupported type in array"), // 如果类型不被支持，抛出错误
                    };
                    // 生成类型别名的 TokenStream
                    quote! {
                        pub type #name = [#inner_type; #size];
                    }
                } else {
                    panic!("Unsupported alias type");
                }
            }
            None => {
                println!("Unsupported type: {:?}", name);
                // panic!("Unsupported type");
                quote! {}
            }
        };
        token_stream
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum TypedefType {
    r#struct(TypedefStruct),
    r#enum(TypedefEnum),
    r#alias(TypedefAlias),
}
#[derive(Deserialize, Debug)]
pub struct TypedefAlias {
    pub value: TypedefFieldType, // 支持复杂类型
}

#[derive(Deserialize, Debug)]
pub struct TypedefStruct {
    pub fields: Vec<TypedefField>,
}

#[derive(Deserialize, Debug)]
pub struct TypedefField {
    pub name: String,
    #[serde(deserialize_with = "string_or_struct")]
    pub r#type: TypedefFieldType,
}
// Custom deserialization function for the `defined` variant
fn deserialize_defined<'de, D>(deserializer: D) -> Result<Value, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize the JSON value directly into serde_json::Value
    let value = Value::deserialize(deserializer)?;

    // Optional: Validate that the value is an object with a "name" field
    if !value.is_object() {
        return Err(D::Error::custom("Expected a JSON object for 'defined'"));
    }
    if !value.get("name").and_then(|v| v.as_str()).is_some() {
        return Err(D::Error::custom(
            "Expected a 'name' field in 'defined' object",
        ));
    }

    Ok(value)
}
/// All instances should be annotated with
/// deserialize_with = "string_or_struct"
#[derive(Deserialize, Debug)]
pub enum TypedefFieldType {
    // handled by string_or_struct's string
    PrimitiveOrPubkey(String),

    // rest handled by string_or_struct's struct
    #[serde(rename = "defined", deserialize_with = "deserialize_defined")]
    defined(Value),
    array(TypedefFieldArray),

    #[serde(deserialize_with = "string_or_struct")]
    option(Box<TypedefFieldType>),

    #[serde(deserialize_with = "string_or_struct")]
    vec(Box<TypedefFieldType>),
}

#[derive(Deserialize, Debug)]
pub struct TypedefFieldArray(
    #[serde(deserialize_with = "string_or_struct")] Box<TypedefFieldType>,
    u32, // borsh spec says array sizes are u32
);

/// serde newtype workaround for use in Vec<TypedefFieldType>:
/// https://github.com/serde-rs/serde/issues/723#issuecomment-871016087
#[derive(Deserialize, Debug)]
pub struct TypedefFieldTypeWrap(#[serde(deserialize_with = "string_or_struct")] TypedefFieldType);

impl FromStr for TypedefFieldType {
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::PrimitiveOrPubkey(s.into()))
    }
}

impl FromStr for Box<TypedefFieldType> {
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Box::new(TypedefFieldType::from_str(s)?))
    }
}

#[derive(Deserialize, Debug)]
pub struct TypedefEnum {
    pub variants: Vec<EnumVariant>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum EnumVariantFields {
    Struct(Vec<TypedefField>),
    Tuple(Vec<TypedefFieldTypeWrap>),
}

impl EnumVariantFields {
    pub fn has_pubkey(&self) -> bool {
        match self {
            Self::Struct(v) => v.iter().any(|f| f.r#type.is_or_has_pubkey()),
            Self::Tuple(v) => v.iter().any(|f| f.0.is_or_has_pubkey()),
        }
    }

    pub fn has_defined(&self) -> bool {
        match self {
            Self::Struct(v) => v.iter().any(|f| f.r#type.is_or_has_defined()),
            Self::Tuple(v) => v.iter().any(|f| f.0.is_or_has_defined()),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Option<EnumVariantFields>,
}

impl ToTokens for TypedefStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let typedef_fields = self.fields.iter().map(|f| quote! { pub #f });
        tokens.extend(quote! {
            #(#typedef_fields),*
        })
    }
}

impl ToTokens for TypedefField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = format_ident!("{}", self.name.to_snake_case());
        let ty = &self.r#type;
        tokens.extend(quote! {
            #name: #ty
        })
    }
}

impl ToTokens for TypedefFieldType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty: TokenStream = match self {
            Self::PrimitiveOrPubkey(s) => primitive_or_pubkey_to_token(s).parse().unwrap(),
            Self::defined(s) => s
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap()
                .parse()
                .unwrap(),
            Self::array(a) => a.to_token_stream(),
            Self::vec(v) => quote! {
                Vec<#v>
            },
            Self::option(o) => quote! {
                Option<#o>
            },
        };
        tokens.extend(ty);
    }
}

impl ToTokens for TypedefFieldArray {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty = &self.0;
        let n = Index::from(self.1 as usize);
        tokens.extend(quote! {
            [#ty; #n]
        })
    }
}

impl ToTokens for TypedefEnum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let variants = &self.variants;
        tokens.extend(quote! {
            #(#variants),*
        })
    }
}

impl ToTokens for EnumVariant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let v = format_ident!("{}", self.name.to_pascal_case());
        let maybe_inner_fields = self
            .fields
            .as_ref()
            .map_or(quote! {}, |fields| match fields {
                EnumVariantFields::Struct(v) => {
                    let typedef_fields = v.iter();
                    quote! {
                        { #(#typedef_fields),* }
                    }
                }
                EnumVariantFields::Tuple(v) => {
                    let unnamed_fields = v.iter().map(|wrap| &wrap.0);
                    quote! {
                        ( #(#unnamed_fields),* )
                    }
                }
            });
        tokens.extend(quote! {
            #v #maybe_inner_fields
        });
    }
}

impl TypedefType {
    pub fn has_pubkey_field(&self) -> bool {
        match self {
            Self::r#enum(e) => e.variants.iter().any(|e| e.has_pubkey()),
            Self::r#struct(s) => s.fields.iter().any(|f| f.r#type.is_or_has_pubkey()),
            Self::r#alias(a) => a.value.is_or_has_pubkey(),
        }
    }

    pub fn has_defined_field(&self) -> bool {
        match self {
            Self::r#enum(e) => e.variants.iter().any(|e| e.has_defined()),
            Self::r#struct(s) => s.fields.iter().any(|f| f.r#type.is_or_has_defined()),
            Self::r#alias(a) => a.value.is_or_has_defined(),
        }
    }
}

impl TypedefFieldType {
    pub fn is_or_has_pubkey(&self) -> bool {
        match self {
            Self::PrimitiveOrPubkey(s) => primitive_or_pubkey_to_token(s) == PUBKEY_TOKEN,
            Self::array(a) => a.0.is_or_has_pubkey(),
            Self::option(o) => o.is_or_has_pubkey(),
            Self::vec(v) => v.is_or_has_pubkey(),
            Self::defined(_) => false,
        }
    }

    pub fn is_or_has_defined(&self) -> bool {
        match self {
            Self::PrimitiveOrPubkey(_) => false,
            Self::array(a) => a.0.is_or_has_defined(),
            Self::option(o) => o.is_or_has_defined(),
            Self::vec(v) => v.is_or_has_defined(),
            Self::defined(_) => true,
        }
    }
}

impl EnumVariant {
    pub fn has_pubkey(&self) -> bool {
        match &self.fields {
            None => false,
            Some(fields) => fields.has_pubkey(),
        }
    }

    pub fn has_defined(&self) -> bool {
        match &self.fields {
            None => false,
            Some(fields) => fields.has_defined(),
        }
    }
}
