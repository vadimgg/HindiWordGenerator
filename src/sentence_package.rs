use crate::project::{ProjectRoot, ProjectRootError};
use crate::run_report::unix_now;
use crate::sentence_schema::{parse_sentence_batch, SentenceBatch};
use serde::Serialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

const SENTENCE_OUTPUT_DIR: &str = "output/sentences";
const SENTENCE_PACKAGE_TYPE: &str = "hindi.sentences";

#[derive(Debug)]
pub enum SentencePackageError {
    Project(ProjectRootError),
    Io {
        path: PathBuf,
        source: io::Error,
    },
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    DestinationNotEmpty(PathBuf),
    DestinationIsFile(PathBuf),
    UnsafeAudioPath {
        batch: PathBuf,
        audio: String,
    },
    NoSentenceOutput,
}

impl std::fmt::Display for SentencePackageError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SentencePackageError::Project(error) => write!(formatter, "{error}"),
            SentencePackageError::Io { path, source } => {
                write!(formatter, "Could not access {}\n\n{source}", path.display())
            }
            SentencePackageError::Json { path, source } => {
                write!(formatter, "Could not parse {}\n\n{source}", path.display())
            }
            SentencePackageError::DestinationNotEmpty(path) => write!(
                formatter,
                "Destination is not empty: {}\n\nChoose a new folder for this package.",
                path.display()
            ),
            SentencePackageError::DestinationIsFile(path) => {
                write!(formatter, "Destination is a file: {}", path.display())
            }
            SentencePackageError::UnsafeAudioPath { batch, audio } => write!(
                formatter,
                "Unsafe audio path in {}\n\nAudio path: {audio:?}",
                batch.display()
            ),
            SentencePackageError::NoSentenceOutput => write!(
                formatter,
                "No accepted sentence output found in output/sentences."
            ),
        }
    }
}

impl From<ProjectRootError> for SentencePackageError {
    fn from(error: ProjectRootError) -> Self {
        SentencePackageError::Project(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentencePackageOutcome {
    pub destination: PathBuf,
    pub groups: usize,
    pub batch_files: usize,
    pub cards: usize,
    pub audio_copied: usize,
    pub missing_audio: usize,
    pub manifest: PathBuf,
}

impl SentencePackageOutcome {
    pub fn render(&self) -> String {
        format!(
            "Sentence Package\n\n  destination      {}\n  groups           {}\n  batch files      {}\n  cards            {}\n  audio copied     {}\n  missing audio    {}\n  manifest         {}\n\nCopied\n  output/sentences/\n  audio/sentences/\n\nIndexes\n  indexes/sentences.jsonl\n  indexes/missing_audio.json\n\nNext\n  Use {} as the package root for further processing.",
            self.destination.display(),
            self.groups,
            self.batch_files,
            self.cards,
            self.audio_copied,
            self.missing_audio,
            self.manifest.display(),
            self.destination.display()
        )
    }
}

#[derive(Debug, Serialize)]
struct PackageManifest {
    package_type: &'static str,
    created_at_unix: u64,
    path_base: &'static str,
    source_project: String,
    counts: PackageCounts,
    groups: Vec<PackageGroup>,
    files: PackageFiles,
}

#[derive(Debug, Serialize)]
struct PackageCounts {
    batch_files: usize,
    cards: usize,
    audio_files: usize,
    missing_audio: usize,
}

#[derive(Debug, Serialize)]
struct PackageGroup {
    title: String,
    subtitle: String,
    cards: usize,
}

#[derive(Debug, Serialize)]
struct PackageFiles {
    sentences: Vec<String>,
    audio: Vec<String>,
    missing_audio: Vec<MissingAudio>,
}

#[derive(Debug, Clone, Serialize)]
struct MissingAudio {
    sentence_file: String,
    item_index: usize,
    audio: Option<String>,
}

struct LoadedBatch {
    relative_path: PathBuf,
    raw: Vec<u8>,
    parsed: SentenceBatch,
}

pub fn package_from_current_dir(
    dest: impl AsRef<Path>,
) -> Result<SentencePackageOutcome, SentencePackageError> {
    let root = ProjectRoot::discover_from(current_dir()?)?;
    package_from(&root, dest)
}

pub fn package_from(
    root: &ProjectRoot,
    dest: impl AsRef<Path>,
) -> Result<SentencePackageOutcome, SentencePackageError> {
    let dest = dest.as_ref().to_path_buf();
    prepare_destination(&dest)?;
    let batches = load_batches(root)?;
    if batches.is_empty() {
        return Err(SentencePackageError::NoSentenceOutput);
    }

    let mut sentence_files = Vec::new();
    let mut audio_files = BTreeSet::new();
    let mut missing_audio = Vec::new();
    let mut group_counts: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut card_count = 0usize;

    for batch in &batches {
        copy_file_bytes(&dest, &batch.relative_path, &batch.raw)?;
        sentence_files.push(path_string(&batch.relative_path));
        let title = batch.parsed.title.clone().unwrap_or_default();
        let subtitle = batch.parsed.subtitle.clone().unwrap_or_default();
        let group_key = (title, subtitle);
        *group_counts.entry(group_key).or_default() += batch.parsed.sentences.len();

        for (index, sentence) in batch.parsed.sentences.iter().enumerate() {
            card_count += 1;
            let Some(audio) = sentence
                .audio
                .as_deref()
                .filter(|value| !value.trim().is_empty())
            else {
                missing_audio.push(MissingAudio {
                    sentence_file: path_string(&batch.relative_path),
                    item_index: index + 1,
                    audio: None,
                });
                continue;
            };
            let audio_path = safe_relative_audio_path(audio).ok_or_else(|| {
                SentencePackageError::UnsafeAudioPath {
                    batch: batch.relative_path.clone(),
                    audio: audio.to_string(),
                }
            })?;
            if root.join(&audio_path).is_file() {
                audio_files.insert(audio_path);
            } else {
                missing_audio.push(MissingAudio {
                    sentence_file: path_string(&batch.relative_path),
                    item_index: index + 1,
                    audio: Some(audio.to_string()),
                });
            }
        }
    }

    let mut audio_file_list = Vec::new();
    for audio_path in &audio_files {
        let bytes = fs::read(root.join(audio_path)).map_err(|source| SentencePackageError::Io {
            path: root.join(audio_path),
            source,
        })?;
        copy_file_bytes(&dest, audio_path, &bytes)?;
        audio_file_list.push(path_string(audio_path));
    }

    let groups = group_counts
        .into_iter()
        .map(|((title, subtitle), cards)| PackageGroup {
            title,
            subtitle,
            cards,
        })
        .collect::<Vec<_>>();

    write_indexes(&dest, &batches, &missing_audio)?;
    let manifest = PackageManifest {
        package_type: SENTENCE_PACKAGE_TYPE,
        created_at_unix: unix_now(),
        path_base: "package_root",
        source_project: root.path().display().to_string(),
        counts: PackageCounts {
            batch_files: sentence_files.len(),
            cards: card_count,
            audio_files: audio_file_list.len(),
            missing_audio: missing_audio.len(),
        },
        groups,
        files: PackageFiles {
            sentences: sentence_files,
            audio: audio_file_list,
            missing_audio,
        },
    };
    let manifest_path = PathBuf::from("manifest.json");
    write_pretty_json(&dest, &manifest_path, &manifest)?;

    Ok(SentencePackageOutcome {
        destination: dest,
        groups: manifest.groups.len(),
        batch_files: manifest.counts.batch_files,
        cards: manifest.counts.cards,
        audio_copied: manifest.counts.audio_files,
        missing_audio: manifest.counts.missing_audio,
        manifest: manifest_path,
    })
}

fn prepare_destination(dest: &Path) -> Result<(), SentencePackageError> {
    if dest.is_file() {
        return Err(SentencePackageError::DestinationIsFile(dest.to_path_buf()));
    }
    if dest.is_dir() {
        let mut entries = fs::read_dir(dest).map_err(|source| SentencePackageError::Io {
            path: dest.to_path_buf(),
            source,
        })?;
        if entries
            .next()
            .transpose()
            .map_err(|source| SentencePackageError::Io {
                path: dest.to_path_buf(),
                source,
            })?
            .is_some()
        {
            return Err(SentencePackageError::DestinationNotEmpty(
                dest.to_path_buf(),
            ));
        }
    } else {
        fs::create_dir_all(dest).map_err(|source| SentencePackageError::Io {
            path: dest.to_path_buf(),
            source,
        })?;
    }
    Ok(())
}

fn load_batches(root: &ProjectRoot) -> Result<Vec<LoadedBatch>, SentencePackageError> {
    let mut batches = Vec::new();
    for relative_path in collect_sentence_batch_paths(root)? {
        let path = root.join(&relative_path);
        let raw = fs::read(&path).map_err(|source| SentencePackageError::Io {
            path: path.clone(),
            source,
        })?;
        let parsed =
            parse_sentence_batch(std::str::from_utf8(&raw).unwrap_or("")).map_err(|source| {
                SentencePackageError::Json {
                    path: path.clone(),
                    source,
                }
            })?;
        batches.push(LoadedBatch {
            relative_path,
            raw,
            parsed,
        });
    }
    Ok(batches)
}

fn collect_sentence_batch_paths(root: &ProjectRoot) -> Result<Vec<PathBuf>, SentencePackageError> {
    let dir = root.join(SENTENCE_OUTPUT_DIR);
    let mut paths = Vec::new();
    let entries = fs::read_dir(&dir).map_err(|source| SentencePackageError::Io {
        path: dir.clone(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| SentencePackageError::Io {
            path: dir.clone(),
            source,
        })?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some("json") {
            paths.push(
                path.strip_prefix(root.path())
                    .unwrap_or(&path)
                    .to_path_buf(),
            );
        }
    }
    paths.sort();
    Ok(paths)
}

fn safe_relative_audio_path(value: &str) -> Option<PathBuf> {
    let path = Path::new(value);
    if path.is_absolute() {
        return None;
    }
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)))
    {
        return None;
    }
    Some(path.to_path_buf())
}

fn copy_file_bytes(
    dest: &Path,
    relative_path: &Path,
    bytes: &[u8],
) -> Result<(), SentencePackageError> {
    let target = dest.join(relative_path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|source| SentencePackageError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(&target, bytes).map_err(|source| SentencePackageError::Io {
        path: target,
        source,
    })
}

fn write_indexes(
    dest: &Path,
    batches: &[LoadedBatch],
    missing_audio: &[MissingAudio],
) -> Result<(), SentencePackageError> {
    let mut jsonl = String::new();
    for batch in batches {
        for (index, sentence) in batch.parsed.sentences.iter().enumerate() {
            let line = json!({
                "sentence_file": path_string(&batch.relative_path),
                "item_index": index + 1,
                "title": batch.parsed.title,
                "subtitle": batch.parsed.subtitle,
                "hindi": sentence.hindi,
                "romanisation": sentence.romanisation,
                "english": sentence.english,
                "audio": sentence.audio,
            });
            jsonl.push_str(&serde_json::to_string(&line).map_err(|source| {
                SentencePackageError::Json {
                    path: PathBuf::from("indexes/sentences.jsonl"),
                    source,
                }
            })?);
            jsonl.push('\n');
        }
    }
    copy_file_bytes(dest, Path::new("indexes/sentences.jsonl"), jsonl.as_bytes())?;
    write_pretty_json(dest, Path::new("indexes/missing_audio.json"), missing_audio)
}

fn write_pretty_json<T: Serialize + ?Sized>(
    dest: &Path,
    relative_path: &Path,
    value: &T,
) -> Result<(), SentencePackageError> {
    let mut bytes =
        serde_json::to_vec_pretty(value).map_err(|source| SentencePackageError::Json {
            path: relative_path.to_path_buf(),
            source,
        })?;
    bytes.push(b'\n');
    copy_file_bytes(dest, relative_path, &bytes)
}

fn current_dir() -> Result<PathBuf, SentencePackageError> {
    std::env::current_dir().map_err(|source| SentencePackageError::Io {
        path: PathBuf::from("."),
        source,
    })
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::package_from;
    use crate::project::ProjectRoot;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn packages_sentence_json_and_referenced_audio() {
        let root = fixture_root();
        let audio = "audio/sentences/example_batch_01/01_namaste.mp3";
        fs::create_dir_all(root.join("audio/sentences/example_batch_01")).unwrap();
        fs::write(root.join(audio), b"mp3").unwrap();
        fs::write(
            root.join("output/sentences/example_batch_01.json"),
            format!(
                r#"{{
  "title": "Complete Hindi",
  "subtitle": "Chapter 02",
  "sentences": [
    {{
      "hindi": "नमस्ते।",
      "romanisation": "namaste.",
      "english": "Hello.",
      "audio": "{audio}"
    }}
  ]
}}"#
            ),
        )
        .unwrap();
        let project = ProjectRoot::discover_from(&root).unwrap();
        let dest = temp_path("hindi-package-dest");

        let outcome = package_from(&project, &dest).unwrap();

        assert_eq!(outcome.batch_files, 1);
        assert_eq!(outcome.cards, 1);
        assert_eq!(outcome.audio_copied, 1);
        assert_eq!(outcome.missing_audio, 0);
        assert!(dest.join("manifest.json").is_file());
        assert!(dest
            .join("output/sentences/example_batch_01.json")
            .is_file());
        assert!(dest.join(audio).is_file());
        assert!(dest.join("indexes/sentences.jsonl").is_file());
        assert!(dest.join("indexes/missing_audio.json").is_file());
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(dest).unwrap();
    }

    #[test]
    fn refuses_non_empty_destination() {
        let root = fixture_root();
        fs::write(
            root.join("output/sentences/example_batch_01.json"),
            r#"{"sentences":[]}"#,
        )
        .unwrap();
        let project = ProjectRoot::discover_from(&root).unwrap();
        let dest = temp_path("hindi-package-non-empty");
        fs::create_dir_all(&dest).unwrap();
        fs::write(dest.join("old.txt"), "").unwrap();

        let error = package_from(&project, &dest).unwrap_err();

        assert!(error.to_string().contains("Destination is not empty"));
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(dest).unwrap();
    }

    fn fixture_root() -> PathBuf {
        let root = temp_path("hindi-package-root");
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::create_dir_all(root.join("input")).unwrap();
        fs::create_dir_all(root.join("output/sentences")).unwrap();
        fs::write(root.join("docs/DESIGN.md"), "").unwrap();
        fs::write(root.join("docs/ROADMAP.md"), "").unwrap();
        root
    }

    fn temp_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{label}-{}-{nanos}", std::process::id()))
    }
}
