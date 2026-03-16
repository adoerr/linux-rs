use wayland_client::{ConnectError, DispatchError};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Wayland connection error
    #[error("Wayland connection error {0}")]
    Connect(#[from] ConnectError),

    /// Wayland dispatching error
    #[error("Wayland dispatching error {0}")]
    Dispatch(#[from] DispatchError),

    /// Other error
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
