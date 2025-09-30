//! 非Anchor IDL数据结构
//!
//! 定义专门用于非Anchor合约的统一IDL格式，支持1字节discriminator、
//! 自定义序列化格式等非Anchor特性

use serde::{Deserialize, Serialize, Deserializer};
use serde::de::Error;

/// 非Anchor合约的统一IDL格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonAnchorIdl {
    /// 程序名称（可选，优先使用，如果没有则从metadata获取）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 程序版本（可选，优先使用，如果没有则从metadata获取）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// 程序地址
    pub address: String,
    /// 元数据（可选，但通常存在）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<NonAnchorMetadata>,
    /// 指令定义
    pub instructions: Option<Vec<NonAnchorInstruction>>,
    /// 账户定义 - 支持嵌套type格式
    #[serde(deserialize_with = "deserialize_accounts_with_nested_type")]
    pub accounts: Option<Vec<NonAnchorAccount>>,
    /// 类型定义
    pub types: Option<Vec<NonAnchorType>>,
    /// 错误定义
    pub errors: Option<Vec<NonAnchorError>>,
    /// 事件定义
    pub events: Option<Vec<NonAnchorEvent>>,
    /// Discriminator大小（字节数）- 私有配置字段
    #[serde(skip, default = "default_discriminator_size")]
    discriminator_size: u8,
    /// 序列化格式 - 私有配置字段
    #[serde(skip, default)]
    serialization_format: SerializationFormat,
    // 字段分配缓存已移除 - 传统模板系统不再使用
    // #[serde(skip)]
    // field_allocation_cache: std::sync::OnceLock<crate::templates::field_analyzer::FieldAllocationMap>,
}

/// 非Anchor合约元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonAnchorMetadata {
    /// 程序地址（可选，通常在根级别）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// 程序名称
    pub name: String,
    /// 程序版本
    pub version: String,
    /// IDL规范类型（shank、bincode等）
    pub spec: String,
    /// 描述
    pub description: Option<String>,
}

/// 序列化格式枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializationFormat {
    /// Bincode序列化
    Bincode,
    /// Borsh序列化
    Borsh,
    /// 自定义序列化格式
    Custom(String),
}

impl Default for SerializationFormat {
    fn default() -> Self {
        Self::Borsh // NonAnchor合约常用Borsh序列化
    }
}

/// NonAnchor默认discriminator大小：1字节
fn default_discriminator_size() -> u8 {
    1
}

/// 非Anchor指令定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonAnchorInstruction {
    /// 指令名称
    pub name: String,
    /// 指令discriminator（可选，支持缺失discriminator的IDL）
    pub discriminator: Option<Vec<u8>>,
    /// 指令参数
    pub args: Option<Vec<NonAnchorField>>,
    /// 指令账户
    pub accounts: Option<Vec<NonAnchorAccount>>,
    /// 文档注释
    pub docs: Option<Vec<String>>,
}

/// 非Anchor账户中间解析结构（支持嵌套type格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawNonAnchorAccount {
    /// 账户名称
    pub name: String,
    /// 是否可变 - 支持writable, is_write, is_mut等多种命名，默认false
    #[serde(alias = "writable", alias = "is_write", default)]
    pub is_mut: bool,
    /// 是否签名者 - 映射到标准字段名，默认false
    #[serde(alias = "signer", default)]
    pub is_signer: bool,
    /// 账户discriminator
    pub discriminator: Option<Vec<u8>>,
    /// 文档注释
    pub docs: Option<Vec<String>>,
    
    /// 直接字段格式支持
    pub fields: Option<Vec<NonAnchorField>>,
    
    /// 嵌套类型格式支持 {"type": {"kind": "struct", "fields": [...]}}
    #[serde(rename = "type")]
    pub type_def: Option<NonAnchorTypeKind>,
}

/// 非Anchor账户定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonAnchorAccount {
    /// 账户名称
    pub name: String,
    /// 是否可变 - 支持writable, is_write, is_mut等多种命名，默认false
    #[serde(alias = "writable", alias = "is_write", default)]
    pub is_mut: bool,
    /// 是否签名者 - 映射到标准字段名，默认false
    #[serde(alias = "signer", default)]
    pub is_signer: bool,
    /// 账户discriminator
    pub discriminator: Option<Vec<u8>>,
    /// 账户字段
    pub fields: Option<Vec<NonAnchorField>>,
    /// 文档注释
    pub docs: Option<Vec<String>>,
}

/// 实现从RawNonAnchorAccount到NonAnchorAccount的智能转换
impl From<RawNonAnchorAccount> for NonAnchorAccount {
    fn from(raw: RawNonAnchorAccount) -> Self {
        // 智能字段提取：优先使用直接字段，然后尝试从嵌套type中提取
        let fields = raw.fields.or_else(|| {
            // 从嵌套的type.fields中提取字段
            if let Some(NonAnchorTypeKind::Struct { fields }) = raw.type_def {
                log::debug!("🔄 从嵌套type结构提取账户字段: {} -> {} 个字段", 
                    raw.name, fields.len());
                Some(fields)
            } else {
                log::debug!("❌ 账户 {} 无字段定义（非Struct类型或无type定义）", raw.name);
                None
            }
        });
        
        NonAnchorAccount {
            name: raw.name,
            is_mut: raw.is_mut,
            is_signer: raw.is_signer,
            discriminator: raw.discriminator,
            fields,
            docs: raw.docs,
        }
    }
}

/// 非Anchor类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonAnchorType {
    /// 类型名称（可选，兼容缺少name字段的IDL）
    #[serde(default = "default_type_name")]
    pub name: String,
    /// 类型定义（嵌套结构，包含kind和fields）
    #[serde(rename = "type")]
    pub type_def: NonAnchorTypeKind,
    /// 文档注释
    pub docs: Option<Vec<String>>,
}

/// 默认类型名称
fn default_type_name() -> String {
    "UnnamedType".to_string()
}

/// 非Anchor类型种类
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum NonAnchorTypeKind {
    /// 结构体
    #[serde(rename = "struct")]
    Struct {
        fields: Vec<NonAnchorField>,
    },
    /// 枚举
    #[serde(rename = "enum")]
    Enum {
        variants: Vec<NonAnchorEnumVariant>,
    },
    /// 类型别名
    #[serde(rename = "alias")]
    Alias {
        value: NonAnchorFieldType,
    },
}

/// 非Anchor枚举变体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonAnchorEnumVariant {
    /// 变体名称（可选，兼容缺少name字段的IDL）
    #[serde(default = "default_variant_name")]
    pub name: String,
    /// 变体字段（如果有）
    pub fields: Option<Vec<NonAnchorField>>,
    /// 文档注释
    pub docs: Option<Vec<String>>,
}

/// 默认变体名称
fn default_variant_name() -> String {
    "UnnamedVariant".to_string()
}

/// 默认字段名称
fn default_field_name() -> String {
    "unnamed_field".to_string()
}

/// 非Anchor字段定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonAnchorField {
    /// 字段名称（可选，兼容缺少name字段的IDL）
    #[serde(default = "default_field_name")]
    pub name: String,
    /// 字段类型
    #[serde(rename = "type")]
    pub field_type: NonAnchorFieldType,
    /// 文档注释
    pub docs: Option<Vec<String>>,
}

/// 非Anchor字段类型
#[derive(Debug, Clone, Serialize)]
pub enum NonAnchorFieldType {
    /// 基础类型（字符串形式）
    Basic(String),
    /// 可选类型：{"option": "内部类型"}
    Option {
        option: Box<NonAnchorFieldType>,
    },
    /// 向量类型：{"vec": "内部类型"}
    Vec {
        vec: Box<NonAnchorFieldType>,
    },
    /// 数组类型：{"array": ["内部类型", 大小]}
    Array {
        array: (Box<NonAnchorFieldType>, usize),
    },
    /// 已定义类型：{"defined": "类型名"}
    Defined {
        defined: String,
    },
    /// 哈希映射类型：{"hashMap": ["key类型", "value类型"]}
    HashMap {
        key: Box<NonAnchorFieldType>,
        value: Box<NonAnchorFieldType>,
    },
    /// 复合类型
    Complex {
        /// 类型种类
        kind: String,
        /// 泛型参数或数组大小等
        params: Option<Vec<serde_json::Value>>,
    },
}

/// 非Anchor错误定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonAnchorError {
    /// 错误名称
    pub name: String,
    /// 错误码
    pub code: u32,
    /// 错误消息
    pub msg: String,
    /// 文档注释
    pub docs: Option<Vec<String>>,
}

/// 非Anchor事件定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonAnchorEvent {
    /// 事件名称
    pub name: String,
    /// 事件discriminator（可选，支持缺失discriminator的事件）
    pub discriminator: Option<Vec<u8>>,
    /// 事件字段
    pub fields: Option<Vec<NonAnchorField>>,
    /// 文档注释
    pub docs: Option<Vec<String>>,
}

impl NonAnchorIdl {
    // ======= 私有字段访问器方法 =======
    
    /// 获取discriminator大小（字节数）
    pub fn discriminator_size(&self) -> u8 {
        self.discriminator_size
    }
    
    /// 获取序列化格式
    pub fn serialization_format(&self) -> &SerializationFormat {
        &self.serialization_format
    }
    
    /// 设置discriminator大小（Builder模式）
    pub fn with_discriminator_size(mut self, size: u8) -> Self {
        self.discriminator_size = size;
        self
    }
    
    /// 设置序列化格式（Builder模式）
    pub fn with_serialization_format(mut self, format: SerializationFormat) -> Self {
        self.serialization_format = format;
        self
    }
    
    /// 检查是否使用1字节discriminator
    pub fn is_single_byte_discriminator(&self) -> bool {
        self.discriminator_size == 1
    }
    
    /// 检查是否使用Borsh序列化
    pub fn uses_borsh_serialization(&self) -> bool {
        matches!(self.serialization_format, SerializationFormat::Borsh)
    }
    
    // ======= 原有方法 =======
    
    /// 创建一个空的NonAnchorIdl
    pub fn empty(name: String, address: String) -> Self {
        Self {
            name: Some(name.clone()),
            version: Some("0.1.0".to_string()),
            address: address.clone(),
            metadata: Some(NonAnchorMetadata {
                address: Some(address),
                name,
                version: "0.1.0".to_string(),
                spec: "non-anchor".to_string(),
                description: None,
            }),
            instructions: None,
            accounts: None,
            types: None,
            errors: None,
            events: None,
            discriminator_size: 1, // 默认1字节
            serialization_format: SerializationFormat::Borsh, // 默认Borsh
            // field_allocation_cache: std::sync::OnceLock::new(),
        }
    }



    /// 设置指令
    pub fn with_instructions(mut self, instructions: Vec<NonAnchorInstruction>) -> Self {
        self.instructions = Some(instructions);
        self
    }

    /// 获取指令列表（兼容Legacy系统）
    pub fn instructions(&self) -> &[NonAnchorInstruction] {
        self.instructions.as_deref().unwrap_or(&[])
    }

    /// 获取程序名称（智能获取：优先使用根级别name，fallback到metadata.name）
    pub fn program_name(&self) -> &str {
        // 优先使用根级别的name
        if let Some(ref name) = self.name {
            return name;
        }
        
        // fallback到metadata.name
        if let Some(ref metadata) = self.metadata {
            return &metadata.name;
        }
        
        // 最后fallback
        "unknown"
    }
    
    /// 获取程序版本（智能获取：优先使用根级别version，fallback到metadata.version）
    pub fn program_version(&self) -> &str {
        // 优先使用根级别的version
        if let Some(ref version) = self.version {
            return version;
        }
        
        // fallback到metadata.version
        if let Some(ref metadata) = self.metadata {
            return &metadata.version;
        }
        
        // 最后fallback
        "0.0.0"
    }
}

impl NonAnchorIdl {
    /// 设置账户
    pub fn with_accounts(mut self, accounts: Vec<NonAnchorAccount>) -> Self {
        self.accounts = Some(accounts);
        self
    }

    /// 设置类型
    pub fn with_types(mut self, types: Vec<NonAnchorType>) -> Self {
        self.types = Some(types);
        self
    }

    /// 设置错误
    pub fn with_errors(mut self, errors: Vec<NonAnchorError>) -> Self {
        self.errors = Some(errors);
        self
    }

    /// 设置事件
    pub fn with_events(mut self, events: Vec<NonAnchorEvent>) -> Self {
        self.events = Some(events);
        self
    }


    /// 获取指定账户对象
    pub fn get_account(&self, account_name: &str) -> Option<&NonAnchorAccount> {
        self.accounts.as_ref()?.iter().find(|account| account.name == account_name)
    }

    /// 获取指定事件对象  
    pub fn get_event(&self, event_name: &str) -> Option<&NonAnchorEvent> {
        self.events.as_ref()?.iter().find(|event| event.name == event_name)
    }
}

impl NonAnchorInstruction {
    /// 获取指令的discriminator，如果缺失则根据索引生成1字节discriminator
    pub fn get_discriminator_with_fallback(&self, instruction_index: usize) -> Vec<u8> {
        if let Some(ref discriminator) = self.discriminator {
            discriminator.clone()
        } else {
            // 为缺失discriminator的指令生成基于索引的1字节discriminator
            vec![(instruction_index as u8)]
        }
    }
    
    /// 检查是否有显式定义的discriminator
    pub fn has_explicit_discriminator(&self) -> bool {
        self.discriminator.is_some()
    }
}

use std::sync::atomic::{AtomicUsize, Ordering};

// NonAnchor字段类型递归深度监控
static NON_ANCHOR_FIELD_TYPE_RECURSION_DEPTH: AtomicUsize = AtomicUsize::new(0);

impl<'de> serde::Deserialize<'de> for NonAnchorFieldType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        Self::parse_value(value).map_err(D::Error::custom)
    }
}

impl NonAnchorFieldType {
    fn parse_value(value: serde_json::Value) -> Result<Self, String> {
        // 递归深度监控
        let depth = NON_ANCHOR_FIELD_TYPE_RECURSION_DEPTH.fetch_add(1, Ordering::SeqCst);
        
        if depth > 3000 {
            NON_ANCHOR_FIELD_TYPE_RECURSION_DEPTH.fetch_sub(1, Ordering::SeqCst);
            return Err(format!("NonAnchorFieldType recursion too deep: {}", depth));
        }
        
        log::trace!("📊 NonAnchorFieldType recursion depth: {}", depth);
        log::trace!("🔍 NonAnchorFieldType::parse_value called with: {:?}", 
                 serde_json::to_string(&value).unwrap_or_default());
        
        let result = match value {
            // 优先处理简单字符串类型，避免递归
            serde_json::Value::String(s) => {
                log::trace!("✅ NonAnchorFieldType: Basic({})", s);
                Ok(NonAnchorFieldType::Basic(s))
            }
            
            // 处理对象类型
            serde_json::Value::Object(map) => {
                
                // 检查 "option" 类型 - {"option": "内部类型"}
                if let Some(option_value) = map.get("option") {
                    log::trace!("🔍 Found option type, processing...");
                    log::trace!("🔄 RECURSION: Parsing option inner type: {:?}", 
                             serde_json::to_string(option_value).unwrap_or_default());
                    let inner = Self::parse_value(option_value.clone())?;
                    log::trace!("✅ NonAnchorFieldType: Option({:?})", inner);
                    return Ok(NonAnchorFieldType::Option {
                        option: Box::new(inner),
                    });
                }
                
                // 检查 "vec" 类型 - {"vec": "内部类型"}
                if let Some(vec_value) = map.get("vec") {
                    log::trace!("🔍 Found vec type, processing...");
                    log::trace!("🔄 RECURSION: Parsing vec inner type: {:?}", 
                             serde_json::to_string(vec_value).unwrap_or_default());
                    let inner = Self::parse_value(vec_value.clone())?;
                    log::trace!("✅ NonAnchorFieldType: Vec({:?})", inner);
                    return Ok(NonAnchorFieldType::Vec {
                        vec: Box::new(inner),
                    });
                }
                
                // 检查 "array" 类型 - {"array": ["内部类型", 大小]}
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
                            log::trace!("✅ NonAnchorFieldType: Array({:?}, {})", inner, size);
                            return Ok(NonAnchorFieldType::Array {
                                array: (Box::new(inner), size),
                            });
                        } else {
                            return Err("Array must have exactly 2 elements [type, size]".to_string());
                        }
                    }
                }
                
                // 检查 "hashMap" 类型 - {"hashMap": ["key类型", "value类型"]}
                if let Some(hashmap_value) = map.get("hashMap") {
                    log::trace!("🔍 Found hashMap type, processing...");
                    if let serde_json::Value::Array(arr) = hashmap_value {
                        if arr.len() == 2 {
                            log::trace!("🔄 RECURSION: Parsing hashMap key type: {:?}", 
                                     serde_json::to_string(&arr[0]).unwrap_or_default());
                            // 递归解析key类型
                            let key_type = Self::parse_value(arr[0].clone())?;
                            log::trace!("🔄 RECURSION: Parsing hashMap value type: {:?}", 
                                     serde_json::to_string(&arr[1]).unwrap_or_default());
                            // 递归解析value类型
                            let value_type = Self::parse_value(arr[1].clone())?;
                            log::trace!("✅ NonAnchorFieldType: HashMap({:?}, {:?})", key_type, value_type);
                            return Ok(NonAnchorFieldType::HashMap {
                                key: Box::new(key_type),
                                value: Box::new(value_type),
                            });
                        } else {
                            return Err("HashMap must have exactly 2 elements [key_type, value_type]".to_string());
                        }
                    }
                }
                
                // 检查 "defined" 类型 - {"defined": "类型名"}
                if let Some(defined_value) = map.get("defined") {
                    if let serde_json::Value::String(type_name) = defined_value {
                        log::trace!("✅ NonAnchorFieldType: Defined({})", type_name);
                        return Ok(NonAnchorFieldType::Defined {
                            defined: type_name.clone(),
                        });
                    } else {
                        return Err("Defined type name must be a string".to_string());
                    }
                }
                
                // 检查复合类型 - {"kind": "类型", "params": [...]}
                if let Some(serde_json::Value::String(kind_str)) = map.get("kind") {
                    log::trace!("✅ NonAnchorFieldType: Complex(kind: {})", kind_str);
                    let params = map.get("params").map(|p| vec![p.clone()]);
                    return Ok(NonAnchorFieldType::Complex {
                        kind: kind_str.clone(),
                        params,
                    });
                }
                
                log::trace!("❌ Unknown NonAnchorFieldType object format with keys: {:?}", 
                         map.keys().collect::<Vec<_>>());
                Err("Unknown NonAnchorFieldType object format".to_string())
            }
            
            _ => {
                log::trace!("❌ Invalid NonAnchorFieldType format: {:?}", value);
                Err("Invalid NonAnchorFieldType format".to_string())
            }
        };
        
        // 递归深度计数器递减
        NON_ANCHOR_FIELD_TYPE_RECURSION_DEPTH.fetch_sub(1, Ordering::SeqCst);
        log::trace!("📊 NonAnchorFieldType recursion depth after: {}", 
                 NON_ANCHOR_FIELD_TYPE_RECURSION_DEPTH.load(Ordering::SeqCst));
        
        result
    }
}

impl NonAnchorIdl {
    // ======= 字段分配缓存成员函数已移除 - 传统模板系统不再使用 =======

    // /// 获取字段分配结果（线程安全缓存）
    // pub fn get_field_allocation(&self) -> &crate::templates::field_analyzer::FieldAllocationMap {
    //     self.field_allocation_cache.get_or_init(|| {
    //         log::debug!("🔄 NonAnchorIdl: 初始化字段分配缓存");
    //         crate::templates::field_analyzer::FieldAllocationAnalyzer::analyze_non_anchor_idl(self)
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

    /// 获取指定事件的字段定义 (直接从IDL，不使用分配缓存)
    pub fn get_event_fields(&self, event_name: &str) -> Option<&Vec<NonAnchorField>> {
        if let Some(events) = &self.events {
            events.iter()
                .find(|event| event.name == event_name)
                .and_then(|event| event.fields.as_ref())
        } else {
            None
        }
    }

    /// 获取指定账户的字段定义 (直接从IDL，不使用分配缓存)
    pub fn get_account_fields(&self, account_name: &str) -> Option<&Vec<NonAnchorField>> {
        if let Some(accounts) = &self.accounts {
            accounts.iter()
                .find(|account| account.name == account_name)
                .and_then(|account| account.fields.as_ref())
        } else {
            None
        }
    }

    /// 检查指定账户是否有直接字段定义
    pub fn has_account_fields(&self, account_name: &str) -> bool {
        self.get_account_fields(account_name).is_some()
    }

    /// 检查指定事件是否有直接字段定义
    pub fn has_event_fields(&self, event_name: &str) -> bool {
        self.get_event_fields(event_name).is_some()
    }
}

/// 自定义反序列化函数：处理嵌套type格式的accounts
fn deserialize_accounts_with_nested_type<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<NonAnchorAccount>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    
    let raw_accounts: Option<Vec<RawNonAnchorAccount>> = Option::deserialize(deserializer)?;
    Ok(raw_accounts.map(|accounts| {
        log::debug!("🔄 NonAnchor账户解析 - 处理 {} 个账户", accounts.len());
        
        let processed_accounts: Vec<NonAnchorAccount> = accounts
            .into_iter()
            .map(|raw| {
                let account_name = raw.name.clone();
                let has_direct_fields = raw.fields.is_some();
                let has_nested_type = raw.type_def.is_some();
                
                log::debug!("  - 账户 {}: 直接字段={}, 嵌套类型={}", 
                    account_name, has_direct_fields, has_nested_type);
                
                NonAnchorAccount::from(raw)
            })
            .collect();
            
        // 输出解析结果统计
        for account in &processed_accounts {
            let field_count = account.fields.as_ref().map_or(0, |f| f.len());
            log::debug!("  ✅ 账户 {} -> {} 个字段", account.name, field_count);
        }
        
        processed_accounts
    }))
}

impl NonAnchorIdl {
    /// 手动解析NonAnchor IDL JSON字符串
    pub fn parse_json(json_str: &str) -> Result<Self, serde_json::Error> {
        log::debug!("🔧 使用手动解析器处理NonAnchor IDL");
        
        // 首先解析为通用JSON值
        let json_value: serde_json::Value = serde_json::from_str(json_str)?;
        
        let obj = json_value.as_object().ok_or_else(|| {
            serde_json::Error::custom("IDL must be a JSON object")
        })?;
        
        // 智能获取name字段
        let name = obj.get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                // fallback到metadata.name
                obj.get("metadata")
                    .and_then(|m| m.as_object())
                    .and_then(|m| m.get("name"))
                    .and_then(|v| v.as_str())
                    .map(|s| {
                        log::debug!("🔄 从metadata.name获取程序名称: {}", s);
                        s.to_string()
                    })
            });
            
        // 智能获取version字段
        let version = obj.get("version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                // fallback到metadata.version
                obj.get("metadata")
                    .and_then(|m| m.as_object())
                    .and_then(|m| m.get("version"))
                    .and_then(|v| v.as_str())
                    .map(|s| {
                        log::debug!("🔄 从metadata.version获取程序版本: {}", s);
                        s.to_string()
                    })
            });
        
        // 获取address字段（优先从metadata.address获取，其次从顶级address）
        let address = obj.get("metadata")
            .and_then(|m| m.as_object())
            .and_then(|m| m.get("address"))
            .and_then(|v| v.as_str())
            .or_else(|| obj.get("address").and_then(|v| v.as_str()))
            .ok_or_else(|| {
                log::debug!("❌ NonAnchor IDL缺少address字段（在metadata.address或顶级address）");
                serde_json::Error::custom("Missing required field: address")
            })?
            .to_string();
            
        // 解析metadata（可选）
        let metadata: Option<NonAnchorMetadata> = obj.get("metadata")
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()?;
            
        // 恢复复杂字段的解析，每个字段都有详细日志
        log::debug!("🔄 开始解析复杂字段，每个字段单独处理");
        
        // 解析instructions字段
        log::debug!("📋 正在解析instructions字段...");
        let instructions: Option<Vec<NonAnchorInstruction>> = obj.get("instructions")
            .map(|v| {
                log::debug!("📋 发现instructions字段，包含 {} 个指令", 
                    v.as_array().map_or(0, |arr| arr.len()));
                log::debug!("📋 instructions JSON片段: {}", 
                    serde_json::to_string(v).unwrap_or_default().chars().take(200).collect::<String>());
                serde_json::from_value(v.clone())
            })
            .transpose()?;
        log::debug!("✅ instructions解析完成，结果: {}", 
            instructions.as_ref().map_or(0, |v| v.len()));
            
        // 解析accounts字段
        log::debug!("🏦 正在解析accounts字段...");
        let accounts: Option<Vec<RawNonAnchorAccount>> = obj.get("accounts")
            .map(|v| {
                log::debug!("🏦 发现accounts字段，包含 {} 个账户", 
                    v.as_array().map_or(0, |arr| arr.len()));
                log::debug!("🏦 accounts JSON片段: {}", 
                    serde_json::to_string(v).unwrap_or_default().chars().take(200).collect::<String>());
                serde_json::from_value(v.clone())
            })
            .transpose()?;
        log::debug!("✅ accounts解析完成，结果: {}", 
            accounts.as_ref().map_or(0, |v| v.len()));
        
        // 转换accounts格式
        let accounts = accounts.map(|raw_accounts| {
            log::debug!("🔄 正在转换 {} 个RawNonAnchorAccount为NonAnchorAccount", raw_accounts.len());
            raw_accounts.into_iter().map(|raw| NonAnchorAccount::from(raw)).collect()
        });
            
        // 解析types字段 - 使用手动解析避免递归类型冲突
        log::debug!("🔧 正在解析types字段...");
        let types: Option<Vec<NonAnchorType>> = obj.get("types")
            .map(|v| {
                log::debug!("🔧 发现types字段，包含 {} 个类型", 
                    v.as_array().map_or(0, |arr| arr.len()));
                log::debug!("🔧 types JSON片段: {}", 
                    serde_json::to_string(v).unwrap_or_default().chars().take(200).collect::<String>());
                Self::parse_types_manually(v)
            })
            .transpose()?;
        log::debug!("✅ types解析完成，结果: {}", 
            types.as_ref().map_or(0, |v| v.len()));
            
        // 解析errors字段
        log::debug!("⚠️ 正在解析errors字段...");
        let errors: Option<Vec<NonAnchorError>> = obj.get("errors")
            .map(|v| {
                log::debug!("⚠️ 发现errors字段，包含 {} 个错误", 
                    v.as_array().map_or(0, |arr| arr.len()));
                log::debug!("⚠️ errors JSON片段: {}", 
                    serde_json::to_string(v).unwrap_or_default().chars().take(200).collect::<String>());
                serde_json::from_value(v.clone())
            })
            .transpose()?;
        log::debug!("✅ errors解析完成，结果: {}", 
            errors.as_ref().map_or(0, |v| v.len()));
            
        // 解析events字段
        log::debug!("🎉 正在解析events字段...");
        let events: Option<Vec<NonAnchorEvent>> = obj.get("events")
            .map(|v| {
                log::debug!("🎉 发现events字段，包含 {} 个事件", 
                    v.as_array().map_or(0, |arr| arr.len()));
                log::debug!("🎉 events JSON片段: {}", 
                    serde_json::to_string(v).unwrap_or_default().chars().take(200).collect::<String>());
                serde_json::from_value(v.clone())
            })
            .transpose()?;
        log::debug!("✅ events解析完成，结果: {}", 
            events.as_ref().map_or(0, |v| v.len()));
        
        log::debug!("✅ 手动解析完成 - name: {:?}, version: {:?}", name, version);
        
        Ok(NonAnchorIdl {
            name,
            version,
            address,
            metadata,
            instructions,
            accounts,
            types,
            errors,
            events,
            discriminator_size: 1, // 默认1字节
            serialization_format: SerializationFormat::Borsh, // 默认Borsh
            // field_allocation_cache: std::sync::OnceLock::new(),
        })
    }

    /// 手动解析types数组，避免serde递归类型冲突
    fn parse_types_manually(types_value: &serde_json::Value) -> Result<Vec<NonAnchorType>, serde_json::Error> {
        log::debug!("🔧 开始手动解析types数组");
        
        let types_array = types_value.as_array().ok_or_else(|| {
            serde_json::Error::custom("types字段必须是数组")
        })?;
        
        let mut parsed_types = Vec::new();
        
        for (index, type_value) in types_array.iter().enumerate() {
            log::debug!("🔧 解析第{}个类型", index + 1);
            
            let type_obj = type_value.as_object().ok_or_else(|| {
                serde_json::Error::custom(format!("types[{}]必须是对象", index))
            })?;
            
            // 解析name字段
            let name = type_obj.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("UnnamedType")
                .to_string();
            
            // 解析type字段 - 这里是顶层type对象
            let type_def = type_obj.get("type")
                .ok_or_else(|| {
                    serde_json::Error::custom(format!("types[{}]缺少type字段", index))
                })?;
                
            let type_kind = Self::parse_type_kind(type_def)?;
            
            // 解析docs字段
            let docs = type_obj.get("docs")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                });
            
            parsed_types.push(NonAnchorType {
                name,
                type_def: type_kind,
                docs,
            });
            
            log::debug!("✅ 第{}个类型解析完成: {}", index + 1, parsed_types.last().unwrap().name);
        }
        
        log::debug!("✅ types数组手动解析完成，共{}个类型", parsed_types.len());
        Ok(parsed_types)
    }
    
    /// 解析类型种类（Struct/Enum/Alias）
    fn parse_type_kind(type_value: &serde_json::Value) -> Result<NonAnchorTypeKind, serde_json::Error> {
        let type_obj = type_value.as_object().ok_or_else(|| {
            serde_json::Error::custom("type字段必须是对象")
        })?;
        
        let kind = type_obj.get("kind")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                serde_json::Error::custom("type对象缺少kind字段")
            })?;
            
        match kind {
            "struct" => {
                let fields_value = type_obj.get("fields")
                    .ok_or_else(|| {
                        serde_json::Error::custom("struct类型缺少fields字段")
                    })?;
                    
                let fields = Self::parse_fields_manually(fields_value)?;
                Ok(NonAnchorTypeKind::Struct { fields })
            }
            "enum" => {
                let variants_value = type_obj.get("variants")
                    .ok_or_else(|| {
                        serde_json::Error::custom("enum类型缺少variants字段")
                    })?;
                    
                // 手动解析enum variants，处理特殊字段格式
                let variants = Self::parse_enum_variants_manually(variants_value)?;
                Ok(NonAnchorTypeKind::Enum { variants })
            }
            "alias" => {
                let value_field = type_obj.get("value")
                    .ok_or_else(|| {
                        serde_json::Error::custom("alias类型缺少value字段")
                    })?;
                    
                let value = Self::parse_field_type_manually(value_field)?;
                Ok(NonAnchorTypeKind::Alias { value })
            }
            _ => Err(serde_json::Error::custom(format!("未知的类型kind: {}", kind)))
        }
    }
    
    /// 手动解析fields数组
    fn parse_fields_manually(fields_value: &serde_json::Value) -> Result<Vec<NonAnchorField>, serde_json::Error> {
        let fields_array = fields_value.as_array().ok_or_else(|| {
            serde_json::Error::custom("fields必须是数组")
        })?;
        
        let mut parsed_fields = Vec::new();
        
        for (index, field_value) in fields_array.iter().enumerate() {
            let field_obj = field_value.as_object().ok_or_else(|| {
                serde_json::Error::custom(format!("fields[{}]必须是对象", index))
            })?;
            
            let name = field_obj.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unnamed_field")
                .to_string();
                
            let type_value = field_obj.get("type")
                .ok_or_else(|| {
                    serde_json::Error::custom(format!("fields[{}]缺少type字段", index))
                })?;
                
            let field_type = Self::parse_field_type_manually(type_value)?;
            
            let docs = field_obj.get("docs")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                });
            
            parsed_fields.push(NonAnchorField {
                name,
                field_type,
                docs,
            });
        }
        
        Ok(parsed_fields)
    }
    
    /// 手动解析enum variants，处理特殊的fields格式
    fn parse_enum_variants_manually(variants_value: &serde_json::Value) -> Result<Vec<NonAnchorEnumVariant>, serde_json::Error> {
        let variants_array = variants_value.as_array().ok_or_else(|| {
            serde_json::Error::custom("variants必须是数组")
        })?;
        
        let mut parsed_variants = Vec::new();
        
        for (index, variant_value) in variants_array.iter().enumerate() {
            let variant_obj = variant_value.as_object().ok_or_else(|| {
                serde_json::Error::custom(format!("variants[{}]必须是对象", index))
            })?;
            
            let name = variant_obj.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("UnnamedVariant")
                .to_string();
            
            // 解析fields字段 - 支持两种格式：标准格式和Phoenix特殊格式
            let fields = variant_obj.get("fields")
                .map(|fields_value| {
                    log::debug!("🔧 解析enum variant '{}' 的字段，字段数量: {}", 
                        name, fields_value.as_array().map_or(0, |arr| arr.len()));
                    
                    if let Some(fields_array) = fields_value.as_array() {
                        // 检查字段格式类型
                        if let Some(first_field) = fields_array.first() {
                            if let Some(first_obj) = first_field.as_object() {
                                // 检查是否是Phoenix特殊格式：[{"defined": "TypeName"}] (无name字段)
                                if first_obj.contains_key("defined") && !first_obj.contains_key("name") {
                                    log::debug!("🔧 检测到Phoenix特殊格式enum variant fields (无name字段)");
                                    // Phoenix特殊格式处理
                                    let mut parsed_fields = Vec::new();
                                    for (field_index, field_value) in fields_array.iter().enumerate() {
                                        if let Some(field_obj) = field_value.as_object() {
                                            if let Some(defined) = field_obj.get("defined").and_then(|v| v.as_str()) {
                                                parsed_fields.push(NonAnchorField {
                                                    name: format!("field_{}", field_index),
                                                    field_type: NonAnchorFieldType::Defined { 
                                                        defined: defined.to_string() 
                                                    },
                                                    docs: None,
                                                });
                                            }
                                        }
                                    }
                                    Ok(parsed_fields)
                                } else {
                                    log::debug!("🔧 检测到标准格式enum variant fields (有name字段)");
                                    // 标准格式处理
                                    Self::parse_fields_manually(fields_value)
                                }
                            } else {
                                Self::parse_fields_manually(fields_value)
                            }
                        } else {
                            Self::parse_fields_manually(fields_value)
                        }
                    } else {
                        Self::parse_fields_manually(fields_value)
                    }
                })
                .transpose()?;
            
            let docs = variant_obj.get("docs")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                });
            
            parsed_variants.push(NonAnchorEnumVariant {
                name,
                fields,
                docs,
            });
        }
        
        Ok(parsed_variants)
    }
    
    /// 手动解析字段类型 - 处理字符串或复杂类型
    fn parse_field_type_manually(type_value: &serde_json::Value) -> Result<NonAnchorFieldType, serde_json::Error> {
        match type_value {
            serde_json::Value::String(s) => {
                // 简单字符串类型如 "u64", "publicKey"
                Ok(NonAnchorFieldType::Basic(s.clone()))
            }
            serde_json::Value::Object(obj) => {
                // 复杂类型如 {"defined": "OrderPacket"}, {"array": ["u64", 32]}
                if let Some(defined) = obj.get("defined").and_then(|v| v.as_str()) {
                    Ok(NonAnchorFieldType::Defined { 
                        defined: defined.to_string() 
                    })
                } else if let Some(option) = obj.get("option") {
                    let inner = Self::parse_field_type_manually(option)?;
                    Ok(NonAnchorFieldType::Option {
                        option: Box::new(inner)
                    })
                } else if let Some(vec) = obj.get("vec") {
                    let inner = Self::parse_field_type_manually(vec)?;
                    Ok(NonAnchorFieldType::Vec {
                        vec: Box::new(inner)
                    })
                } else if let Some(array) = obj.get("array") {
                    let array_value = array.as_array().ok_or_else(|| {
                        serde_json::Error::custom("array类型必须是数组")
                    })?;
                    
                    if array_value.len() != 2 {
                        return Err(serde_json::Error::custom("array类型必须有2个元素[type, size]"));
                    }
                    
                    let inner_type = Self::parse_field_type_manually(&array_value[0])?;
                    let size = array_value[1].as_u64().ok_or_else(|| {
                        serde_json::Error::custom("array size必须是数字")
                    })? as usize;
                    
                    Ok(NonAnchorFieldType::Array { 
                        array: (Box::new(inner_type), size) 
                    })
                } else {
                    Err(serde_json::Error::custom("无法识别的复杂类型"))
                }
            }
            _ => Err(serde_json::Error::custom("type字段必须是字符串或对象"))
        }
    }
}