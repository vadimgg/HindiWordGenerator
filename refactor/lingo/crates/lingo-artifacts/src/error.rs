use lingo_application::ports::ArtifactFailure;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("artifact would contain unsafe path {0:?}")]
    UnsafePath(String),
    #[error("artifact contains duplicate path {0}")]
    DuplicatePath(String),
    #[error("artifact I/O failed at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("artifact encoding failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("artifact zip failed: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Anki database failed: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("artifact integrity verification failed for {0}")]
    Integrity(PathBuf),
    #[error("artifact plan is invalid: {0}")]
    Invalid(String),
}

impl From<ArtifactError> for ArtifactFailure {
    fn from(error: ArtifactError) -> Self {
        Self(error.to_string())
    }
}

pub(crate) fn io_at(path: impl Into<PathBuf>, source: std::io::Error) -> ArtifactError {
    ArtifactError::Io {
        path: path.into(),
        source,
    }
}
