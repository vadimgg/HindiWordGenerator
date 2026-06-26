use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Error)]
pub enum AtomicFileError {
    #[error("target already exists: {0}")]
    Collision(PathBuf),
    #[error("target parent does not exist: {0}")]
    MissingParent(PathBuf),
    #[error("file operation failed at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

pub fn create_atomic(target: &Path, bytes: &[u8]) -> Result<(), AtomicFileError> {
    require_parent(target)?;
    if target.exists() {
        return Err(AtomicFileError::Collision(target.to_path_buf()));
    }
    let temp = unique_sibling(target);
    write_new_and_sync(&temp, bytes)?;
    if let Err(source) = fs::rename(&temp, target) {
        let _ = fs::remove_file(&temp);
        return Err(AtomicFileError::Io {
            path: target.to_path_buf(),
            source,
        });
    }
    sync_parent(target)?;
    Ok(())
}

pub fn replace_atomic(target: &Path, bytes: &[u8]) -> Result<(), AtomicFileError> {
    require_parent(target)?;
    let temp = unique_sibling(target);
    write_new_and_sync(&temp, bytes)?;
    if let Err(source) = fs::rename(&temp, target) {
        let _ = fs::remove_file(&temp);
        return Err(AtomicFileError::Io {
            path: target.to_path_buf(),
            source,
        });
    }
    sync_parent(target)?;
    Ok(())
}

fn require_parent(target: &Path) -> Result<(), AtomicFileError> {
    let Some(parent) = target.parent() else {
        return Err(AtomicFileError::MissingParent(target.to_path_buf()));
    };
    if !parent.is_dir() {
        return Err(AtomicFileError::MissingParent(parent.to_path_buf()));
    }
    Ok(())
}

fn write_new_and_sync(path: &Path, bytes: &[u8]) -> Result<(), AtomicFileError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| AtomicFileError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    if let Err(source) = file.write_all(bytes).and_then(|_| file.sync_all()) {
        let _ = fs::remove_file(path);
        return Err(AtomicFileError::Io {
            path: path.to_path_buf(),
            source,
        });
    }
    Ok(())
}

fn sync_parent(target: &Path) -> Result<(), AtomicFileError> {
    let Some(parent) = target.parent() else {
        return Ok(());
    };
    let directory = OpenOptions::new()
        .read(true)
        .open(parent)
        .map_err(|source| AtomicFileError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    directory.sync_all().map_err(|source| AtomicFileError::Io {
        path: parent.to_path_buf(),
        source,
    })
}

fn unique_sibling(target: &Path) -> PathBuf {
    let file_name = target
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("lingo-data");
    loop {
        let count = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let candidate =
            target.with_file_name(format!(".{file_name}.tmp-{}-{count}", std::process::id()));
        if !candidate.exists() {
            return candidate;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AtomicFileError, create_atomic};

    #[test]
    fn create_refuses_collision() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.json");
        std::fs::write(&path, "old").unwrap();
        assert!(matches!(
            create_atomic(&path, b"new"),
            Err(AtomicFileError::Collision(_))
        ));
        assert_eq!(std::fs::read_to_string(path).unwrap(), "old");
    }
}
