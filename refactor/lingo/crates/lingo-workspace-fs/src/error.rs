use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FsAdapterError {
    #[error("I/O failed at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid data at {path}: {message}")]
    Invalid { path: PathBuf, message: String },
}
