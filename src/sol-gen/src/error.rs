use std::io;

#[derive(Debug, thiserror::Error)]
pub enum SolGenError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
