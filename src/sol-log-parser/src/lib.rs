//! A small utility crate for parsing solana logs
pub use error::LogParseError;
pub use parsed_log::ParsedLog;
pub use raw_log::RawLog;
pub use structured_log::{parsed::ParsedStructuredLog, raw::RawStructuredLog};

pub mod error;
pub mod parsed_log;
pub mod raw_log;
pub mod structured_log;

pub type Result<T> = std::result::Result<T, LogParseError>;
