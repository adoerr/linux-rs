//!
//! This file is part of syscall-rs
//!

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // System call error
    #[error("System call error: {0}")]
    Syscall(#[from] std::io::Error),
}
