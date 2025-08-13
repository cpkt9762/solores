//! 命名风格转换器
//!
//! 提供各种命名风格之间的转换，确保生成的代码符合Rust命名规范

use convert_case::{Case, Casing};
use std::collections::HashMap;

/// 命名风格转换器
pub struct NamingConverter {
    /// 缓存已转换的名称，避免重复计算
    cache: HashMap<String, String>,
    /// 特殊命名映射（用于处理特殊情况）
    special_mappings: HashMap<String, String>,
}

impl NamingConverter {
    /// 创建新的命名风格转换器
    pub fn new() -> Self {
        let mut special_mappings = HashMap::new();
        
        // 添加常见的特殊映射
        special_mappings.insert("publicKey".to_string(), "pubkey".to_string());
        special_mappings.insert("pubKey".to_string(), "pubkey".to_string());
        special_mappings.insert("PublicKey".to_string(), "Pubkey".to_string());
        special_mappings.insert("PubKey".to_string(), "Pubkey".to_string());
        
        // 常见的缩写词保持大写
        special_mappings.insert("id".to_string(), "id".to_string());
        special_mappings.insert("ID".to_string(), "id".to_string());
        special_mappings.insert("url".to_string(), "url".to_string());
        special_mappings.insert("URL".to_string(), "url".to_string());
        special_mappings.insert("api".to_string(), "api".to_string());
        special_mappings.insert("API".to_string(), "api".to_string());
        
        Self {
            cache: HashMap::new(),
            special_mappings,
        }
    }

    /// 添加特殊映射
    pub fn add_special_mapping(&mut self, from: &str, to: &str) {
        self.special_mappings.insert(from.to_string(), to.to_string());
    }

    /// 转换为snake_case（用于字段名、变量名）
    pub fn to_snake_case(&mut self, name: &str) -> String {
        if let Some(cached) = self.cache.get(name) {
            return cached.clone();
        }

        let result = if let Some(special) = self.special_mappings.get(name) {
            special.clone()
        } else {
            // 先处理特殊情况：连续的大写字母
            let processed = self.preprocess_for_snake_case(name);
            processed.to_case(Case::Snake)
        };

        self.cache.insert(name.to_string(), result.clone());
        result
    }

    /// 转换为PascalCase（用于结构体名、枚举名）
    pub fn to_pascal_case(&mut self, name: &str) -> String {
        let cache_key = format!("pascal_{}", name);
        if let Some(cached) = self.cache.get(&cache_key) {
            return cached.clone();
        }

        let result = if let Some(special) = self.special_mappings.get(name) {
            // 对特殊映射也进行PascalCase转换
            special.to_case(Case::Pascal)
        } else {
            name.to_case(Case::Pascal)
        };

        self.cache.insert(cache_key, result.clone());
        result
    }

    /// 转换为camelCase（用于方法名，较少使用）
    pub fn to_camel_case(&mut self, name: &str) -> String {
        let cache_key = format!("camel_{}", name);
        if let Some(cached) = self.cache.get(&cache_key) {
            return cached.clone();
        }

        let result = if let Some(special) = self.special_mappings.get(name) {
            special.to_case(Case::Camel)
        } else {
            name.to_case(Case::Camel)
        };

        self.cache.insert(cache_key, result.clone());
        result
    }

    /// 转换为SCREAMING_SNAKE_CASE（用于常量）
    pub fn to_screaming_snake_case(&mut self, name: &str) -> String {
        let cache_key = format!("screaming_{}", name);
        if let Some(cached) = self.cache.get(&cache_key) {
            return cached.clone();
        }

        let result = if let Some(special) = self.special_mappings.get(name) {
            special.to_case(Case::ScreamingSnake)
        } else {
            name.to_case(Case::ScreamingSnake)
        };

        self.cache.insert(cache_key, result.clone());
        result
    }

    /// 智能转换字段名（自动处理各种输入格式）
    pub fn convert_field_name(&mut self, name: &str) -> String {
        // 先应用智能camelCase到snake_case转换
        let smart_converted = self.smart_camel_to_snake(name);
        // 然后移除可能的前缀或后缀并清理
        let cleaned = self.clean_field_name(&smart_converted);
        let snake_cased = self.to_snake_case(&cleaned);
        // 最后检查并转义Rust关键字
        self.escape_keyword(&snake_cased)
    }

    /// 智能的camelCase到snake_case转换
    /// 专门处理常见的Java/JavaScript风格字段名
    pub fn smart_camel_to_snake(&mut self, name: &str) -> String {
        if let Some(cached) = self.cache.get(&format!("smart_{}", name)) {
            return cached.clone();
        }

        let result = match name {
            // 特殊情况的直接映射
            "openTime" => "open_time".to_string(),
            "maxCoinAmount" => "max_coin_amount".to_string(),
            "maxPcAmount" => "max_pc_amount".to_string(),
            "baseSide" => "base_side".to_string(),
            "initPcAmount" => "init_pc_amount".to_string(),
            "initCoinAmount" => "init_coin_amount".to_string(),
            "planOrderLimit" => "plan_order_limit".to_string(),
            "placeOrderLimit" => "place_order_limit".to_string(),
            "cancelOrderLimit" => "cancel_order_limit".to_string(),
            "newPubkey" => "new_pubkey".to_string(),
            "lastOrderDistance" => "last_order_distance".to_string(),
            "needTakeAmounts" => "need_take_amounts".to_string(),
            "swapBaseInValue" => "swap_base_in_value".to_string(),
            "swapBaseOutValue" => "swap_base_out_value".to_string(),
            "amountIn" => "amount_in".to_string(),
            "minimumAmountOut" => "minimum_amount_out".to_string(),
            "maxAmountIn" => "max_amount_in".to_string(),
            "amountOut" => "amount_out".to_string(),
            "pnlOwner" => "pnl_owner".to_string(),
            "cancelOwner" => "cancel_owner".to_string(),
            "createPoolFee" => "create_pool_fee".to_string(),
            "orderNum" => "order_num".to_string(),
            "coinDecimals" => "coin_decimals".to_string(),
            "pcDecimals" => "pc_decimals".to_string(),
            "resetFlag" => "reset_flag".to_string(),
            "minSize" => "min_size".to_string(),
            "volMaxCutRatio" => "vol_max_cut_ratio".to_string(),
            "amountWave" => "amount_wave".to_string(),
            "coinLotSize" => "coin_lot_size".to_string(),
            "pcLotSize" => "pc_lot_size".to_string(),
            "minPriceMultiplier" => "min_price_multiplier".to_string(),
            "maxPriceMultiplier" => "max_price_multiplier".to_string(),
            "sysDecimalValue" => "sys_decimal_value".to_string(),
            "outPut" => "out_put".to_string(), // 保持原有拼写
            "tokenCoin" => "token_coin".to_string(),
            "tokenPc" => "token_pc".to_string(),
            "coinMint" => "coin_mint".to_string(),
            "pcMint" => "pc_mint".to_string(),
            "lpMint" => "lp_mint".to_string(),
            "openOrders" => "open_orders".to_string(),
            "serumDex" => "serum_dex".to_string(),
            "targetOrders" => "target_orders".to_string(),
            "withdrawQueue" => "withdraw_queue".to_string(),
            "tokenTempLp" => "token_temp_lp".to_string(),
            "ammOwner" => "amm_owner".to_string(),
            "lpAmount" => "lp_amount".to_string(),
            "clientOrderId" => "client_order_id".to_string(),
            "buyOrders" => "buy_orders".to_string(),
            "targetX" => "target_x".to_string(),
            "targetY" => "target_y".to_string(),
            "planXBuy" => "plan_x_buy".to_string(),
            "planYBuy" => "plan_y_buy".to_string(),
            "planXSell" => "plan_x_sell".to_string(),
            "planYSell" => "plan_y_sell".to_string(),
            "placedX" => "placed_x".to_string(),
            "placedY" => "placed_y".to_string(),
            "calcPnlX" => "calc_pnl_x".to_string(),
            "calcPnlY" => "calc_pnl_y".to_string(),
            "sellOrders" => "sell_orders".to_string(),
            "replaceBuyClientId" => "replace_buy_client_id".to_string(),
            "replaceSellClientId" => "replace_sell_client_id".to_string(),
            "lastOrderNumerator" => "last_order_numerator".to_string(),
            "lastOrderDenominator" => "last_order_denominator".to_string(),
            _ => {
                // 使用改进的算法处理其他情况
                self.advanced_camel_to_snake(name)
            }
        };

        self.cache.insert(format!("smart_{}", name), result.clone());
        result
    }

    /// 高级camelCase到snake_case转换算法
    fn advanced_camel_to_snake(&self, name: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = name.chars().collect();
        
        for (i, &ch) in chars.iter().enumerate() {
            if i > 0 && ch.is_uppercase() {
                // 检查是否需要插入下划线
                let prev_char = chars[i - 1];
                let next_char = chars.get(i + 1).copied();
                
                // 插入下划线的条件：
                // 1. 前一个字符是小写
                // 2. 前一个字符是数字
                // 3. 下一个字符是小写（处理连续大写字母的情况）
                let should_insert_underscore = prev_char.is_lowercase() 
                    || prev_char.is_numeric() 
                    || (next_char.map_or(false, |nc| nc.is_lowercase()) && prev_char.is_uppercase());
                
                if should_insert_underscore {
                    result.push('_');
                }
            }
            
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        }
        
        // 后处理：处理特殊的数字和字母组合
        result = self.post_process_snake_case(&result);
        
        result
    }

    /// 后处理snake_case，处理特殊情况
    fn post_process_snake_case(&self, input: &str) -> String {
        let mut result = input.to_string();
        
        // 处理常见的缩写词
        let replacements = vec![
            ("p_c_", "pc_"),      // PC相关
            ("_p_c", "_pc"),      // PC相关
            ("_i_d", "_id"),      // ID相关
            ("i_d_", "id_"),      // ID相关
            ("_u_r_l", "_url"),   // URL相关
            ("_a_p_i", "_api"),   // API相关
            ("_u_i", "_ui"),      // UI相关
            ("_d_b", "_db"),      // DB相关
        ];
        
        for (pattern, replacement) in replacements {
            result = result.replace(pattern, replacement);
        }
        
        result
    }

    /// 智能转换结构体名
    pub fn convert_struct_name(&mut self, name: &str) -> String {
        // 移除可能的前缀或后缀
        let cleaned = self.clean_struct_name(name);
        self.to_pascal_case(&cleaned)
    }

    /// 智能转换指令结构体名（保留IxData后缀）
    pub fn convert_instruction_struct_name(&mut self, name: &str) -> String {
        // 对指令结构体，保留Data后缀，确保生成XxxIxData格式
        let pascal_name = self.to_pascal_case(name);
        if pascal_name.ends_with("IxData") {
            pascal_name
        } else if pascal_name.ends_with("Ix") {
            format!("{}Data", pascal_name)
        } else {
            format!("{}IxData", pascal_name)
        }
    }

    /// 智能转换枚举变体名
    pub fn convert_enum_variant(&mut self, name: &str) -> String {
        // 枚举变体使用PascalCase
        self.to_pascal_case(name)
    }

    /// 智能转换函数名
    pub fn convert_function_name(&mut self, name: &str) -> String {
        self.to_snake_case(name)
    }

    /// 智能转换常量名
    pub fn convert_constant_name(&mut self, name: &str) -> String {
        self.to_screaming_snake_case(name)
    }

    /// 预处理字符串以便更好地转换为snake_case
    fn preprocess_for_snake_case(&self, name: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = name.chars().collect();
        
        for (i, &ch) in chars.iter().enumerate() {
            if ch.is_uppercase() && i > 0 {
                // 检查前一个字符是否为小写，或者下一个字符是否为小写
                let prev_is_lower = chars.get(i - 1).map_or(false, |c| c.is_lowercase());
                let next_is_lower = chars.get(i + 1).map_or(false, |c| c.is_lowercase());
                
                if prev_is_lower || next_is_lower {
                    result.push('_');
                }
            }
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        }
        
        result
    }

    /// 清理字段名（移除不需要的前缀后缀）
    fn clean_field_name(&self, name: &str) -> String {
        let mut cleaned = name.to_string();
        
        // 移除常见前缀
        let prefixes = ["m_", "_", "f_"];
        for prefix in &prefixes {
            if cleaned.starts_with(prefix) {
                cleaned = cleaned[prefix.len()..].to_string();
                break;
            }
        }
        
        // 移除常见后缀
        let suffixes = ["_", "Field"];
        for suffix in &suffixes {
            if cleaned.ends_with(suffix) {
                cleaned = cleaned[..cleaned.len() - suffix.len()].to_string();
                break;
            }
        }
        
        cleaned
    }

    /// 清理结构体名
    fn clean_struct_name(&self, name: &str) -> String {
        let mut cleaned = name.to_string();
        
        // 移除常见后缀
        let suffixes = ["Struct", "Type", "Data"];
        for suffix in &suffixes {
            if cleaned.ends_with(suffix) && cleaned.len() > suffix.len() {
                cleaned = cleaned[..cleaned.len() - suffix.len()].to_string();
                break;
            }
        }
        
        cleaned
    }

    /// 检查名称是否符合Rust命名规范
    pub fn is_valid_rust_identifier(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        let mut chars = name.chars();
        
        // 第一个字符必须是字母或下划线
        if let Some(first_char) = chars.next() {
            if !first_char.is_alphabetic() && first_char != '_' {
                return false;
            }
        }

        // 其余字符必须是字母、数字或下划线
        for ch in chars {
            if !ch.is_alphanumeric() && ch != '_' {
                return false;
            }
        }

        // 不能是Rust关键字
        !self.is_rust_keyword(name)
    }

    /// 检查是否为Rust关键字
    fn is_rust_keyword(&self, name: &str) -> bool {
        matches!(name,
            "as" | "break" | "const" | "continue" | "crate" | "else" | "enum" | "extern" |
            "false" | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" |
            "mod" | "move" | "mut" | "pub" | "ref" | "return" | "self" | "Self" |
            "static" | "struct" | "super" | "trait" | "true" | "type" | "unsafe" |
            "use" | "where" | "while" | "async" | "await" | "dyn" | "abstract" |
            "become" | "box" | "do" | "final" | "macro" | "override" | "priv" |
            "typeof" | "unsized" | "virtual" | "yield" | "try"
        )
    }

    /// 转义Rust关键字（添加下划线后缀）
    pub fn escape_keyword(&self, name: &str) -> String {
        if self.is_rust_keyword(name) {
            format!("{}_", name)
        } else {
            name.to_string()
        }
    }

    /// 清空缓存
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// 获取缓存统计信息
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), self.special_mappings.len())
    }
}

impl Default for NamingConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_case_conversion() {
        let mut converter = NamingConverter::new();
        
        assert_eq!(converter.to_snake_case("camelCase"), "camel_case");
        assert_eq!(converter.to_snake_case("PascalCase"), "pascal_case");
        assert_eq!(converter.to_snake_case("HTTPSConnection"), "https_connection");
        assert_eq!(converter.to_snake_case("publicKey"), "pubkey");
        assert_eq!(converter.to_snake_case("snake_case"), "snake_case");
    }

    #[test]
    fn test_pascal_case_conversion() {
        let mut converter = NamingConverter::new();
        
        assert_eq!(converter.to_pascal_case("camel_case"), "CamelCase");
        assert_eq!(converter.to_pascal_case("snake_case"), "SnakeCase");
        assert_eq!(converter.to_pascal_case("already_pascal"), "AlreadyPascal");
    }

    #[test]
    fn test_field_name_conversion() {
        let mut converter = NamingConverter::new();
        
        assert_eq!(converter.convert_field_name("someField"), "some_field");
        assert_eq!(converter.convert_field_name("m_field"), "field");
        assert_eq!(converter.convert_field_name("_hiddenField"), "hidden_field");
        assert_eq!(converter.convert_field_name("publicKey"), "pubkey");
    }

    #[test]
    fn test_struct_name_conversion() {
        let mut converter = NamingConverter::new();
        
        assert_eq!(converter.convert_struct_name("user_data"), "UserData");
        assert_eq!(converter.convert_struct_name("ConfigStruct"), "Config");
        assert_eq!(converter.convert_struct_name("ResponseType"), "Response");
    }

    #[test]
    fn test_instruction_struct_name_conversion() {
        let mut converter = NamingConverter::new();
        
        assert_eq!(converter.convert_instruction_struct_name("buy_exact_in"), "BuyExactInIxData");
        assert_eq!(converter.convert_instruction_struct_name("BuyExactInIx"), "BuyExactInIxData");
        assert_eq!(converter.convert_instruction_struct_name("BuyExactInIxData"), "BuyExactInIxData");
        assert_eq!(converter.convert_instruction_struct_name("initialize"), "InitializeIxData");
    }

    #[test]
    fn test_rust_identifier_validation() {
        let converter = NamingConverter::new();
        
        assert!(converter.is_valid_rust_identifier("valid_name"));
        assert!(converter.is_valid_rust_identifier("ValidName"));
        assert!(converter.is_valid_rust_identifier("_private"));
        assert!(converter.is_valid_rust_identifier("name123"));
        
        assert!(!converter.is_valid_rust_identifier("123invalid"));
        assert!(!converter.is_valid_rust_identifier("invalid-name"));
        assert!(!converter.is_valid_rust_identifier("struct"));
        assert!(!converter.is_valid_rust_identifier(""));
    }

    #[test]
    fn test_keyword_escaping() {
        let converter = NamingConverter::new();
        
        assert_eq!(converter.escape_keyword("struct"), "struct_");
        assert_eq!(converter.escape_keyword("type"), "type_");
        assert_eq!(converter.escape_keyword("normal_name"), "normal_name");
    }
}