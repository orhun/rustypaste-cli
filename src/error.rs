use thiserror::Error as ThisError;

/// Custom error type.
#[derive(Debug, ThisError)]
pub enum Error {
    /// Error that might occur while handling I/O operations.
    #[error("IO error: `{0}`")]
    IoError(#[from] std::io::Error),
    /// Error that might occur while parsing the configuration file.
    #[error("TOML parsing error: `{0}`")]
    TomlError(#[from] toml::de::Error),
    /// Error that might occur while processing/sending requests.
    #[error("Request error: `{0}`")]
    RequestError(#[from] Box<ureq::Error>),
    /// Error that might occur while uploading files.
    #[error("Upload error: `{0}`")]
    UploadError(String),
    /// Error that might occur when no server address is provided.
    #[error("No rustypaste server address is given.")]
    NoServerAddressError,
    /// Error that might occur during the preparation of the multipart data.
    #[error("Multipart IO error: `{0}`")]
    MultipartIOError(#[from] multipart::client::lazy::LazyError<'static, std::io::Error>),
}

/// Type alias for the Result type.
pub type Result<T> = std::result::Result<T, Error>;
