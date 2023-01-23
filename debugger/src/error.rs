pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// IO error
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    /// Nul byte error
    #[error("Nul byte error: {0}")]
    Nul(#[from] std::ffi::NulError),

    /// Other error
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
