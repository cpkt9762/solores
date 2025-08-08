//! Anchor 模式模板
//!
//! 为 Anchor 合约提供模板生成，支持 8 字节 discriminator

pub mod instructions_template;
pub mod accounts_template;
pub mod events_template;
pub mod types_template;
pub mod errors_template;
pub mod parsers_template;
pub mod instructions_parser_test_template;
pub mod accounts_parser_test_template;

// Re-export templates for convenient access
pub use instructions_template::AnchorInstructionsTemplate;
pub use accounts_template::AnchorAccountsTemplate;
pub use events_template::AnchorEventsTemplate;
pub use types_template::AnchorTypesTemplate;
pub use errors_template::AnchorErrorsTemplate;
pub use parsers_template::AnchorParsersTemplate;
pub use instructions_parser_test_template::AnchorInstructionsParserTestTemplate;
pub use accounts_parser_test_template::AnchorAccountsParserTestTemplate;