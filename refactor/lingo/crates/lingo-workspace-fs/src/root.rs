use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceRoot(PathBuf);

#[derive(Debug, Error)]
pub enum RootError {
    #[error("lingo workspace not found from {0}")]
    NotFound(PathBuf),
    #[error("workspace target is not a directory: {0}")]
    NotDirectory(PathBuf),
    #[error("could not canonicalize workspace path {path}: {source}")]
    Canonicalize {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl WorkspaceRoot {
    pub fn discover(from: &Path) -> Result<Self, RootError> {
        let start = if from.is_file() {
            from.parent().unwrap_or(from)
        } else {
            from
        };
        for candidate in start.ancestors() {
            if candidate.join("config.toml").is_file() {
                return Self::from_existing(candidate);
            }
        }
        Err(RootError::NotFound(from.to_path_buf()))
    }

    pub fn from_existing(path: &Path) -> Result<Self, RootError> {
        if !path.is_dir() {
            return Err(RootError::NotDirectory(path.to_path_buf()));
        }
        let canonical = path
            .canonicalize()
            .map_err(|source| RootError::Canonicalize {
                path: path.to_path_buf(),
                source,
            })?;
        Ok(Self(canonical))
    }

    pub fn new_target(path: &Path) -> Result<Self, RootError> {
        if path.exists() && !path.is_dir() {
            return Err(RootError::NotDirectory(path.to_path_buf()));
        }
        std::fs::create_dir_all(path).map_err(|source| RootError::Canonicalize {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_existing(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn join(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.0.join(relative)
    }
}
