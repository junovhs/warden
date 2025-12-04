// src/error.rs
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SlopChopError {
    #[error("I/O error: {source} (path: {path})")]
    Io {
        source: std::io::Error,
        path: PathBuf,
    },

    #[error("Not inside a Git repository")]
    NotInGitRepo,

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Generic error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, SlopChopError>;

// Allow `?` on std::io::Error by converting to SlopChopError::Io with unknown path.
impl From<std::io::Error> for SlopChopError {
    fn from(source: std::io::Error) -> Self {
        SlopChopError::Io {
            source,
            path: PathBuf::from("<unknown>"),
        }
    }
}

// Gracefully convert WalkDir errors
impl From<walkdir::Error> for SlopChopError {
    fn from(e: walkdir::Error) -> Self {
        SlopChopError::Other(e.to_string())
    }
}
