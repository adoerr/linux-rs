pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Other error
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
