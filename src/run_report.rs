use crate::project::ProjectRoot;
use serde::Serialize;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const SENTENCE_RUN_DIR: &str = "runs/sentences";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SentenceRunReport {
    pub command: String,
    pub status: String,
    pub source_files: Vec<String>,
    pub targets: Vec<String>,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_digest: Option<String>,
    pub prompt_path: String,
    pub prompt_fingerprint: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stages: Vec<SentenceStageReport>,
    pub started_at_unix: u64,
    pub finished_at_unix: u64,
    pub validation: ValidationSummary,
    pub writes: WriteSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SentenceStageReport {
    pub stage_id: String,
    pub prompt_version: String,
    pub prompt_fingerprint: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_digest: Option<String>,
    pub duration_ms: u128,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationSummary {
    pub valid: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WriteSummary {
    pub accepted: Vec<String>,
    pub skipped: Vec<String>,
}

#[derive(Debug)]
pub enum RunReportError {
    Serialize(serde_json::Error),
    Io { path: PathBuf, source: io::Error },
}

impl std::fmt::Display for RunReportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunReportError::Serialize(error) => {
                write!(formatter, "Could not serialize run report: {error}")
            }
            RunReportError::Io { path, source } => {
                write!(
                    formatter,
                    "Could not write run report {}\n\n{source}",
                    path.display()
                )
            }
        }
    }
}

pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

pub fn write_sentence_run_report(
    root: &ProjectRoot,
    report: &SentenceRunReport,
) -> Result<PathBuf, RunReportError> {
    let dir = root.join(SENTENCE_RUN_DIR);
    fs::create_dir_all(&dir).map_err(|source| RunReportError::Io {
        path: dir.clone(),
        source,
    })?;
    let path = unique_report_path(&dir, &report.model);
    let mut bytes = serde_json::to_vec_pretty(report).map_err(RunReportError::Serialize)?;
    bytes.push(b'\n');
    fs::write(&path, bytes).map_err(|source| RunReportError::Io {
        path: path.clone(),
        source,
    })?;
    Ok(path
        .strip_prefix(root.path())
        .unwrap_or(&path)
        .to_path_buf())
}

fn unique_report_path(dir: &Path, model: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    dir.join(format!("{}_{}.json", nanos, slug(model)))
}

fn slug(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::slug;

    #[test]
    fn model_slug_is_filesystem_safe() {
        assert_eq!(
            slug("ollama:translategemma:12b"),
            "ollama_translategemma_12b"
        );
    }
}
