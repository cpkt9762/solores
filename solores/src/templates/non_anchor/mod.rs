//! 非 Anchor 模式模板
//!
//! 为非 Anchor 合约提供模板生成，支持 1 字节 discriminator 或基于长度的识别

pub mod instructions_template;
pub mod accounts_template;
pub mod events_template;
pub mod types_template;
pub mod errors_template;
pub mod parsers_template;
pub mod instructions_parser_test_template;
pub mod accounts_parser_test_template;

// Re-export templates for convenient access
pub use instructions_template::NonAnchorInstructionsTemplate;
pub use accounts_template::NonAnchorAccountsTemplate;
pub use events_template::NonAnchorEventsTemplate;
pub use types_template::NonAnchorTypesTemplate;
pub use errors_template::NonAnchorErrorsTemplate;
pub use parsers_template::NonAnchorParsersTemplate;
pub use instructions_parser_test_template::NonAnchorInstructionsParserTestTemplate;
pub use accounts_parser_test_template::NonAnchorAccountsParserTestTemplate;