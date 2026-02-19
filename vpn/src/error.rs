pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// IO error
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    /// Nix error
    #[error("Nix error: {0}")]
    Nix(#[from] nix::Error),

    /// Unknown token error
    #[error("Unknown token error: {0}")]
    Token(u64),

    /// Other error
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
