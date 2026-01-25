//!
//! This file is part of syscall-rs
//!

/// Result type
pub type Result<T> = std::result::Result<T, Error>;

/// Error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// System call error
    #[error("System call error: {0}")]
    Syscall(#[from] std::io::Error),

    /// Interior nul byte error
    #[error("Nul byte error: {0}")]
    Nul(#[from] std::ffi::NulError),

    /// Nix errno
    #[error("Nix error: {0}")]
    Nix(#[from] nix::errno::Errno),
}
