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
    #[serde(default)]
    pub docs: Option<Vec<String>>,
    pub discriminator: Option<[u8; 8]>,
    pub r#type: Option<TypedefType>,
}

impl NamedType {
    pub fn to_token_stream(&self, cli_args: &crate::Args) -> TokenStream {
        let name = format_ident!("{}", conditional_pascal_case(&self.name));
        
        // Generate documentation comments if present
        let doc_comments = if let Some(docs) = &self.docs {
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

        let token_stream = match &self.r#type {
            Some(TypedefType::r#struct(typedef_struct)) => {
                let derive = if cli_args.zero_copy.iter().any(|e| e == &self.name) {
                    quote! {
                        #[repr(C)]
                        #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, Eq, Pod, Copy, Zeroable)]
                    }
                } else {
                    // Check if the struct has complex types that need custom Default implementation
                    let has_complex_types = typedef_struct.fields.iter().any(|field| {
                        match &field.r#type {
                            TypedefFieldType::PrimitiveOrPubkey(s) if s == "publicKey" || s == "Pubkey" => true,
                            TypedefFieldType::defined(_) => true,
                            TypedefFieldType::array(_) => true,
                            TypedefFieldType::vec(_) => true,
                            TypedefFieldType::option(_) => true,
                            _ => false,
                        }
                    });
                    
                    if has_complex_types {
                        // Don't derive Default, we'll implement it manually
                        quote! {
                            #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, Eq)]
                        }
                    } else {
                        // Simple types can use derive Default
                        quote! {
                            #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, Eq, Default)]
                        }
                    }
                };

                // Add discriminator field if present
                let discriminator_field = if let Some(disc) = &self.discriminator {
                    quote! { pub discriminator: [u8; 8], }
                } else {
                    quote! {}
                };

                // Generate custom Default implementation if needed
                let custom_default_impl = if !cli_args.zero_copy.iter().any(|e| e == &self.name) {
                    let has_complex_types = typedef_struct.fields.iter().any(|field| {
                        match &field.r#type {
                            TypedefFieldType::PrimitiveOrPubkey(s) if s == "publicKey" || s == "Pubkey" => true,
                            TypedefFieldType::defined(_) => true,
                            TypedefFieldType::array(_) => true,
                            TypedefFieldType::vec(_) => true,
                            TypedefFieldType::option(_) => true,
                            _ => false,
                        }
                    });
                    
                    if has_complex_types {
                        let default_fields = typedef_struct.fields.iter().map(|field| {
                            let field_name = to_snake_case_with_underscores(&field.name);
                            let name = if is_rust_keyword(&field_name) {
                                format_ident!("r#{}", field_name)
                            } else {
                                format_ident!("{}", field_name)
                            };
                            
                            let default_value = match &field.r#type {
                                TypedefFieldType::PrimitiveOrPubkey(s) if s == "publicKey" || s == "Pubkey" => {
                                    quote! { Pubkey::default() }
                                }
                                TypedefFieldType::array(TypedefFieldArray(inner_type, size)) => {
                                    let size_literal = proc_macro2::Literal::usize_unsuffixed(*size as usize);
                                    // Check if inner type is Copy-able primitive
                                    match &**inner_type {
                                        TypedefFieldType::PrimitiveOrPubkey(s) => {
                                            match s.as_str() {
                                                "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "bool" => {
                                                    quote! { [Default::default(); #size_literal] }
                                                }
                                                _ => {
                                                    // For non-primitive array elements (including Pubkey and custom types)
                                                    quote! { core::array::from_fn(|_| Default::default()) }
                                                }
                                            }
                                        }
                                        _ => {
                                            // For complex array element types
                                            quote! { core::array::from_fn(|_| Default::default()) }
                                        }
                                    }
                                }
                                TypedefFieldType::vec(_) => {
                                    quote! { Vec::new() }
                                }
                                TypedefFieldType::option(_) => {
                                    quote! { None }
                                }
                                TypedefFieldType::defined(_) => {
                                    quote! { Default::default() }
                                }
                                _ => {
                                    quote! { Default::default() }
                                }
                            };
                            
                            quote! { #name: #default_value }
                        });
                        
                        let discriminator_default = if self.discriminator.is_some() {
                            quote! { discriminator: [0u8; 8], }
                        } else {
                            quote! {}
                        };
                        
                        quote! {
                            impl Default for #name {
                                fn default() -> Self {
                                    Self {
                                        #discriminator_default
                                        #(#default_fields),*
                                    }
                                }
                            }
                        }
                    } else {
                        quote! {}
                    }
                } else {
                    quote! {}
                };

                // 在 `TypedefStruct` 分支中直接生成 TokenStream
                quote! {
                    #doc_comments
                    #derive
                    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                    pub struct #name {
                        #discriminator_field
                        #typedef_struct
                    }
                    
                    #custom_default_impl
                }
            }

            Some(TypedefType::r#enum(typedef_enum)) => {
                // Generate Default implementation for enum using first variant
                let first_variant_default = if !typedef_enum.variants.is_empty() {
                    let first_variant = &typedef_enum.variants[0];
                    let variant_name = format_ident!("{}", conditional_pascal_case(&first_variant.name));
                    
                    // Check variant type and generate appropriate default value
                    let default_value = if let Some(ref fields) = first_variant.fields {
                        match fields {
                            EnumVariantFields::Struct(struct_fields) => {
                                if struct_fields.is_empty() {
                                    // Empty struct variant
                                    quote! { Self::#variant_name }
                                } else {
                                    // Struct variant with named fields
                                    let field_defaults: Vec<_> = struct_fields.iter().map(|field| {
                                        // Convert field name to snake_case to match the actual field name
                                        let snake_case_name = to_snake_case_with_underscores(&field.name);
                                        let field_name = format_ident!("{}", &snake_case_name);
                                        quote! { #field_name: Default::default() }
                                    }).collect();
                                    quote! { Self::#variant_name { #(#field_defaults),* } }
                                }
                            },
                            EnumVariantFields::Tuple(tuple_fields) => {
                                if tuple_fields.is_empty() {
                                    // Empty tuple variant
                                    quote! { Self::#variant_name }
                                } else {
                                    // Tuple variant with positional fields
                                    let field_defaults: Vec<_> = tuple_fields.iter().map(|_| {
                                        quote! { Default::default() }
                                    }).collect();
                                    quote! { Self::#variant_name(#(#field_defaults),*) }
                                }
                            }
                        }
                    } else {
                        // Unit variant (no fields property)
                        quote! { Self::#variant_name }
                    };
                    
                    quote! {
                        impl Default for #name {
                            fn default() -> Self {
                                #default_value
                            }
                        }
                    }
                } else {
                    quote! {}  // No variants, no Default impl
                };
                
                // 为 enum 生成 TokenStream
                quote! {
                    #doc_comments
                    #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, Eq)]
                    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                    pub enum #name {
                        #typedef_enum
                    }
                    
                    #first_variant_default
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
                        #doc_comments
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
    #[serde(default)]
    pub docs: Option<Vec<String>>,
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
        println!("value: {:?}", value);
        return Ok(value);
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
    #[serde(deserialize_with = "string_or_struct")] pub Box<TypedefFieldType>,
    pub u32, // borsh spec says array sizes are u32
);

/// serde newtype workaround for use in Vec<TypedefFieldType>:
/// https://github.com/serde-rs/serde/issues/723#issuecomment-871016087
#[derive(Deserialize, Debug)]
pub struct TypedefFieldTypeWrap(#[serde(deserialize_with = "string_or_struct")] pub TypedefFieldType);

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
        let typedef_fields = &self.fields;
        tokens.extend(quote! {
            #(#typedef_fields),*
        })
    }
}
fn to_snake_case_with_underscores(name: &str) -> String {
    let mut result = String::new();
    let mut leading_underscores = String::new();

    // 收集前导下划线
    if name.starts_with('_') {
        leading_underscores.push('_');
    }

    // 使用 convert_case 转换剩余部分
    let rest = &name[leading_underscores.len()..];
    let converted = rest.to_snake_case();

    // 拼接前导下划线和转换后的部分
    result.push_str(&leading_underscores);
    result.push_str(&converted);
    result
}

fn is_rust_keyword(word: &str) -> bool {
    matches!(
        word,
        "as" | "async" | "await" | "break" | "const" | "continue" | "crate" | "dyn" 
        | "else" | "enum" | "extern" | "false" | "fn" | "for" | "if" | "impl" 
        | "in" | "let" | "loop" | "match" | "mod" | "move" | "mut" | "pub" 
        | "ref" | "return" | "self" | "Self" | "static" | "struct" | "super" 
        | "trait" | "true" | "type" | "unsafe" | "use" | "where" | "while"
        | "abstract" | "become" | "box" | "do" | "final" | "macro" | "override"
        | "priv" | "typeof" | "unsized" | "virtual" | "yield" | "try"
    )
}
impl TypedefField {
    // 为结构体字段生成tokens（带pub）
    pub fn to_struct_field_tokens(&self) -> TokenStream {
        let field_name = to_snake_case_with_underscores(&self.name);
        // Handle reserved keywords by using raw identifier syntax
        let name = if is_rust_keyword(&field_name) {
            format_ident!("r#{}", field_name)
        } else {
            format_ident!("{}", field_name)
        };
        let ty = &self.r#type;
        
        // Generate documentation comments from IDL field docs
        let doc_comments = if let Some(docs) = &self.docs {
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
        
        quote! {
            #doc_comments
            pub #name: #ty
        }
    }
    
    // 为enum变体字段生成tokens（不带pub）
    fn to_enum_field_tokens(&self) -> TokenStream {
        let field_name = to_snake_case_with_underscores(&self.name);
        // Handle reserved keywords by using raw identifier syntax
        let name = if is_rust_keyword(&field_name) {
            format_ident!("r#{}", field_name)
        } else {
            format_ident!("{}", field_name)
        };
        let ty = &self.r#type;
        
        // Generate documentation comments from IDL field docs
        let doc_comments = if let Some(docs) = &self.docs {
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
        
        quote! {
            #doc_comments
            #name: #ty
        }
    }
}

impl ToTokens for TypedefField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // 默认行为：为结构体字段生成带pub的tokens
        tokens.extend(self.to_struct_field_tokens())
    }
}

impl ToTokens for TypedefFieldType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty: TokenStream = match self {
            Self::PrimitiveOrPubkey(s) => primitive_or_pubkey_to_token(s).parse().unwrap(),
            Self::defined(s) => {
                if s.is_object() {
                    s.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap()
                        .parse()
                        .unwrap()
                } else {
                    let type_str = s.as_str().unwrap();
                    // Special handling for SmallVec - map to Vec
                    if type_str.starts_with("SmallVec<") {
                        // Extract the first type parameter from SmallVec<T,N>
                        // e.g., "SmallVec<u8,u8>" -> "Vec<u8>"
                        if let Some(start) = type_str.find('<') {
                            if let Some(comma) = type_str.find(',') {
                                let inner_type = &type_str[start+1..comma];
                                format!("Vec<{}>", inner_type).parse().unwrap()
                            } else {
                                // Fallback if format is unexpected
                                "Vec<u8>".parse().unwrap()
                            }
                        } else {
                            // Fallback if format is unexpected
                            "Vec<u8>".parse().unwrap()
                        }
                    } else {
                        type_str.to_string().parse().unwrap()
                    }
                }
            }
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
                    // 使用enum字段专用的生成方法（不带pub）
                    let enum_fields = v.iter().map(|f| f.to_enum_field_tokens());
                    quote! {
                        { #(#enum_fields),* }
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
