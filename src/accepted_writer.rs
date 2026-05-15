#![allow(dead_code)]

use crate::sentence_schema::SentenceBatch;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub enum AcceptedWriteError {
    Collision(PathBuf),
    MissingParent(PathBuf),
    Serialize(serde_json::Error),
    Io { path: PathBuf, source: io::Error },
}

impl std::fmt::Display for AcceptedWriteError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AcceptedWriteError::Collision(path) => {
                write!(
                    formatter,
                    "Accepted output already exists: {}",
                    path.display()
                )
            }
            AcceptedWriteError::MissingParent(path) => {
                write!(
                    formatter,
                    "Target directory does not exist: {}",
                    path.display()
                )
            }
            AcceptedWriteError::Serialize(error) => {
                write!(formatter, "Could not serialize accepted output: {error}")
            }
            AcceptedWriteError::Io { path, source } => {
                write!(formatter, "Could not write {}\n\n{source}", path.display())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedWriteResult {
    pub path: PathBuf,
}

pub fn write_sentence_batch(
    target: &Path,
    batch: &SentenceBatch,
) -> Result<AcceptedWriteResult, AcceptedWriteError> {
    if target.exists() {
        return Err(AcceptedWriteError::Collision(target.to_path_buf()));
    }
    let Some(parent) = target.parent() else {
        return Err(AcceptedWriteError::MissingParent(target.to_path_buf()));
    };
    if !parent.is_dir() {
        return Err(AcceptedWriteError::MissingParent(parent.to_path_buf()));
    }

    let mut bytes = serde_json::to_vec_pretty(batch).map_err(AcceptedWriteError::Serialize)?;
    bytes.push(b'\n');

    let temp = temp_path_for(target);
    let write_result = write_temp(&temp, &bytes);
    if let Err(error) = write_result {
        let _ = fs::remove_file(&temp);
        return Err(error);
    }
    if let Err(source) = fs::rename(&temp, target) {
        let _ = fs::remove_file(&temp);
        return Err(AcceptedWriteError::Io {
            path: target.to_path_buf(),
            source,
        });
    }

    Ok(AcceptedWriteResult {
        path: target.to_path_buf(),
    })
}

fn write_temp(path: &Path, bytes: &[u8]) -> Result<(), AcceptedWriteError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| AcceptedWriteError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|source| AcceptedWriteError::Io {
            path: path.to_path_buf(),
            source,
        })
}

fn temp_path_for(target: &Path) -> PathBuf {
    let file_name = target
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("accepted-output.json");
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    target.with_file_name(format!(".{file_name}.tmp-{}-{nanos}", std::process::id()))
}

#[cfg(test)]
mod tests {
    use super::{write_sentence_batch, AcceptedWriteError};
    use crate::sentence_schema::parse_sentence_batch;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn writes_sentence_batch_atomically() {
        let dir = temp_dir("accepted-writer-ok");
        fs::create_dir_all(&dir).unwrap();
        let target = dir.join("batch_01.json");

        let result = write_sentence_batch(&target, &batch()).unwrap();

        assert_eq!(result.path, target);
        let content = fs::read_to_string(&target).unwrap();
        assert!(content.contains("\"sentences\""));
        assert!(fs::read_dir(&dir).unwrap().all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains(".tmp-")));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn refuses_to_overwrite_existing_output() {
        let dir = temp_dir("accepted-writer-collision");
        fs::create_dir_all(&dir).unwrap();
        let target = dir.join("batch_01.json");
        fs::write(&target, "original").unwrap();

        let error = write_sentence_batch(&target, &batch()).unwrap_err();

        assert!(matches!(error, AcceptedWriteError::Collision(_)));
        assert_eq!(fs::read_to_string(&target).unwrap(), "original");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn missing_parent_does_not_create_target() {
        let dir = temp_dir("accepted-writer-missing-parent");
        let target = dir.join("missing").join("batch_01.json");

        let error = write_sentence_batch(&target, &batch()).unwrap_err();

        assert!(matches!(error, AcceptedWriteError::MissingParent(_)));
        assert!(!target.exists());
    }

    fn batch() -> crate::sentence_schema::SentenceBatch {
        parse_sentence_batch(
            r#"{
              "title": "Complete Hindi",
              "subtitle": "Chapter 02",
              "sentences": [{
                "hindi": "यहाँ",
                "romanisation": "yahā̃",
                "english": "Here.",
                "literal": "here",
                "register": "standard",
                "source_ref": {
                  "file": "input/sentences/example.yaml",
                  "item_id": "0001",
                  "fingerprint": "sha256:test"
                },
                "tokens": [{"hindi":"यहाँ","roman":"yahā̃","kind":"word","word_id":"w1"}],
                "words": [{"id":"w1","hindi":"यहाँ","roman":"yahā̃","meaning":"here"}]
              }]
            }"#,
        )
        .unwrap()
    }

    fn temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("hindi-{label}-{}-{nanos}", std::process::id()))
    }
}
