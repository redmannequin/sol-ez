use std::{io, num::ParseIntError};

#[derive(Debug, thiserror::Error)]
pub enum SolGenError {
    #[error("expected {0} found {1}")]
    ExpectedToken(String, String),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error("invalid char '{0}'")]
    InvalidChar(char),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
