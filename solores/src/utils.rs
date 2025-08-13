use std::{
    collections::HashSet,
    fmt,
    fs::{File, OpenOptions},
    hash::Hash,
    marker::PhantomData,
    path::Path,
    str::FromStr,
};

use heck::ToPascalCase;
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use void::Void;

pub const PUBKEY_TOKEN: &str = "Pubkey";

pub fn primitive_or_pubkey_to_token(s: &str) -> String {
    match s {
        "publicKey" | "pubkey" | "Pubkey" => PUBKEY_TOKEN.to_owned(),
        "string" => s.to_pascal_case(),
        "bytes" => "Vec<u8>".to_owned(),
        _ => s.to_owned(),
    }
}

pub fn open_file_create_overwrite<P: AsRef<Path>>(path: P) -> std::io::Result<File> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
}

/// Copied from https://serde.rs/string-or-struct.html
pub fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = Void>,
    D: Deserializer<'de>,
{
    // This is a Visitor that forwards string types to T's `FromStr` impl and
    // forwards map types to T's `Deserialize` impl. The `PhantomData` is to
    // keep the compiler from complaining about T being an unused generic type
    // parameter. We need T in order to know the Value type for the Visitor
    // impl.
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = Void>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_map<M>(self, map: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            // `MapAccessDeserializer` is a wrapper that turns a `MapAccess`
            // into a `Deserializer`, allowing it to be used as the input to T's
            // `Deserialize` implementation. T then deserializes itself using
            // the entries from the map visitor.
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}

pub struct UniqueByReportDupsResult<'a, T> {
    pub unique: Vec<&'a T>,
    pub duplicates: Vec<&'a T>,
}

/// Preserves order of vals
pub fn unique_by_report_dups<'a, I, T, V, F>(vals: I, mut f: F) -> UniqueByReportDupsResult<'a, T>
where
    I: Iterator<Item = &'a T>,
    V: Eq + Hash,
    F: FnMut(&T) -> V,
{
    let mut hashes = HashSet::new();
    let mut unique = Vec::new();
    let mut duplicates = Vec::new();
    for val in vals {
        let hash = f(val);
        if hashes.contains(&hash) {
            duplicates.push(val);
        } else {
            hashes.insert(hash);
            unique.push(val);
        }
    }
    UniqueByReportDupsResult { unique, duplicates }
}

pub fn conditional_pascal_case(s: &str) -> String {
    // Only apply PascalCase if the string does not start with an uppercase letter.
    if s.chars().next().map_or(false, |c| c.is_uppercase()) {
        s.to_string()
    } else {
        s.to_pascal_case()
    }
}

/// 将字段名转换为安全的Rust标识符，处理关键字
pub fn sanitize_field_name(field_name: &str) -> String {
    // Rust关键字列表
    const RUST_KEYWORDS: &[&str] = &[
        "as", "break", "const", "continue", "crate", "else", "enum", "extern",
        "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
        "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
        "super", "trait", "true", "type", "unsafe", "use", "where", "while",
        "async", "await", "dyn", "abstract", "become", "box", "do", "final",
        "macro", "override", "priv", "typeof", "unsized", "virtual", "yield",
        "try",
    ];
    
    if RUST_KEYWORDS.contains(&field_name) {
        // 使用后缀避免关键字冲突，而不是raw identifier
        format!("{}_field", field_name)
    } else {
        field_name.to_string()
    }
}

/// 创建安全的Rust标识符
pub fn create_safe_ident(field_name: &str) -> syn::Ident {
    let safe_name = sanitize_field_name(field_name);
    syn::Ident::new(&safe_name, proc_macro2::Span::call_site())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "bytes_to_u8")]
    fn test_bytes_to_u8_feature_enabled() {
        let result = primitive_or_pubkey_to_token("bytes");
        assert_eq!(result, "u8");

        let result = primitive_or_pubkey_to_token("publicKey");
        assert_eq!(result, PUBKEY_TOKEN.to_owned());

        let result = primitive_or_pubkey_to_token("string");
        assert_eq!(result, "String");
    }

    #[test]
    #[cfg(not(feature = "bytes_to_u8"))]
    fn test_bytes_to_u8_feature_disabled() {
        let result = primitive_or_pubkey_to_token("bytes");
        assert_eq!(result, "bytes");

        let result = primitive_or_pubkey_to_token("publicKey");
        assert_eq!(result, PUBKEY_TOKEN.to_owned());

        let result = primitive_or_pubkey_to_token("string");
        assert_eq!(result, "String");
    }

    #[test]
    fn test_already_uppercase() {
        let input = "I80F48";
        let expected = "I80F48";
        assert_eq!(conditional_pascal_case(input), expected);
    }

    #[test]
    fn test_lowercase_single_word() {
        let input = "pool";
        let expected = "Pool";
        assert_eq!(conditional_pascal_case(input), expected);
    }

    #[test]
    fn test_mixed_case_string() {
        let input = "exampleString";
        let expected = "ExampleString";
        assert_eq!(conditional_pascal_case(input), expected);
    }

    #[test]
    fn test_empty_string() {
        let input = "";
        let expected = "";
        assert_eq!(conditional_pascal_case(input), expected);
    }

    #[test]
    fn test_already_pascal_case() {
        let input = "PascalCase";
        let expected = "PascalCase";
        assert_eq!(conditional_pascal_case(input), expected);
    }

    #[test]
    fn test_multiple_words() {
        let input = "multiple words";
        let expected = "MultipleWords";
        assert_eq!(conditional_pascal_case(input), expected);
    }

    #[test]
    fn test_numeric_start() {
        let input = "123abc";
        let expected = "123abc";
        assert_eq!(conditional_pascal_case(input), expected);
    }

    #[test]
    fn test_uppercase_first_letter() {
        let input = "Uppercase";
        let expected = "Uppercase";
        assert_eq!(conditional_pascal_case(input), expected);
    }
}

/// 字段名转换工具：将 camelCase 转换为 snake_case，并生成对应的条件编译 serde rename 属性
/// 
/// # Arguments
/// * `camel_name` - camelCase 格式的字段名
/// 
/// # Returns
/// * `(snake_case_name, conditional_serde_attribute_token)`
pub fn to_snake_case_with_serde(camel_name: &str) -> (String, TokenStream) {
    let snake_name = camel_name.to_case(Case::Snake);
    let safe_snake_name = sanitize_field_name(&snake_name);
    
    // 如果转换后的名称与原名称不同，生成条件编译的 serde rename 属性
    let serde_attr = if safe_snake_name != camel_name {
        quote! { #[cfg_attr(feature = "serde", serde(rename = #camel_name))] }
    } else {
        quote! {}
    };
    
    (safe_snake_name, serde_attr)
}

/// 检查字段名是否需要重命名（是否为 camelCase）
pub fn needs_snake_case_conversion(name: &str) -> bool {
    let snake_version = name.to_case(Case::Snake);
    snake_version != name
}

/// 为 Pubkey 类型生成条件编译的 serde 序列化属性
/// 
/// 使用 serde_with::DisplayFromStr 将 Pubkey 序列化为字符串格式
/// 
/// # Returns
/// * `TokenStream` - 生成的 serde 属性代码
pub fn generate_pubkey_serde_attr() -> TokenStream {
    quote! {
        #[cfg_attr(
            feature = "serde",
            serde(with = "serde_with::As::<serde_with::DisplayFromStr>")
        )]
    }
}

/// 检查字段类型是否为 Pubkey
pub fn is_pubkey_type(type_str: &str) -> bool {
    matches!(type_str, "publicKey" | "pubkey" | "Pubkey")
}

#[cfg(test)]
mod field_naming_tests {
    use super::*;

    #[test]
    fn test_camel_to_snake_conversion() {
        let (snake_name, _) = to_snake_case_with_serde("buyOrders");
        assert_eq!(snake_name, "buy_orders");
    }

    #[test]
    fn test_target_x_conversion() {
        let (snake_name, _) = to_snake_case_with_serde("targetX");
        assert_eq!(snake_name, "target_x");
    }

    #[test]
    fn test_already_snake_case() {
        let (snake_name, _) = to_snake_case_with_serde("already_snake");
        assert_eq!(snake_name, "already_snake");
    }

    #[test]
    fn test_needs_conversion_check() {
        assert!(needs_snake_case_conversion("buyOrders"));
        assert!(needs_snake_case_conversion("targetX"));
        assert!(!needs_snake_case_conversion("already_snake"));
        assert!(!needs_snake_case_conversion("simple"));
    }
}
