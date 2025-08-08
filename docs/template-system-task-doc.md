# Solores 模板系统开发任务文档

## 项目背景

为 Solores Solana IDL 代码生成器创建模板系统，支持 Anchor 和非 Anchor 两种合约模式，专门针对 instructions、accounts、events、parser tests 四个组件进行模板化改造。

## 代表性 JSON 文件选择

### 1. Anchor 格式代表: `raydium_launchpad.json`

**选择理由**: 
- ✅ 完整的 Anchor IDL 结构
- ✅ 包含 8字节 discriminator
- ✅ 有完整的 instructions/accounts/types/events/errors
- ✅ 复杂度适中，适合模板开发

**特征分析**:
```json
{
  "address": "LancP9TtWTzQDgv6D3nMJgW6zb2nF5MqEQ3QpQRN1gg",
  "instructions": [
    {
      "discriminator": [250, 234, 13, 123, 213, 156, 19, 236], // 8字节
      "name": "buy_exact_in",
      "accounts": [...],
      "args": [...]
    }
  ],
  "accounts": [...],
  "types": [...],
  "events": [...],
  "errors": [...]
}
```

### 2. 非 Anchor 格式代表: `phoenix.json`

**选择理由**:
- ✅ 典型的非 Anchor 格式
- ✅ 有自定义 discriminator 值 (不是连续的)
- ✅ 复杂的指令结构
- ✅ 当前测试中发现的问题案例，适合验证

**特征分析**:
```json
{
  "instructions": [
    {
      "name": "swap",
      "discriminator": [0], // 1字节
    },
    {
      "name": "initialize_market", 
      "discriminator": [100], // 跳跃值
    }
  ]
}
```

### 3. 简单格式代表: `moonshot.json`

**选择理由**:
- ✅ 结构简单，适合测试基础功能
- ✅ 组件较少，便于调试
- ✅ 代表最小化 IDL 格式

---

## 任务1: 创建模板系统基础架构

### 目标
在 `solores/src/templates/` 目录下建立模板系统的基础架构。

### 文件结构要求
```
solores/src/templates/
├── mod.rs                          # 模板系统总入口
├── common/
│   ├── mod.rs                      # 通用组件入口
│   ├── doc_generator.rs            # 文档注释生成器
│   ├── import_manager.rs           # 导入管理器  
│   ├── attribute_generator.rs      # 属性生成器
│   └── test_utils.rs              # 通用测试工具
├── anchor/
│   ├── mod.rs                      # Anchor 模式入口
│   ├── instructions_template.rs    # Instructions 模板
│   ├── accounts_template.rs        # Accounts 模板
│   ├── events_template.rs          # Events 模板
│   ├── types_template.rs           # Types 模板
│   ├── parsers_template.rs         # Parsers 模板
│   ├── instructions_parser_test_template.rs  # Instructions Parser 测试模板
│   └── accounts_parser_test_template.rs      # Accounts Parser 测试模板
├── non_anchor/
│   ├── mod.rs                      # 非 Anchor 模式入口
│   ├── instructions_template.rs    # Instructions 模板
│   ├── accounts_template.rs        # Accounts 模板
│   ├── events_template.rs          # Events 模板
│   ├── types_template.rs           # Types 模板
│   ├── parsers_template.rs         # Parsers 模板
│   ├── instructions_parser_test_template.rs  # Instructions Parser 测试模板
│   └── accounts_parser_test_template.rs      # Accounts Parser 测试模板
└── factory.rs                      # 模板工厂
```

### 核心 Trait 设计要求
```rust
// templates/mod.rs
pub trait ContractModeTemplate {
    type DiscriminatorType;
    fn discriminator_size() -> usize;
    fn parse_discriminator_code() -> TokenStream;
    fn generate_constants(&self) -> TokenStream;
    fn generate_tests(&self) -> TokenStream;
}

pub enum ContractMode {
    Anchor,
    NonAnchor,
}
```

---

## 任务2: 实现 Anchor 模式模板

### 参考文件
`idls/raydium_launchpad.json`

### 核心要求

#### Discriminator 特征
- 类型: `[u8; 8]`
- 常量: `pub const INIT_IX_DISCM: [u8; 8] = [250, 234, 13, 123, 213, 156, 19, 236];`
- 解析: `let discriminator: [u8; 8] = data[0..8].try_into()?;`

#### Instructions 模板 (`anchor/instructions_template.rs`)
```rust
pub struct AnchorInstructionsTemplate<'a> {
    pub idl: &'a AnchorIdl,
    pub args: &'a Args,
}

impl<'a> AnchorInstructionsTemplate<'a> {
    // 必需方法:
    pub fn generate_discriminator_constants(&self) -> TokenStream;
    pub fn generate_instruction_enum(&self) -> TokenStream;
    
    pub fn generate_parse_function(&self) -> TokenStream {
        // 关键：与Accounts处理保持一致
        quote! {
            pub fn parse_instruction(data: &[u8], accounts: &[Pubkey]) -> Result<ProgramInstruction, std::io::Error> {
                let discriminator: [u8; 8] = data[0..8].try_into()?;
                let mut ix_data = &data[0..];  // 从完整数据开始，不跳过discriminator
                
                match discriminator {
                    CONST_NAME => {
                        let args = ArgsStruct::deserialize(&mut ix_data)?;  // Args包含discriminator
                        Ok(ProgramInstruction::Variant(ix_accounts, args))
                    }
                }
            }
        }
    }
    
    pub fn generate_match_arms(&self) -> TokenStream;
}
```

#### Accounts 模板 (`anchor/accounts_template.rs`)
```rust
pub struct AnchorAccountsTemplate<'a> {
    pub idl: &'a AnchorIdl,
    pub args: &'a Args,
}

impl<'a> AnchorAccountsTemplate<'a> {
    // 基于 discriminator 的账户识别
    pub fn generate_account_enum(&self) -> TokenStream;
    pub fn generate_unpack_function(&self) -> TokenStream;
}
```

#### Events 模板 (`anchor/events_template.rs`)
```rust
pub struct AnchorEventsTemplate<'a> {
    pub events: &'a [Event],
    pub named_types: &'a [NamedType],
}

impl<'a> AnchorEventsTemplate<'a> {
    pub fn generate_event_structs(&self) -> TokenStream;
    pub fn generate_event_wrappers(&self) -> TokenStream;
}
```

#### Types 模板 (`anchor/types_template.rs`)
```rust
pub struct AnchorTypesTemplate<'a> {
    pub idl: &'a AnchorIdl,
    pub args: &'a Args,
}

impl<'a> AnchorTypesTemplate<'a> {
    pub fn generate_type_structs(&self) -> TokenStream {
        // Anchor模式特点：
        // - 结构体包含 discriminator: [u8; 8] 字段
        // - Default实现使用SHA256计算的discriminator值
        // - 完整的Borsh序列化支持
        quote! {
            #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
            pub struct TypeName {
                pub discriminator: [u8; 8],  // Anchor特有：8字节discriminator
                pub field1: u64,
                pub field2: Pubkey,
            }
            
            impl Default for TypeName {
                fn default() -> Self {
                    Self {
                        discriminator: TYPE_NAME_DISCM,  // SHA256计算值
                        field1: 0,
                        field2: Pubkey::default(),
                    }
                }
            }
        }
    }
    
    pub fn generate_type_constants(&self) -> TokenStream {
        // 生成8字节discriminator常量
        quote! {
            pub const TYPE_NAME_DISCM: [u8; 8] = [234, 156, 78, 123, 245, 67, 89, 12];
        }
    }
}
```

#### Parsers 模板 (`anchor/parsers_template.rs`)
```rust
pub struct AnchorParsersTemplate<'a> {
    pub idl: &'a AnchorIdl,
    pub args: &'a Args,
}

impl<'a> AnchorParsersTemplate<'a> {
    pub fn generate_instructions_parser(&self) -> TokenStream {
        // Anchor模式Instructions解析器特点：
        // - 8字节discriminator解析
        // - 从完整数据deserialize (Args包含discriminator字段)
        // - SHA256基础的discriminator常量
        quote! {
            pub fn parse_instruction(data: &[u8], accounts: &[Pubkey]) -> Result<ProgramInstruction, std::io::Error> {
                if data.len() < 8 {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Too short for discriminator"));
                }
                
                let discriminator: [u8; 8] = data[0..8].try_into()?;
                let mut ix_data = &data[0..];  // 从完整数据开始
                
                match discriminator {
                    INIT_IX_DISCM => {
                        check_min_accounts_req(accounts, INIT_IX_ACCOUNTS_LEN, "Initialize")?;
                        let ix_accounts = InitializeKeys::from(&accounts[..INIT_IX_ACCOUNTS_LEN]);
                        let args = InitializeIxArgs::deserialize(&mut ix_data)?;
                        Ok(ProgramInstruction::Initialize(ix_accounts, args))
                    }
                    _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Unknown discriminator"))
                }
            }
        }
    }
    
    pub fn generate_accounts_parser(&self) -> TokenStream {
        // Anchor模式Accounts解析器特点：
        // - 基于8字节discriminator识别
        // - SHA256计算的discriminator验证
        quote! {
            pub fn try_unpack_account(data: &[u8]) -> Result<ProgramAccount, std::io::Error> {
                if data.len() < 8 {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Too short"));
                }
                
                let discriminator: [u8; 8] = data[0..8].try_into()?;
                
                match discriminator {
                    ACCOUNT_DISCM => Ok(ProgramAccount::Account(Account::from_bytes(data)?)),
                    _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Unknown account"))
                }
            }
        }
    }
}
```

#### Instructions Parser 测试模板 (`anchor/instructions_parser_test_template.rs`)
```rust
pub struct AnchorInstructionsParserTestTemplate;

impl AnchorInstructionsParserTestTemplate {
    // 专门针对 parsers/instructions.rs 的测试
    pub fn generate_instructions_consistency_tests(&self, instructions: &[NamedInstruction]) -> TokenStream {
        let test_functions = instructions.iter().map(|ix| {
            let test_name = syn::Ident::new(&format!("test_{}_consistency", ix.name.to_case(Case::Snake)), proc_macro2::Span::call_site());
            let variant_name = syn::Ident::new(&ix.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let keys_struct_name = syn::Ident::new(&format!("{}Keys", ix.name.to_case(Case::Pascal)), proc_macro2::Span::call_site());
            let accounts_len_const = syn::Ident::new(&format!("{}_IX_ACCOUNTS_LEN", ix.name.to_shouty_snake_case()), proc_macro2::Span::call_site());
            
            let has_args = ix.args.as_ref().map_or(false, |args| !args.is_empty());
            if has_args {
                let args_struct_name = syn::Ident::new(&format!("{}IxArgs", ix.name.to_case(Case::Pascal)), proc_macro2::Span::call_site());
                
                quote! {
                    #[test]
                    fn #test_name() {
                        // Create mock accounts
                        let mock_accounts = vec![Pubkey::default(); #accounts_len_const];
                        
                        // Create test arguments - Args包含discriminator字段
                        let test_args = #args_struct_name::default();
                        let test_data = test_args.try_to_vec().unwrap();  // 完整序列化包含discriminator
                        
                        // Parse the instruction - 从完整数据开始解析
                        match parse_instruction(&test_data, &mock_accounts) {
                            Ok(ProgramInstruction::#variant_name(keys, args)) => {
                                assert_eq!(keys, #keys_struct_name::from(&mock_accounts[..#accounts_len_const]));
                                assert_eq!(args, test_args);
                            }
                            Ok(_) => panic!("Discriminator matched wrong instruction type"),
                            Err(e) => {
                                let error_msg = e.to_string();
                                assert!(!error_msg.contains("Unknown instruction discriminator"), 
                                       "Should recognize discriminator for {}", stringify!(#variant_name));
                            }
                        }
                    }
                }
            } else {
                // 无参数指令测试
                quote! {
                    #[test] 
                    fn #test_name() {
                        let mock_accounts = vec![Pubkey::default(); #accounts_len_const];
                        
                        // 无参数指令只有discriminator - 使用默认Args结构体
                        let test_args = #args_struct_name::default();  // 包含discriminator
                        let test_data = test_args.try_to_vec().unwrap();
                        
                        let parsed = parse_instruction(&test_data, &mock_accounts).expect("Failed to parse instruction");
                        match parsed {
                            ProgramInstruction::#variant_name(keys) => {
                                assert_eq!(keys, #keys_struct_name::from(&mock_accounts[..#accounts_len_const]));
                            }
                            _ => panic!("Parsed instruction has wrong type"),
                        }
                    }
                }
            }
        });

        quote! {
            #[cfg(test)]
            mod instructions_consistency_tests {
                use super::*;
                #[allow(unused_imports)]  // 消除导入警告
                use borsh::{BorshDeserialize, BorshSerialize};
                #[allow(unused_imports)]
                use crate::types::*;
                #[allow(unused_imports)]
                use crate::*;
                
                #(#test_functions)*
            }
        }
    }
}
```

#### Accounts Parser 测试模板 (`anchor/accounts_parser_test_template.rs`)
```rust
pub struct AnchorAccountsParserTestTemplate;

impl AnchorAccountsParserTestTemplate {
    // 专门针对 parsers/accounts.rs 的测试
    pub fn generate_accounts_consistency_tests(&self, accounts: &[NamedAccount]) -> TokenStream {
        let test_functions = accounts.iter().map(|acc| {
            let test_name = syn::Ident::new(&format!("test_{}_consistency", acc.name.to_case(Case::Snake)), proc_macro2::Span::call_site());
            let variant_name = syn::Ident::new(&acc.name, proc_macro2::Span::call_site());
            let struct_name = syn::Ident::new(&acc.name, proc_macro2::Span::call_site());
            
            quote! {
                #[test]
                fn #test_name() {
                    // Create test account data - Account包含discriminator字段
                    let test_account = #struct_name::default();
                    let test_data = test_account.try_to_vec().unwrap();  // 完整序列化包含discriminator
                    
                    // Parse the account - 从完整数据开始解析
                    match try_unpack_account(&test_data) {
                        Ok(ProgramAccount::#variant_name(account)) => {
                            // 验证account数据与原始数据一致
                            assert_eq!(account, test_account);
                        }
                        Ok(_) => panic!("Discriminator matched wrong account type"),
                        Err(e) => {
                            let error_msg = e.to_string();
                            assert!(!error_msg.contains("Unknown account discriminator"), 
                                   "Should recognize discriminator for {}", stringify!(#variant_name));
                        }
                    }
                }
            }
        });

        quote! {
            #[cfg(test)]
            mod accounts_consistency_tests {
                use super::*;
                #[allow(unused_imports)]  // 消除导入警告
                use borsh::{BorshDeserialize, BorshSerialize};
                #[allow(unused_imports)]
                use crate::types::*;
                #[allow(unused_imports)]
                use crate::*;
                
                #(#test_functions)*
            }
        }
    }
}
```

---

## 任务3: 实现非 Anchor 模式模板

### 参考文件
`idls/phoenix.json`

### 核心要求

#### Discriminator 特征  
- 类型: `u8`
- 常量: `pub const INIT_IX_DISCM: u8 = 100;` (使用 IDL 中的值，不是索引)
- 解析: `let discriminator = data[0];`

#### Instructions 模板 (`non_anchor/instructions_template.rs`)
```rust
pub struct NonAnchorInstructionsTemplate<'a> {
    pub idl: &'a AnchorIdl,
    pub args: &'a Args,
}

impl<'a> NonAnchorInstructionsTemplate<'a> {
    // 必需方法:
    pub fn generate_discriminator_constants(&self) -> TokenStream;
    
    pub fn generate_parse_function(&self) -> TokenStream {
        // 关键：与Anchor Instructions保持一致
        quote! {
            pub fn parse_instruction(data: &[u8], accounts: &[Pubkey]) -> Result<ProgramInstruction, std::io::Error> {
                let discriminator = data[0];  // 1字节discriminator
                let mut ix_data = &data[0..];  // 从完整数据开始，不跳过discriminator
                
                match discriminator {
                    CONST_NAME => {
                        let args = ArgsStruct::deserialize(&mut ix_data)?;  // Args包含discriminator
                        Ok(ProgramInstruction::Variant(ix_accounts, args))
                    }
                }
            }
        }
    }
    
    // 关键逻辑: 如果 IDL 有 discriminator，使用该值；否则使用数组索引
    pub fn get_discriminator_value(&self, ix: &NamedInstruction, index: usize) -> u8;
}
```

#### Accounts 模板 (`non_anchor/accounts_template.rs`)
```rust  
pub struct NonAnchorAccountsTemplate<'a> {
    pub idl: &'a AnchorIdl,
    pub args: &'a Args,
}

impl<'a> NonAnchorAccountsTemplate<'a> {
    // 基于长度的账户识别
    pub fn generate_length_based_unpack(&self) -> TokenStream;
    // 生成: const ACCOUNT_LEN: usize = std::mem::size_of::<Account>();
}
```

#### Types 模板 (`non_anchor/types_template.rs`)
```rust
pub struct NonAnchorTypesTemplate<'a> {
    pub idl: &'a AnchorIdl,
    pub args: &'a Args,
}

impl<'a> NonAnchorTypesTemplate<'a> {
    pub fn generate_type_structs(&self) -> TokenStream {
        // 非Anchor模式特点：
        // - 可选的 discriminator: u8 字段 (基于是否有discriminator)
        // - Default实现使用简单索引值或0
        // - 标准的Borsh序列化支持
        quote! {
            #[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq)]
            pub struct TypeName {
                pub discriminator: u8,  // 非Anchor特有：1字节discriminator或无
                pub field1: u64,
                pub field2: Pubkey,
            }
            
            impl Default for TypeName {
                fn default() -> Self {
                    Self {
                        discriminator: TYPE_NAME_DISCM,  // 简单索引值
                        field1: 0,
                        field2: Pubkey::default(),
                    }
                }
            }
        }
    }
    
    pub fn generate_type_constants(&self) -> TokenStream {
        // 生成1字节discriminator常量 (IDL值或索引)
        quote! {
            pub const TYPE_NAME_DISCM: u8 = 2;  // 来自IDL或数组索引
        }
    }
}
```

#### Parsers 模板 (`non_anchor/parsers_template.rs`)
```rust
pub struct NonAnchorParsersTemplate<'a> {
    pub idl: &'a AnchorIdl,
    pub args: &'a Args,
}

impl<'a> NonAnchorParsersTemplate<'a> {
    pub fn generate_instructions_parser(&self) -> TokenStream {
        // 非Anchor模式Instructions解析器特点：
        // - 1字节discriminator解析
        // - 从完整数据deserialize (Args包含discriminator字段)
        // - 索引基础的discriminator常量
        quote! {
            pub fn parse_instruction(data: &[u8], accounts: &[Pubkey]) -> Result<ProgramInstruction, std::io::Error> {
                if data.is_empty() {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Empty data"));
                }
                
                let discriminator = data[0];  // 1字节discriminator
                let mut ix_data = &data[0..];  // 从完整数据开始
                
                match discriminator {
                    INIT_IX_DISCM => {
                        check_min_accounts_req(accounts, INIT_IX_ACCOUNTS_LEN, "Initialize")?;
                        let ix_accounts = InitializeKeys::from(&accounts[..INIT_IX_ACCOUNTS_LEN]);
                        let args = InitializeIxArgs::deserialize(&mut ix_data)?;
                        Ok(ProgramInstruction::Initialize(ix_accounts, args))
                    }
                    _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Unknown discriminator"))
                }
            }
        }
    }
    
    pub fn generate_accounts_parser(&self) -> TokenStream {
        // 非Anchor模式Accounts解析器特点：
        // - 基于数据长度识别
        // - std::mem::size_of计算的长度匹配
        quote! {
            pub fn try_unpack_account(data: &[u8]) -> Result<ProgramAccount, std::io::Error> {
                let data_len = data.len();
                const ACCOUNT_LEN: usize = std::mem::size_of::<Account>();
                
                match data_len {
                    ACCOUNT_LEN => Ok(ProgramAccount::Account(Account::from_bytes(data)?)),
                    _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid length"))
                }
            }
        }
    }
}
```

#### Instructions Parser 测试模板 (`non_anchor/instructions_parser_test_template.rs`)
```rust
pub struct NonAnchorInstructionsParserTestTemplate;

impl NonAnchorInstructionsParserTestTemplate {
    // 专门针对 parsers/instructions.rs 的测试
    pub fn generate_instructions_consistency_tests(&self, instructions: &[NamedInstruction]) -> TokenStream {
        let test_functions = instructions.iter().map(|ix| {
            let test_name = syn::Ident::new(&format!("test_{}_consistency", ix.name.to_case(Case::Snake)), proc_macro2::Span::call_site());
            let variant_name = syn::Ident::new(&ix.name.to_case(Case::Pascal), proc_macro2::Span::call_site());
            let keys_struct_name = syn::Ident::new(&format!("{}Keys", ix.name.to_case(Case::Pascal)), proc_macro2::Span::call_site());
            let accounts_len_const = syn::Ident::new(&format!("{}_IX_ACCOUNTS_LEN", ix.name.to_shouty_snake_case()), proc_macro2::Span::call_site());
            
            let has_args = ix.args.as_ref().map_or(false, |args| !args.is_empty());
            if has_args {
                let args_struct_name = syn::Ident::new(&format!("{}IxArgs", ix.name.to_case(Case::Pascal)), proc_macro2::Span::call_site());
                
                quote! {
                    #[test]
                    fn #test_name() {
                        // Create mock accounts
                        let mock_accounts = vec![Pubkey::default(); #accounts_len_const];
                        
                        // 非Anchor模式: Args同样包含discriminator字段(1字节)
                        let test_args = #args_struct_name::default();
                        let test_data = test_args.try_to_vec().unwrap();  // 完整序列化包含1字节discriminator
                        
                        // Parse the instruction - 从完整数据开始解析
                        match parse_instruction(&test_data, &mock_accounts) {
                            Ok(ProgramInstruction::#variant_name(keys, args)) => {
                                assert_eq!(keys, #keys_struct_name::from(&mock_accounts[..#accounts_len_const]));
                                assert_eq!(args, test_args);
                            }
                            Ok(_) => panic!("Discriminator matched wrong instruction type"),
                            Err(e) => {
                                let error_msg = e.to_string();
                                assert!(!error_msg.contains("Unknown instruction discriminator"), 
                                       "Should recognize discriminator for {}", stringify!(#variant_name));
                            }
                        }
                    }
                }
            } else {
                // 无参数指令测试
                quote! {
                    #[test] 
                    fn #test_name() {
                        let mock_accounts = vec![Pubkey::default(); #accounts_len_const];
                        
                        // 无参数指令只有1字节discriminator - 使用默认Args结构体
                        let test_args = #args_struct_name::default();  // 包含1字节discriminator
                        let test_data = test_args.try_to_vec().unwrap();
                        
                        let parsed = parse_instruction(&test_data, &mock_accounts).expect("Failed to parse instruction");
                        match parsed {
                            ProgramInstruction::#variant_name(keys) => {
                                assert_eq!(keys, #keys_struct_name::from(&mock_accounts[..#accounts_len_const]));
                            }
                            _ => panic!("Parsed instruction has wrong type"),
                        }
                    }
                }
            }
        });

        quote! {
            #[cfg(test)]
            mod instructions_consistency_tests {
                use super::*;
                #[allow(unused_imports)]  // 消除导入警告
                use borsh::{BorshDeserialize, BorshSerialize};
                #[allow(unused_imports)]
                use crate::types::*;
                #[allow(unused_imports)]
                use crate::*;
                
                #(#test_functions)*
            }
        }
    }
}
```

#### Accounts Parser 测试模板 (`non_anchor/accounts_parser_test_template.rs`)  
```rust
pub struct NonAnchorAccountsParserTestTemplate;

impl NonAnchorAccountsParserTestTemplate {
    // 专门针对 parsers/accounts.rs 的测试
    pub fn generate_accounts_consistency_tests(&self, accounts: &[NamedAccount]) -> TokenStream {
        let test_functions = accounts.iter().map(|acc| {
            let test_name = syn::Ident::new(&format!("test_{}_consistency", acc.name.to_case(Case::Snake)), proc_macro2::Span::call_site());
            let variant_name = syn::Ident::new(&acc.name, proc_macro2::Span::call_site());
            let struct_name = syn::Ident::new(&acc.name, proc_macro2::Span::call_site());
            
            quote! {
                #[test]
                fn #test_name() {
                    // Create test account data - 非Anchor基于长度识别
                    let test_account = #struct_name::default();
                    let test_data = test_account.try_to_vec().unwrap();
                    
                    // Parse the account - 基于数据长度匹配
                    match try_unpack_account(&test_data) {
                        Ok(ProgramAccount::#variant_name(account)) => {
                            // 验证account数据与原始数据一致
                            assert_eq!(account, test_account);
                            
                            // 验证数据长度匹配期望值
                            assert_eq!(test_data.len(), #struct_name::LEN, "Data length should match struct size");
                        }
                        Ok(_) => panic!("Length matched wrong account type"),
                        Err(e) => {
                            let error_msg = e.to_string();
                            assert!(!error_msg.contains("Invalid Account data length"), 
                                   "Should recognize account length for {}", stringify!(#variant_name));
                        }
                    }
                }
            }
        });

        quote! {
            #[cfg(test)]
            mod accounts_consistency_tests {
                use super::*;
                #[allow(unused_imports)]  // 消除导入警告
                use borsh::{BorshDeserialize, BorshSerialize};
                #[allow(unused_imports)]
                use crate::types::*;
                #[allow(unused_imports)]
                use crate::*;
                
                #(#test_functions)*
            }
        }
    }
}
```

---

## 任务4: 创建模板工厂和集成点

### 模板工厂 (`factory.rs`)
```rust
pub struct TemplateFactory;

impl TemplateFactory {
    pub fn create_instructions_template(mode: ContractMode, idl: &AnchorIdl, args: &Args) -> Box<dyn TemplateGenerator>;
    pub fn create_accounts_template(mode: ContractMode, idl: &AnchorIdl, args: &Args) -> Box<dyn TemplateGenerator>;
    pub fn create_events_template(mode: ContractMode, events: &[Event], types: &[NamedType]) -> Box<dyn TemplateGenerator>;
    
    // 新增的核心模板
    pub fn create_types_template(mode: ContractMode, idl: &AnchorIdl, args: &Args) -> Box<dyn TypesTemplateGenerator>;
    pub fn create_parsers_template(mode: ContractMode, idl: &AnchorIdl, args: &Args) -> Box<dyn ParsersTemplateGenerator>;
    
    // Parser测试模板分离为两个独立方法
    pub fn create_instructions_parser_test_template(mode: ContractMode) -> Box<dyn InstructionsParserTestGenerator>;
    pub fn create_accounts_parser_test_template(mode: ContractMode) -> Box<dyn AccountsParserTestGenerator>;
}

// 核心模板生成器trait
pub trait TypesTemplateGenerator {
    fn generate_type_structs(&self) -> TokenStream;
    fn generate_type_constants(&self) -> TokenStream;
}

pub trait ParsersTemplateGenerator {
    fn generate_instructions_parser(&self) -> TokenStream;
    fn generate_accounts_parser(&self) -> TokenStream;
}

// Parser测试生成器trait
pub trait InstructionsParserTestGenerator {
    fn generate_instructions_consistency_tests(&self, instructions: &[NamedInstruction]) -> TokenStream;
}

pub trait AccountsParserTestGenerator {
    fn generate_accounts_consistency_tests(&self, accounts: &[NamedAccount]) -> TokenStream;
}
```

### 集成到现有代码
在以下文件中集成模板系统:
- `solores/src/idl_format/anchor/parsers/instructions.rs`
- `solores/src/idl_format/anchor/parsers/accounts.rs`
- `solores/src/idl_format/anchor/events/mod.rs`

---

## 任务5: 验证和测试

### 验证文件
使用选定的代表性文件进行验证:
1. `raydium_launchpad.json` - Anchor 模式验证
2. `phoenix.json` - 非 Anchor 模式验证  
3. `moonshot.json` - 简单格式验证

### 测试要求
1. **功能测试**: 生成的代码与当前版本功能一致
2. **编译测试**: 所有生成的代码都能编译通过
3. **警告消除**: 解决当前的 7 个编译警告问题
4. **回归测试**: 确保 16 个库仍然 100% 编译成功

### 验证脚本
```bash
# 1. 生成测试
RUST_LOG=info ./target/release/solores idls/raydium_launchpad.json -o test_template_anchor --generate-parser
RUST_LOG=info ./target/release/solores idls/phoenix.json -o test_template_non_anchor --generate-parser
RUST_LOG=info ./target/release/solores idls/moonshot.json -o test_template_simple --generate-parser

# 2. 编译测试
cd test_template_anchor && cargo test
cd test_template_non_anchor && cargo test  
cd test_template_simple && cargo test

# 3. 运行完整回归测试
./final_batch_test.sh
```

---

## 任务6: 文档和整理

### 文档要求
1. **README.md**: 模板系统使用说明
2. **API 文档**: 每个模板的使用方法
3. **示例代码**: 展示如何使用模板系统
4. **迁移指南**: 从旧系统到新系统的迁移步骤

### 代码质量要求
1. **错误处理**: 完善的错误处理机制
2. **性能优化**: 避免不必要的重复计算
3. **内存管理**: 合理使用 TokenStream 缓存
4. **代码风格**: 遵循项目现有的代码风格

---

## 关键技术点

### Discriminator 处理差异

#### Instructions Parser 处理方式
```rust
// Anchor Instructions (8字节) - 正确方式
pub fn parse_instruction(data: &[u8], accounts: &[Pubkey]) -> Result<ProgramInstruction, std::io::Error> {
    let discriminator: [u8; 8] = data[0..8].try_into()?;
    let mut ix_data = &data[0..];  // ✅ 从完整数据开始，Args包含discriminator字段
    let args = BuyExactInIxArgs::deserialize(&mut ix_data)?;  // 自动读取discriminator
    Ok(ProgramInstruction::BuyExactIn(ix_accounts, args))
}

// 非 Anchor Instructions (1字节) - 正确方式  
pub fn parse_instruction(data: &[u8], accounts: &[Pubkey]) -> Result<ProgramInstruction, std::io::Error> {
    let discriminator = data[0];
    let mut ix_data = &data[0..];  // ✅ 从完整数据开始，Args包含discriminator字段
    let args = InitializeIxArgs::deserialize(&mut ix_data)?;  // 自动读取discriminator
    Ok(ProgramInstruction::Initialize(ix_accounts, args))
}
```

#### Accounts Parser 处理方式 (参考标准)
```rust
// Anchor Accounts (8字节) - 标准正确实现
pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
    if &data[0..8] != POOL_STATE_ACCOUNT_DISCM {  // 验证discriminator
        return Err(std::io::Error::new(...));
    }
    borsh::from_slice(data)  // ✅ 从完整数据deserialize，包含discriminator字段
}

// 非 Anchor Accounts - 基于长度识别
pub fn try_unpack_account(data: &[u8]) -> Result<ProgramAccount, std::io::Error> {
    match data.len() {
        ACCOUNT_LEN => Ok(ProgramAccount::Account(Account::from_bytes(data)?)),
        _ => Err(std::io::Error::new(...))
    }
}
```

#### 关键技术要点
- **结构体设计**: Instructions的`BuyExactInIxArgs`和Accounts的`PoolState`都包含discriminator字段
- **处理一致性**: 两者都应该从完整数据(`&data[0..]`)开始deserialize
- **验证逻辑**: Accounts先验证discriminator匹配，Instructions通过match discriminator路由

### 测试数据生成差异
```rust
// Anchor
test_data.extend_from_slice(&DISCRIMINATOR_CONST); // [u8; 8]

// 非 Anchor
test_data.push(DISCRIMINATOR_CONST); // u8
```

### 常量定义差异
```rust
// Anchor
pub const INIT_IX_DISCM: [u8; 8] = [250, 234, 13, 123, 213, 156, 19, 236];

// 非 Anchor  
pub const INIT_IX_DISCM: u8 = 100; // 来自 IDL discriminator[0] 或索引
```

---

## 成功标准

### 阶段1: 基础架构 ✅
- [ ] 文件结构创建完成
- [ ] 基础 trait 和 enum 定义完成
- [ ] 模板工厂框架完成

### 阶段2-3: 模板实现 ✅  
- [ ] Anchor 模式 4 个模板完成
- [ ] 非 Anchor 模式 4 个模板完成
- [ ] 通过代表性文件验证

### 阶段4: 集成 ✅
- [ ] 现有生成器集成模板系统
- [ ] 功能等价性验证通过
- [ ] 编译警告问题解决

### 阶段5: 验证 ✅
- [ ] 16 个库 100% 编译成功
- [ ] 所有测试通过 (当前 9/16 → 16/16)
- [ ] 性能无明显下降

### 最终交付
- [ ] 完整的模板系统
- [ ] 解决编译警告问题
- [ ] 16 个库 100% 测试成功
- [ ] 完整的文档和示例

---

## 注意事项

### 当前问题分析
目前有 7 个库因为编译警告失败:
- `BorshDeserialize` 和 `BorshSerialize` 导入但未使用
- `crate::types::*` 和 `crate::*` 导入但未使用

### 解决方案
使用 `#[allow(unused_imports)]` 属性来消除这些警告，同时保持代码生成的一致性。

### 技术要求
- 所有模板必须生成语法正确的 Rust 代码
- 生成的代码必须与当前版本功能等价
- 模板系统必须易于维护和扩展
- 必须提供完整的错误处理和用户友好的错误信息

这个任务文档提供了完整的开发指导，agents 可以按照这个文档进行系统性的开发工作。