//! Anchor IDL数据结构
//!
//! 定义专门用于Anchor合约的统一IDL格式，支持8字节discriminator、
//! Anchor特有的字段约定等Anchor特性

use serde::{Deserialize, Serialize};
// use std::sync::OnceLock;

// 类型别名用于兼容Legacy系统
pub type Event = AnchorEvent;
pub type IxAccount = AnchorAccountConstraint;
pub type TypedefField = AnchorField;

/// 枚举变体字段（兼容Legacy系统）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnumVariantFields {
    Struct(Vec<AnchorField>),
    Tuple(Vec<AnchorField>),
}

/// 指令账户解析函数（兼容Legacy系统）
pub fn to_ix_accounts(accounts: &[AnchorAccountConstraint]) -> Vec<AnchorAccountConstraint> {
    accounts.to_vec()
}

/// Anchor合约的统一IDL格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorIdl {
    /// 程序名称（可选，优先从metadata.name获取）
    pub name: Option<String>,
    /// 程序版本（可选，优先从metadata.version获取）
    pub version: Option<String>,
    /// 程序地址
    pub address: String,
    /// 元数据
    pub metadata: AnchorMetadata,
    /// 指令定义
    pub instructions: Option<Vec<AnchorInstruction>>,
    /// 账户定义
    pub accounts: Option<Vec<AnchorAccount>>,
    /// 类型定义
    pub types: Option<Vec<AnchorType>>,
    /// 事件定义
    pub events: Option<Vec<AnchorEvent>>,
    /// 错误定义
    pub errors: Option<Vec<AnchorError>>,
    /// 常量定义
    pub constants: Option<Vec<AnchorConstant>>,
    // 字段分配缓存已移除 - 传统模板系统不再使用
    // #[serde(skip)]
    // pub field_allocation_cache: OnceLock<crate::templates::field_analyzer::FieldAllocationMap>,
}

/// Anchor合约元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorMetadata {
    /// 程序地址（可选，通常在根级别）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// 程序名称
    pub name: String,
    /// 程序版本
    pub version: String,
    /// IDL规范类型（固定为"anchor"）
    pub spec: String,
    /// 描述
    pub description: Option<String>,
}

/// Anchor指令定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorInstruction {
    /// 指令名称
    pub name: String,
    /// 指令discriminator（固定8字节）
    pub discriminator: [u8; 8],
    /// 指令参数
    pub args: Option<Vec<AnchorField>>,
    /// 指令账户
    pub accounts: Option<Vec<AnchorAccountConstraint>>,
    /// 文档注释
    pub docs: Option<Vec<String>>,
}

/// Anchor账户定义
#[derive(Debug, Clone, Serialize)]
pub struct AnchorAccount {
    /// 账户名称
    pub name: String,
    /// 账户discriminator（固定8字节）
    pub discriminator: [u8; 8],
    /// 账户字段
    pub fields: Option<Vec<AnchorField>>,
    /// 文档注释
    pub docs: Option<Vec<String>>,
}

impl<'de> serde::Deserialize<'de> for AnchorAccount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct AnchorAccountVisitor;

        impl<'de> Visitor<'de> for AnchorAccountVisitor {
            type Value = AnchorAccount;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a JSON object representing an AnchorAccount")
            }

            fn visit_map<V>(self, mut map: V) -> Result<AnchorAccount, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name = None;
                let mut discriminator = None;
                let mut fields = None;
                let mut docs = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "name" => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        "discriminator" => {
                            if discriminator.is_some() {
                                return Err(de::Error::duplicate_field("discriminator"));
                            }
                            discriminator = Some(map.next_value()?);
                        }
                        "fields" => {
                            // 直接存在的fields字段
                            if fields.is_some() {
                                return Err(de::Error::duplicate_field("fields"));
                            }
                            fields = Some(map.next_value()?);
                        }
                        "type" => {
                            // 从嵌套的type对象中提取fields
                            let type_obj: serde_json::Value = map.next_value()?;
                            
                            if let serde_json::Value::Object(ref type_map) = type_obj {
                                // 检查是否为struct类型
                                if let Some(kind_value) = type_map.get("kind") {
                                    if kind_value.as_str() == Some("struct") {
                                        // 提取fields
                                        if let Some(fields_value) = type_map.get("fields") {
                                            log::debug!("🔍 从type.fields提取账户字段");
                                            let mut parsed_fields = Vec::new();
                                            
                                            if let serde_json::Value::Array(fields_array) = fields_value {
                                                for field_value in fields_array {
                                                    if let serde_json::Value::Object(field_obj) = field_value {
                                                        if let (Some(field_name), Some(field_type_value)) = (
                                                            field_obj.get("name").and_then(|v| v.as_str()),
                                                            field_obj.get("type")
                                                        ) {
                                                            let field_type = AnchorFieldType::parse_value(field_type_value.clone())
                                                                .map_err(de::Error::custom)?;
                                                            
                                                            let field_docs = field_obj.get("docs")
                                                                .and_then(|v| v.as_array())
                                                                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
                                                            
                                                            parsed_fields.push(AnchorField {
                                                                name: field_name.to_string(),
                                                                field_type,
                                                                kind: None,
                                                                docs: field_docs,
                                                            });
                                                        }
                                                    }
                                                }
                                            }
                                            
                                            if !parsed_fields.is_empty() {
                                                fields = Some(parsed_fields);
                                                log::debug!("✅ 成功从type.fields解析到 {} 个字段", fields.as_ref().unwrap().len());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "docs" => {
                            if docs.is_some() {
                                return Err(de::Error::duplicate_field("docs"));
                            }
                            docs = Some(map.next_value()?);
                        }
                        _ => {
                            // 忽略未知字段
                            let _: serde_json::Value = map.next_value()?;
                        }
                    }
                }

                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let discriminator = discriminator.ok_or_else(|| de::Error::missing_field("discriminator"))?;

                Ok(AnchorAccount {
                    name,
                    discriminator,
                    fields,
                    docs,
                })
            }
        }

        deserializer.deserialize_map(AnchorAccountVisitor)
    }
}

/// PDA Seed 类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PdaSeed {
    /// 常量seed
    #[serde(rename = "const")]
    Const {
        value: Vec<u8>,
    },
    /// 账户字段引用
    #[serde(rename = "account")]
    Account {
        path: String,
    },
    /// 指令参数引用
    #[serde(rename = "arg")]
    Arg {
        path: String,
    },
}

/// PDA定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdaDefinition {
    /// Seeds用于生成PDA
    pub seeds: Vec<PdaSeed>,
    /// 可选的程序ID（用于跨程序PDA）
    pub program: Option<PdaSeed>,
}

/// Anchor账户约束
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorAccountConstraint {
    /// 账户名称
    pub name: String,
    /// 是否可变 - 支持writable, is_write, is_mut, isMut等多种命名
    #[serde(alias = "writable", alias = "is_write", alias = "isMut", default)]
    pub is_mut: bool,
    /// 是否签名者 - 映射到标准字段名，支持signer, isSigner等
    #[serde(alias = "signer", alias = "isSigner", default)]
    pub is_signer: bool,
    /// 是否可选
    pub is_optional: Option<bool>,
    /// 约束条件
    pub constraints: Option<Vec<String>>,
    /// 文档注释
    pub docs: Option<Vec<String>>,
    /// PDA定义（如果该账户是PDA）
    pub pda: Option<PdaDefinition>,
}

/// Anchor类型定义
#[derive(Debug, Clone, Serialize)]
pub struct AnchorType {
    /// 类型名称
    pub name: String,
    /// 类型种类（Optional for Legacy compatibility）
    pub kind: Option<AnchorTypeKind>,
    /// 文档注释
    pub docs: Option<Vec<String>>,
}

impl<'de> serde::Deserialize<'de> for AnchorType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct AnchorTypeVisitor;

        impl<'de> Visitor<'de> for AnchorTypeVisitor {
            type Value = AnchorType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a JSON object representing an AnchorType")
            }

            fn visit_map<V>(self, mut map: V) -> Result<AnchorType, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name = None;
                let mut docs = None;
                let mut type_info = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "name" => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        "docs" => {
                            if docs.is_some() {
                                return Err(de::Error::duplicate_field("docs"));
                            }
                            docs = Some(map.next_value()?);
                        }
                        "type" => {
                            if type_info.is_some() {
                                return Err(de::Error::duplicate_field("type"));
                            }
                            // 解析嵌套的 type 对象
                            let type_obj: serde_json::Value = map.next_value()?;
                            
                            // 从 type 对象中提取 kind 和 fields
                            if let serde_json::Value::Object(ref type_map) = type_obj {
                                let kind_str = type_map.get("kind")
                                    .and_then(|v| v.as_str());
                                
                                let kind = match kind_str {
                                    Some("struct") => {
                                        if let Some(fields_value) = type_map.get("fields") {
                                            // 手动解析字段数组以处理简化的JSON格式
                                            let mut fields = Vec::new();
                                            if let serde_json::Value::Array(fields_array) = fields_value {
                                                for field_value in fields_array {
                                                    if let serde_json::Value::Object(field_obj) = field_value {
                                                        if let (Some(name), Some(type_value)) = (
                                                            field_obj.get("name").and_then(|v| v.as_str()),
                                                            field_obj.get("type")
                                                        ) {
                                                            let field_type = AnchorFieldType::parse_value(type_value.clone())
                                                                .map_err(de::Error::custom)?;
                                                            
                                                            let docs = field_obj.get("docs")
                                                                .and_then(|v| v.as_array())
                                                                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
                                                            
                                                            fields.push(AnchorField {
                                                                name: name.to_string(),
                                                                field_type,
                                                                kind: None, // 对于结构体字段，kind通常为None
                                                                docs,
                                                            });
                                                        }
                                                    }
                                                }
                                            }
                                            Some(AnchorTypeKind::Struct(fields))
                                        } else {
                                            Some(AnchorTypeKind::Struct(Vec::new()))
                                        }
                                    },
                                    Some("enum") => {
                                        if let Some(variants_value) = type_map.get("variants") {
                                            // 手动解析枚举变体数组以处理简化的JSON格式
                                            let mut variants = Vec::new();
                                            if let serde_json::Value::Array(variants_array) = variants_value {
                                                for variant_value in variants_array {
                                                    if let serde_json::Value::Object(variant_obj) = variant_value {
                                                        if let Some(name) = variant_obj.get("name").and_then(|v| v.as_str()) {
                                                            let docs = variant_obj.get("docs")
                                                                .and_then(|v| v.as_array())
                                                                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
                                                            
                                                            // 解析变体字段（如果存在）
                                                            let fields = if let Some(fields_value) = variant_obj.get("fields") {
                                                                let mut variant_fields = Vec::new();
                                                                if let serde_json::Value::Array(fields_array) = fields_value {
                                                                    for field_value in fields_array {
                                                                        if let serde_json::Value::Object(field_obj) = field_value {
                                                                            if let (Some(field_name), Some(field_type_value)) = (
                                                                                field_obj.get("name").and_then(|v| v.as_str()),
                                                                                field_obj.get("type")
                                                                            ) {
                                                                                let field_type = AnchorFieldType::parse_value(field_type_value.clone())
                                                                                    .map_err(de::Error::custom)?;
                                                                                
                                                                                let field_docs = field_obj.get("docs")
                                                                                    .and_then(|v| v.as_array())
                                                                                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
                                                                                
                                                                                variant_fields.push(AnchorField {
                                                                                    name: field_name.to_string(),
                                                                                    field_type,
                                                                                    kind: None,
                                                                                    docs: field_docs,
                                                                                });
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                Some(variant_fields)
                                                            } else {
                                                                None
                                                            };
                                                            
                                                            variants.push(AnchorEnumVariant {
                                                                name: name.to_string(),
                                                                fields,
                                                                docs,
                                                            });
                                                        }
                                                    }
                                                }
                                            }
                                            Some(AnchorTypeKind::Enum(variants))
                                        } else {
                                            Some(AnchorTypeKind::Enum(Vec::new()))
                                        }
                                    },
                                    _ => {
                                        // 对于其他情况，尝试解析为 Alias
                                        let field_type = AnchorFieldType::parse_value(type_obj.clone())
                                            .map_err(de::Error::custom)?;
                                        Some(AnchorTypeKind::Alias(field_type))
                                    }
                                };
                                
                                type_info = Some(kind);
                            } else {
                                // 如果不是对象，直接尝试解析为 AnchorFieldType
                                let field_type = AnchorFieldType::parse_value(type_obj)
                                    .map_err(de::Error::custom)?;
                                type_info = Some(Some(AnchorTypeKind::Alias(field_type)));
                            }
                        }
                        _ => {
                            // 忽略未知字段
                            let _: serde_json::Value = map.next_value()?;
                        }
                    }
                }

                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                
                Ok(AnchorType {
                    name,
                    kind: type_info.flatten(),
                    docs,
                })
            }
        }

        deserializer.deserialize_map(AnchorTypeVisitor)
    }
}

/// Anchor类型种类  
#[derive(Debug, Clone, Serialize)]
pub enum AnchorTypeKind {
    /// 结构体（兼容Legacy系统的元组形式）
    Struct(Vec<AnchorField>),
    /// 枚举（兼容Legacy系统的元组形式）  
    Enum(Vec<AnchorEnumVariant>),
    /// 类型别名（兼容Legacy系统的元组形式）
    Alias(AnchorFieldType),
}

impl AnchorTypeKind {
    /// 兼容Legacy系统 - 创建alias变体
    pub fn alias(field_type: AnchorFieldType) -> Self {
        Self::Alias(field_type)
    }
}

/// Anchor枚举变体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorEnumVariant {
    /// 变体名称
    pub name: String,
    /// 变体字段（如果有）
    pub fields: Option<Vec<AnchorField>>,
    /// 文档注释
    pub docs: Option<Vec<String>>,
}

/// Anchor字段定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorField {
    /// 字段名称
    pub name: String,
    /// 字段类型
    #[serde(rename = "type")]
    pub field_type: AnchorFieldType,
    /// 字段种类（兼容Legacy系统）
    pub kind: Option<String>,
    /// 文档注释
    pub docs: Option<Vec<String>>,
}

impl AnchorField {
    /// 创建新字段
    pub fn new(name: String, field_type: AnchorFieldType) -> Self {
        Self {
            name,
            field_type,
            kind: None,
            docs: None,
        }
    }

    /// 设置kind字段（兼容Legacy系统）
    pub fn with_kind(mut self, kind: String) -> Self {
        self.kind = Some(kind);
        self
    }
}

/// Anchor字段类型
#[derive(Debug, Clone, Serialize)]
pub enum AnchorFieldType {
    /// 基础类型（字符串形式）
    Basic(String),
    /// 复合类型
    Complex {
        /// 类型种类
        kind: String,
        /// 泛型参数或数组大小等
        params: Option<Vec<serde_json::Value>>,
    },
    /// Legacy兼容 - 原始类型或公钥
    PrimitiveOrPubkey(String),
    /// Legacy兼容 - 已定义类型
    #[serde(rename = "defined")]
    #[allow(non_camel_case_types)]
    defined(String),
    /// Legacy兼容 - 数组类型
    #[serde(rename = "array")]
    #[allow(non_camel_case_types)]
    array(Box<AnchorFieldType>, usize),
    /// Legacy兼容 - 向量类型
    #[serde(rename = "vec")]
    #[allow(non_camel_case_types)]
    vec(Box<AnchorFieldType>),
    /// Legacy兼容 - 可选类型
    #[serde(rename = "option")]
    #[allow(non_camel_case_types)]
    option(Box<AnchorFieldType>),
}


/// Anchor事件定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorEvent {
    /// 事件名称
    pub name: String,
    /// 事件discriminator（固定8字节）
    pub discriminator: [u8; 8],
    /// 事件字段
    pub fields: Option<Vec<AnchorField>>,
    /// 文档注释
    pub docs: Option<Vec<String>>,
}

/// Anchor错误定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorError {
    /// 错误名称
    pub name: String,
    /// 错误码
    pub code: u32,
    /// 错误消息(可选，为了兼容不同版本的Anchor格式)
    pub msg: Option<String>,
    /// 文档注释
    pub docs: Option<Vec<String>>,
}

/// Anchor常量定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorConstant {
    /// 常量名称
    pub name: String,
    /// 常量类型
    #[serde(rename = "type")]
    pub const_type: String,
    /// 常量值
    pub value: serde_json::Value,
    /// 文档注释
    pub docs: Option<Vec<String>>,
}

impl AnchorIdl {
    /// 手动解析Anchor IDL JSON字符串
    pub fn parse_json(json_str: &str) -> Result<Self, serde_json::Error> {
        log::debug!("🔧 使用手动解析器处理Anchor IDL");
        
        // 首先解析为通用JSON值
        let json_value: serde_json::Value = serde_json::from_str(json_str)?;
        
        let obj = json_value.as_object().ok_or_else(|| {
            serde_json::Error::custom("IDL must be a JSON object")
        })?;
        
        // 获取address字段（优先从metadata.address获取，其次从顶级address）
        let address = obj.get("metadata")
            .and_then(|m| m.as_object())
            .and_then(|m| m.get("address"))
            .and_then(|v| v.as_str())
            .or_else(|| obj.get("address").and_then(|v| v.as_str()))
            .ok_or_else(|| {
                log::debug!("❌ Anchor IDL缺少address字段（在metadata.address或顶级address）");
                serde_json::Error::custom("Missing required field: address")
            })?
            .to_string();
            
        // 解析metadata（必需但可能缺少某些字段）
        let metadata = if let Some(metadata_obj) = obj.get("metadata") {
            log::debug!("✅ 发现metadata字段，正在解析");
            serde_json::from_value(metadata_obj.clone())?
        } else {
            log::debug!("⚠️ 缺少metadata字段，使用默认值");
            // 如果没有metadata，尝试从根级别获取name和version
            let name = obj.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let version = obj.get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0")
                .to_string();
                
            AnchorMetadata {
                address: Some(address.clone()),
                name: name.clone(),
                version: version.clone(),
                spec: "anchor".to_string(),
                description: Some("Generated metadata".to_string()),
            }
        };
        
        // 获取根级别的name（可选，优先使用），如果没有则从metadata获取
        let name = obj.get("name")
            .and_then(|v| v.as_str())
            .map(|s| {
                log::debug!("🔍 从根级别获取name: {}", s);
                s.to_string()
            })
            .or_else(|| {
                log::debug!("🔍 根级别没有name字段，从metadata.name获取: {}", metadata.name);
                Some(metadata.name.clone())
            });
            
        let version = obj.get("version")
            .and_then(|v| v.as_str())
            .map(|s| {
                log::debug!("🔍 从根级别获取version: {}", s);
                s.to_string()
            })
            .or_else(|| {
                log::debug!("🔍 根级别没有version字段，从metadata.version获取: {}", metadata.version);
                Some(metadata.version.clone())
            });
        
        // 恢复复杂字段的解析，每个字段都有详细日志
        log::debug!("🔄 开始解析Anchor复杂字段，每个字段单独处理");
        
        // 解析instructions字段
        log::debug!("📋 正在解析Anchor instructions字段...");
        let instructions: Option<Vec<AnchorInstruction>> = obj.get("instructions")
            .map(|v| {
                log::debug!("📋 发现Anchor instructions字段，包含 {} 个指令", 
                    v.as_array().map_or(0, |arr| arr.len()));
                log::debug!("📋 Anchor instructions JSON片段: {}", 
                    serde_json::to_string(v).unwrap_or_default().chars().take(200).collect::<String>());
                serde_json::from_value(v.clone())
            })
            .transpose()?;
        log::debug!("✅ Anchor instructions解析完成，结果: {}", 
            instructions.as_ref().map_or(0, |v| v.len()));
            
        // 解析accounts字段
        log::debug!("🏦 正在解析Anchor accounts字段...");
        let accounts: Option<Vec<AnchorAccount>> = obj.get("accounts")
            .map(|v| {
                log::debug!("🏦 发现Anchor accounts字段，包含 {} 个账户", 
                    v.as_array().map_or(0, |arr| arr.len()));
                log::debug!("🏦 Anchor accounts JSON片段: {}", 
                    serde_json::to_string(v).unwrap_or_default().chars().take(200).collect::<String>());
                serde_json::from_value(v.clone())
            })
            .transpose()?;
        log::debug!("✅ Anchor accounts解析完成，结果: {}", 
            accounts.as_ref().map_or(0, |v| v.len()));
            
        // 解析types字段
        log::debug!("🔧 正在解析Anchor types字段...");
        let types: Option<Vec<AnchorType>> = obj.get("types")
            .map(|v| {
                log::debug!("🔧 发现Anchor types字段，包含 {} 个类型", 
                    v.as_array().map_or(0, |arr| arr.len()));
                log::debug!("🔧 Anchor types JSON片段: {}", 
                    serde_json::to_string(v).unwrap_or_default().chars().take(200).collect::<String>());
                serde_json::from_value(v.clone())
            })
            .transpose()?;
        log::debug!("✅ Anchor types解析完成，结果: {}", 
            types.as_ref().map_or(0, |v| v.len()));
            
        // 解析events字段
        log::debug!("🎉 正在解析Anchor events字段...");
        let events: Option<Vec<AnchorEvent>> = obj.get("events")
            .map(|v| {
                log::debug!("🎉 发现Anchor events字段，包含 {} 个事件", 
                    v.as_array().map_or(0, |arr| arr.len()));
                log::debug!("🎉 Anchor events JSON片段: {}", 
                    serde_json::to_string(v).unwrap_or_default().chars().take(200).collect::<String>());
                serde_json::from_value(v.clone())
            })
            .transpose()?;
        log::debug!("✅ Anchor events解析完成，结果: {}", 
            events.as_ref().map_or(0, |v| v.len()));
            
        // 解析errors字段
        log::debug!("⚠️ 正在解析Anchor errors字段...");
        let errors: Option<Vec<AnchorError>> = obj.get("errors")
            .map(|v| {
                log::debug!("⚠️ 发现Anchor errors字段，包含 {} 个错误", 
                    v.as_array().map_or(0, |arr| arr.len()));
                log::debug!("⚠️ Anchor errors JSON片段: {}", 
                    serde_json::to_string(v).unwrap_or_default().chars().take(200).collect::<String>());
                serde_json::from_value(v.clone())
            })
            .transpose()?;
        log::debug!("✅ Anchor errors解析完成，结果: {}", 
            errors.as_ref().map_or(0, |v| v.len()));
            
        // 解析constants字段
        log::debug!("🔢 正在解析Anchor constants字段...");
        let constants: Option<Vec<AnchorConstant>> = obj.get("constants")
            .map(|v| {
                log::debug!("🔢 发现Anchor constants字段，包含 {} 个常量", 
                    v.as_array().map_or(0, |arr| arr.len()));
                log::debug!("🔢 Anchor constants JSON片段: {}", 
                    serde_json::to_string(v).unwrap_or_default().chars().take(200).collect::<String>());
                serde_json::from_value(v.clone())
            })
            .transpose()?;
        log::debug!("✅ Anchor constants解析完成，结果: {}", 
            constants.as_ref().map_or(0, |v| v.len()));
        
        log::debug!("✅ Anchor手动解析完成 - name: {:?}, version: {:?}", name, version);
        
        Ok(AnchorIdl {
            name,
            version,
            address,
            metadata,
            instructions,
            accounts,
            types,
            events,
            errors,
            constants,
            // field_allocation_cache: OnceLock::new(),
        })
    }

    /// 创建一个空的AnchorIdl
    pub fn empty(name: String, address: String) -> Self {
        Self {
            name: Some(name.clone()),
            version: Some("0.1.0".to_string()),
            address: address.clone(),
            metadata: AnchorMetadata {
                address: Some(address.clone()),
                name: name.clone(),
                version: "0.1.0".to_string(),
                spec: "anchor".to_string(),
                description: None,
            },
            instructions: None,
            accounts: None,
            types: None,
            events: None,
            errors: None,
            constants: None,
            // field_allocation_cache: OnceLock::new(),
        }
    }

    /// 获取程序名称（兼容Legacy系统）
    pub fn program_name(&self) -> &str {
        // 优先使用metadata.name，如果metadata.name不存在则使用根级别name
        // 大多数Anchor IDL的name都在metadata中
        &self.metadata.name
    }
    
    /// 获取程序版本
    pub fn program_version(&self) -> &str {
        // 优先使用metadata.version，如果metadata.version不存在则使用根级别version
        // 大多数Anchor IDL的version都在metadata中
        &self.metadata.version
    }

    /// 获取指令列表（兼容Legacy系统）
    pub fn instructions(&self) -> Option<&Vec<AnchorInstruction>> {
        self.instructions.as_ref()
    }
}

impl AnchorIdl {
    /// 设置指令
    pub fn with_instructions(mut self, instructions: Vec<AnchorInstruction>) -> Self {
        self.instructions = Some(instructions);
        self
    }

    /// 设置账户
    pub fn with_accounts(mut self, accounts: Vec<AnchorAccount>) -> Self {
        self.accounts = Some(accounts);
        self
    }

    /// 设置类型
    pub fn with_types(mut self, types: Vec<AnchorType>) -> Self {
        self.types = Some(types);
        self
    }

    /// 设置事件
    pub fn with_events(mut self, events: Vec<AnchorEvent>) -> Self {
        self.events = Some(events);
        self
    }

    /// 设置错误
    pub fn with_errors(mut self, errors: Vec<AnchorError>) -> Self {
        self.errors = Some(errors);
        self
    }

    /// 设置常量
    pub fn with_constants(mut self, constants: Vec<AnchorConstant>) -> Self {
        self.constants = Some(constants);
        self
    }

    /// 检查是否为Anchor合约（总是返回true）
    pub fn is_anchor_contract(&self) -> bool {
        true
    }

    // ======= 字段存在判断方法 =======

    /// 检查指定账户是否有字段定义
    pub fn has_account_fields(&self, account_name: &str) -> bool {
        self.get_account_fields(account_name).is_some()
    }

    /// 检查指定事件是否有字段定义
    pub fn has_event_fields(&self, event_name: &str) -> bool {
        self.get_event_fields(event_name).is_some()
    }

    /// 获取指定账户的字段定义
    pub fn get_account_fields(&self, account_name: &str) -> Option<&Vec<AnchorField>> {
        if let Some(accounts) = &self.accounts {
            accounts.iter()
                .find(|account| account.name == account_name)
                .and_then(|account| account.fields.as_ref())
        } else {
            None
        }
    }

    /// 获取指定事件的字段定义
    pub fn get_event_fields(&self, event_name: &str) -> Option<&Vec<AnchorField>> {
        if let Some(events) = &self.events {
            events.iter()
                .find(|event| event.name == event_name)
                .and_then(|event| event.fields.as_ref())
        } else {
            None
        }
    }

    /// 获取指定账户对象
    pub fn get_account(&self, account_name: &str) -> Option<&AnchorAccount> {
        self.accounts.as_ref()?.iter().find(|account| account.name == account_name)
    }

    /// 获取指定事件对象
    pub fn get_event(&self, event_name: &str) -> Option<&AnchorEvent> {
        self.events.as_ref()?.iter().find(|event| event.name == event_name)
    }

    // ======= 字段分配缓存成员函数已移除 - 传统模板系统不再使用 =======

    // /// 获取字段分配结果（线程安全缓存）
    // pub fn get_field_allocation(&self) -> &crate::templates::field_analyzer::FieldAllocationMap {
    //     self.field_allocation_cache.get_or_init(|| {
    //         log::debug!("🔄 AnchorIdl: 初始化字段分配缓存");
    //         crate::templates::field_analyzer::FieldAllocationAnalyzer::analyze_anchor_idl(self)
    //     })
    // }

    // /// 获取指定事件的字段分配结果
    // pub fn get_event_allocated_fields(&self, event_name: &str) -> Option<&Vec<crate::templates::field_analyzer::FieldDefinition>> {
    //     let allocation = self.get_field_allocation();
    //     allocation.events_fields.get(event_name)
    // }

    // /// 获取指定账户的字段分配结果
    // pub fn get_account_allocated_fields(&self, account_name: &str) -> Option<&Vec<crate::templates::field_analyzer::FieldDefinition>> {
    //     let allocation = self.get_field_allocation();
    //     allocation.accounts_fields.get(account_name)
    // }

    // /// 获取剩余类型名称列表（未被Events和Accounts使用的类型）
    // pub fn get_remaining_type_names(&self) -> Vec<String> {
    //     let allocation = self.get_field_allocation();
    //     crate::templates::field_analyzer::FieldAllocationAnalyzer::get_remaining_type_names(allocation)
    // }

    // /// 检查指定类型是否被Events或Accounts使用
    // pub fn is_type_allocated_to_modules(&self, type_name: &str) -> bool {
    //     let allocation = self.get_field_allocation();
    //     allocation.events_used_types.contains(type_name) || 
    //     allocation.accounts_used_types.contains(type_name)
    // }
}

use serde::de::{Deserializer, Error};
use std::sync::atomic::{AtomicUsize, Ordering};

// 递归深度监控
static ANCHOR_FIELD_TYPE_RECURSION_DEPTH: AtomicUsize = AtomicUsize::new(0);
static ANCHOR_TYPE_KIND_RECURSION_DEPTH: AtomicUsize = AtomicUsize::new(0);

impl<'de> serde::Deserialize<'de> for AnchorFieldType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        Self::parse_value(value).map_err(D::Error::custom)
    }
}

impl AnchorFieldType {
    fn parse_value(value: serde_json::Value) -> Result<Self, String> {
        // 递归深度监控
        let depth = ANCHOR_FIELD_TYPE_RECURSION_DEPTH.fetch_add(1, Ordering::SeqCst);
        
        if depth > 3000 {
            ANCHOR_FIELD_TYPE_RECURSION_DEPTH.fetch_sub(1, Ordering::SeqCst);
            return Err(format!("AnchorFieldType recursion too deep: {}", depth));
        }
        
        let result = match value {
            // 优先处理简单字符串类型，避免递归
            serde_json::Value::String(s) => {
                Ok(AnchorFieldType::Basic(s))
            }
            
            // 处理对象类型
            serde_json::Value::Object(map) => {
                
                // 检查 "defined" 类型 - 支持两种格式
                if let Some(defined_value) = map.get("defined") {
                    match defined_value {
                        // 格式1: {"defined": "TypeName"}
                        serde_json::Value::String(name) => {
                            log::trace!("✅ AnchorFieldType: defined({}) [string format]", name);
                            return Ok(AnchorFieldType::defined(name.clone()));
                        }
                        // 格式2: {"defined": {"name": "TypeName"}}
                        serde_json::Value::Object(defined_obj) => {
                            if let Some(serde_json::Value::String(name)) = defined_obj.get("name") {
                                log::trace!("✅ AnchorFieldType: defined({}) [object format]", name);
                                return Ok(AnchorFieldType::defined(name.clone()));
                            }
                        }
                        _ => {}
                    }
                }
                
                // 检查 "array" 类型
                if let Some(array_value) = map.get("array") {
                    log::trace!("🔍 Found array type, processing...");
                    if let serde_json::Value::Array(arr) = array_value {
                        if arr.len() == 2 {
                            log::trace!("🔄 RECURSION: Parsing array inner type: {:?}", 
                                     serde_json::to_string(&arr[0]).unwrap_or_default());
                            // 递归解析内部类型
                            let inner = Self::parse_value(arr[0].clone())?;
                            // 解析数组大小
                            let size = if let serde_json::Value::Number(n) = &arr[1] {
                                n.as_u64().unwrap_or(0) as usize
                            } else {
                                return Err("Array size must be a number".to_string());
                            };
                            log::trace!("✅ AnchorFieldType: array({:?}, {})", inner, size);
                            return Ok(AnchorFieldType::array(Box::new(inner), size));
                        }
                    }
                }
                
                // 检查 "vec" 类型
                if let Some(vec_value) = map.get("vec") {
                    log::trace!("🔍 Found vec type, processing...");
                    log::trace!("🔄 RECURSION: Parsing vec inner type: {:?}", 
                             serde_json::to_string(vec_value).unwrap_or_default());
                    let inner = Self::parse_value(vec_value.clone())?;
                    log::trace!("✅ AnchorFieldType: vec({:?})", inner);
                    return Ok(AnchorFieldType::vec(Box::new(inner)));
                }
                
                // 检查 "option" 类型
                if let Some(option_value) = map.get("option") {
                    log::trace!("🔍 Found option type, processing...");
                    log::trace!("🔄 RECURSION: Parsing option inner type: {:?}", 
                             serde_json::to_string(option_value).unwrap_or_default());
                    let inner = Self::parse_value(option_value.clone())?;
                    log::trace!("✅ AnchorFieldType: option({:?})", inner);
                    return Ok(AnchorFieldType::option(Box::new(inner)));
                }
                
                // 处理Complex类型
                if let Some(serde_json::Value::String(kind_str)) = map.get("kind") {
                    log::trace!("✅ AnchorFieldType: Complex(kind: {})", kind_str);
                    let params = map.get("params").map(|p| vec![p.clone()]);
                    return Ok(AnchorFieldType::Complex {
                        kind: kind_str.clone(),
                        params,
                    });
                }
                
                log::trace!("❌ Unknown AnchorFieldType object format with keys: {:?}", map.keys().collect::<Vec<_>>());
                Err("Unknown AnchorFieldType object format".to_string())
            }
            
            _ => {
                log::trace!("❌ Invalid AnchorFieldType format: {:?}", value);
                Err("Invalid AnchorFieldType format".to_string())
            }
        };
        
        // 递归深度计数器递减
        ANCHOR_FIELD_TYPE_RECURSION_DEPTH.fetch_sub(1, Ordering::SeqCst);
        log::trace!("📊 AnchorFieldType recursion depth after: {}", 
                 ANCHOR_FIELD_TYPE_RECURSION_DEPTH.load(Ordering::SeqCst));
        
        result
    }
}

impl<'de> serde::Deserialize<'de> for AnchorTypeKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        Self::parse_type_kind(value).map_err(D::Error::custom)
    }
}

impl AnchorTypeKind {
    fn parse_type_kind(value: serde_json::Value) -> Result<Self, String> {
        // 递归深度监控
        let depth = ANCHOR_TYPE_KIND_RECURSION_DEPTH.fetch_add(1, Ordering::SeqCst);
        log::trace!("📊 AnchorTypeKind recursion depth: {}", depth);
        
        if depth > 3000 {
            log::trace!("🚨 AnchorTypeKind RECURSION LIMIT EXCEEDED! Depth: {}", depth);
            ANCHOR_TYPE_KIND_RECURSION_DEPTH.fetch_sub(1, Ordering::SeqCst);
            return Err(format!("AnchorTypeKind recursion too deep: {}", depth));
        }
        
        log::trace!("🔍 AnchorTypeKind::parse_type_kind called with: {:?}", 
                 serde_json::to_string(&value).unwrap_or_default());
        
        let result = match value {
            // 尝试解析为结构体字段数组
            serde_json::Value::Array(ref arr) => {
                // 尝试解析为AnchorField数组（Struct变体）
                let mut fields = Vec::new();
                for item in arr {
                    if let Ok(field) = Self::parse_anchor_field(item.clone()) {
                        fields.push(field);
                    } else {
                        // 如果不是AnchorField，可能是AnchorEnumVariant
                        return Self::parse_enum_variants(arr.clone());
                    }
                }
                Ok(AnchorTypeKind::Struct(fields))
            }
            
            // 单个AnchorFieldType（Alias变体）
            _ => {
                log::trace!("🔄 RECURSION: AnchorTypeKind parsing as Alias");
                let field_type = AnchorFieldType::parse_value(value)?;
                Ok(AnchorTypeKind::Alias(field_type))
            }
        };
        
        // 递归深度计数器递减
        ANCHOR_TYPE_KIND_RECURSION_DEPTH.fetch_sub(1, Ordering::SeqCst);
        log::trace!("📊 AnchorTypeKind recursion depth after: {}", 
                 ANCHOR_TYPE_KIND_RECURSION_DEPTH.load(Ordering::SeqCst));
        
        result
    }
    
    fn parse_anchor_field(value: serde_json::Value) -> Result<AnchorField, String> {
        match value {
            serde_json::Value::Object(ref map) => {
                let name = map.get("name")
                    .and_then(|v| v.as_str())
                    .ok_or("Field name is required")?
                    .to_string();
                
                let field_type_value = map.get("type")
                    .ok_or("Field type is required")?;
                let field_type = AnchorFieldType::parse_value(field_type_value.clone())?;
                
                let docs = map.get("docs")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
                
                Ok(AnchorField {
                    name,
                    field_type,
                    kind: None,
                    docs,
                })
            }
            _ => Err("AnchorField must be an object".to_string())
        }
    }
    
    fn parse_enum_variants(arr: Vec<serde_json::Value>) -> Result<AnchorTypeKind, String> {
        let mut variants = Vec::new();
        for item in arr {
            let variant = Self::parse_enum_variant(item)?;
            variants.push(variant);
        }
        Ok(AnchorTypeKind::Enum(variants))
    }
    
    fn parse_enum_variant(value: serde_json::Value) -> Result<AnchorEnumVariant, String> {
        match value {
            serde_json::Value::Object(ref map) => {
                let name = map.get("name")
                    .and_then(|v| v.as_str())
                    .ok_or("Enum variant name is required")?
                    .to_string();
                
                let fields = map.get("fields")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|item| Self::parse_anchor_field(item.clone()).ok())
                            .collect()
                    });
                
                let docs = map.get("docs")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
                
                Ok(AnchorEnumVariant {
                    name,
                    fields,
                    docs,
                })
            }
            _ => Err("AnchorEnumVariant must be an object".to_string())
        }
    }
}