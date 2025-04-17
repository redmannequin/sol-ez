use std::num::ParseIntError;

#[derive(Debug, thiserror::Error)]
pub enum LogParseError {
    #[error(transparent)]
    Pubkey(#[from] solana_pubkey::ParsePubkeyError),
    #[error(transparent)]
    Int(#[from] ParseIntError),
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
}
