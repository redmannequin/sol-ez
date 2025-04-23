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

pub fn quick_pubkey_check(pubkey: &str) -> bool {
    const MIN_CHARS: usize = 32;
    const MAX_CHARS: usize = 44;

    let bytes = pubkey.as_bytes();
    let len = bytes.len();

    if len < MIN_CHARS || len > MAX_CHARS {
        return false;
    }

    bytes.iter().all(|b| match b {
        b'1'..=b'9' | b'A'..=b'H' | b'J'..=b'N' | b'P'..=b'Z' | b'a'..=b'k' | b'm'..=b'z' => true,
        _ => false,
    })
}
