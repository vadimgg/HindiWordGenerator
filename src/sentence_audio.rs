use crate::project::{ProjectRoot, ProjectRootError};
use crate::tts::{TtsBackend, TtsError, UvGttsBackend};
use serde_json::Value;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const SENTENCE_OUTPUT_DIR: &str = "output/sentences";
const SENTENCE_AUDIO_DIR: &str = "audio/sentences";

#[derive(Debug)]
pub enum SentenceAudioError {
    Project(ProjectRootError),
    Io {
        path: PathBuf,
        source: io::Error,
    },
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    UnsupportedBatch(PathBuf),
    Tts {
        path: PathBuf,
        source: TtsError,
    },
}

impl std::fmt::Display for SentenceAudioError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SentenceAudioError::Project(error) => write!(formatter, "{error}"),
            SentenceAudioError::Io { path, source } => {
                write!(formatter, "Could not access {}\n\n{source}", path.display())
            }
            SentenceAudioError::Json { path, source } => {
                write!(formatter, "Could not parse {}\n\n{source}", path.display())
            }
            SentenceAudioError::UnsupportedBatch(path) => {
                write!(
                    formatter,
                    "Unsupported sentence batch schema: {}",
                    path.display()
                )
            }
            SentenceAudioError::Tts { path, source } => {
                write!(
                    formatter,
                    "Could not generate {}\n\n{source}",
                    path.display()
                )
            }
        }
    }
}

impl From<ProjectRootError> for SentenceAudioError {
    fn from(error: ProjectRootError) -> Self {
        SentenceAudioError::Project(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentenceAudioOutcome {
    pub success: bool,
    pub scanned_batches: usize,
    pub scanned_cards: usize,
    pub generated_mp3s: Vec<PathBuf>,
    pub patched_cards: usize,
    pub skipped_existing: usize,
    pub updated_output: Vec<PathBuf>,
    pub message: Option<String>,
    pub recovery: Option<String>,
}

impl SentenceAudioOutcome {
    pub fn render(&self) -> String {
        let mut output = String::from("Sentence Audio\n\n");
        if let Some(message) = &self.message {
            output.push_str("Problem\n");
            output.push_str(&format!("  {message}\n"));
            if let Some(recovery) = &self.recovery {
                output.push_str("\nRun\n");
                output.push_str(&format!("  {recovery}\n"));
            }
            return output;
        }

        output.push_str(&format!("  scanned batches    {}\n", self.scanned_batches));
        output.push_str(&format!("  scanned cards      {}\n", self.scanned_cards));
        output.push_str(&format!(
            "  generated mp3s     {}\n",
            self.generated_mp3s.len()
        ));
        output.push_str(&format!("  patched cards      {}\n", self.patched_cards));
        output.push_str(&format!("  skipped existing   {}\n", self.skipped_existing));

        if !self.generated_mp3s.is_empty() {
            output.push_str("\nGenerated Audio\n");
            for path in &self.generated_mp3s {
                output.push_str(&format!("  {}\n", path.display()));
            }
        }
        if !self.updated_output.is_empty() {
            output.push_str("\nUpdated Output\n");
            for path in &self.updated_output {
                output.push_str(&format!("  {}\n", path.display()));
            }
        }
        if self.generated_mp3s.is_empty() && self.patched_cards == 0 {
            output.push_str("\nNothing to do. Sentence audio is already complete.\n");
        } else {
            output.push_str("\nNext\n  hindi viewer\n");
        }
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentenceAudioPlan {
    batches: Vec<BatchAudioPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BatchAudioPlan {
    relative_path: PathBuf,
    entries: Vec<AudioEntryPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AudioEntryPlan {
    index: usize,
    hindi: String,
    audio_path: PathBuf,
    needs_synthesis: bool,
    needs_patch: bool,
    skipped_existing: bool,
}

pub fn audio_from_current_dir() -> Result<SentenceAudioOutcome, SentenceAudioError> {
    let root = ProjectRoot::discover_from(current_dir()?)?;
    audio_from(&root, &UvGttsBackend)
}

pub fn audio_from<B: TtsBackend>(
    root: &ProjectRoot,
    backend: &B,
) -> Result<SentenceAudioOutcome, SentenceAudioError> {
    let plan = plan_sentence_audio(root)?;
    let scanned_batches = plan.batches.len();
    if scanned_batches == 0 {
        return Ok(SentenceAudioOutcome {
            success: false,
            scanned_batches: 0,
            scanned_cards: 0,
            generated_mp3s: Vec::new(),
            patched_cards: 0,
            skipped_existing: 0,
            updated_output: Vec::new(),
            message: Some("No accepted sentence batches found in output/sentences.".to_string()),
            recovery: Some("hindi sentences generate --max-batches 1".to_string()),
        });
    }

    let scanned_cards = plan.scanned_cards();
    let skipped_existing = plan.skipped_existing();
    let mut generated_mp3s = Vec::new();
    let mut patched_cards = 0usize;
    let mut updated_output = Vec::new();

    for batch in &plan.batches {
        for entry in batch.entries.iter().filter(|entry| entry.needs_synthesis) {
            let relative = write_audio_file(root, entry, backend)?;
            generated_mp3s.push(relative);
        }
        let entries_to_patch = batch
            .entries
            .iter()
            .filter(|entry| entry.needs_patch)
            .collect::<Vec<_>>();
        if !entries_to_patch.is_empty() {
            patch_batch_audio(root, batch, &entries_to_patch)?;
            patched_cards += entries_to_patch.len();
            updated_output.push(batch.relative_path.clone());
        }
    }

    Ok(SentenceAudioOutcome {
        success: true,
        scanned_batches,
        scanned_cards,
        generated_mp3s,
        patched_cards,
        skipped_existing,
        updated_output,
        message: None,
        recovery: None,
    })
}

fn plan_sentence_audio(root: &ProjectRoot) -> Result<SentenceAudioPlan, SentenceAudioError> {
    let paths = collect_sentence_batch_paths(root)?;
    let mut batches = Vec::new();
    for relative_path in paths {
        let path = root.join(&relative_path);
        let content = fs::read_to_string(&path).map_err(|source| SentenceAudioError::Io {
            path: path.clone(),
            source,
        })?;
        let value: Value =
            serde_json::from_str(&content).map_err(|source| SentenceAudioError::Json {
                path: path.clone(),
                source,
            })?;
        let Some(sentences) = value.get("sentences").and_then(Value::as_array) else {
            return Err(SentenceAudioError::UnsupportedBatch(relative_path));
        };
        let stem = relative_path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("sentences");
        let mut entries = Vec::new();
        for (index, sentence) in sentences.iter().enumerate() {
            let index = index + 1;
            let hindi = sentence
                .get("hindi")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            if let Some(audio_path) = sentence
                .get("audio")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .map(PathBuf::from)
            {
                let needs_synthesis = !root.join(&audio_path).is_file();
                entries.push(AudioEntryPlan {
                    index,
                    hindi,
                    audio_path,
                    needs_synthesis,
                    needs_patch: false,
                    skipped_existing: !needs_synthesis,
                });
                continue;
            }
            let hint = sentence
                .get("romanisation")
                .or_else(|| sentence.get("english"))
                .or_else(|| sentence.get("hindi"))
                .and_then(Value::as_str)
                .unwrap_or("sentence");
            let relative_audio_path = Path::new(SENTENCE_AUDIO_DIR)
                .join(stem)
                .join(format!("{index:02}_{}.mp3", slug(hint)));
            let needs_synthesis = !root.join(&relative_audio_path).is_file();
            entries.push(AudioEntryPlan {
                index,
                hindi,
                audio_path: relative_audio_path,
                needs_synthesis,
                needs_patch: true,
                skipped_existing: false,
            });
        }
        batches.push(BatchAudioPlan {
            relative_path,
            entries,
        });
    }
    Ok(SentenceAudioPlan { batches })
}

impl SentenceAudioPlan {
    fn scanned_cards(&self) -> usize {
        self.batches.iter().map(|batch| batch.entries.len()).sum()
    }

    fn skipped_existing(&self) -> usize {
        self.batches
            .iter()
            .flat_map(|batch| &batch.entries)
            .filter(|entry| entry.skipped_existing)
            .count()
    }
}

fn collect_sentence_batch_paths(root: &ProjectRoot) -> Result<Vec<PathBuf>, SentenceAudioError> {
    let dir = root.join(SENTENCE_OUTPUT_DIR);
    let entries = fs::read_dir(&dir).map_err(|source| SentenceAudioError::Io {
        path: dir.clone(),
        source,
    })?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| SentenceAudioError::Io {
            path: dir.clone(),
            source,
        })?;
        if entry
            .path()
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            paths.push(Path::new(SENTENCE_OUTPUT_DIR).join(entry.file_name()));
        }
    }
    paths.sort();
    Ok(paths)
}

fn write_audio_file<B: TtsBackend>(
    root: &ProjectRoot,
    entry: &AudioEntryPlan,
    backend: &B,
) -> Result<PathBuf, SentenceAudioError> {
    let target = root.join(&entry.audio_path);
    if target.exists() {
        return Ok(entry.audio_path.clone());
    }
    let Some(parent) = target.parent() else {
        return Err(SentenceAudioError::Io {
            path: target,
            source: io::Error::other("missing audio parent directory"),
        });
    };
    fs::create_dir_all(parent).map_err(|source| SentenceAudioError::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    let temp = temp_path_for(&target);
    if let Err(source) = backend.synthesize_hindi(&entry.hindi, &temp) {
        let _ = fs::remove_file(&temp);
        return Err(SentenceAudioError::Tts {
            path: entry.audio_path.clone(),
            source,
        });
    }
    if let Err(source) = fs::rename(&temp, &target) {
        let _ = fs::remove_file(&temp);
        return Err(SentenceAudioError::Io {
            path: target,
            source,
        });
    }
    Ok(entry.audio_path.clone())
}

fn patch_batch_audio(
    root: &ProjectRoot,
    batch: &BatchAudioPlan,
    entries: &[&AudioEntryPlan],
) -> Result<(), SentenceAudioError> {
    let path = root.join(&batch.relative_path);
    let content = fs::read_to_string(&path).map_err(|source| SentenceAudioError::Io {
        path: path.clone(),
        source,
    })?;
    let mut value: Value =
        serde_json::from_str(&content).map_err(|source| SentenceAudioError::Json {
            path: path.clone(),
            source,
        })?;
    let Some(sentences) = value.get_mut("sentences").and_then(Value::as_array_mut) else {
        return Err(SentenceAudioError::UnsupportedBatch(
            batch.relative_path.clone(),
        ));
    };
    for entry in entries {
        let Some(sentence) = sentences
            .get_mut(entry.index - 1)
            .and_then(Value::as_object_mut)
        else {
            return Err(SentenceAudioError::UnsupportedBatch(
                batch.relative_path.clone(),
            ));
        };
        sentence.insert(
            "audio".to_string(),
            Value::String(entry.audio_path.to_string_lossy().to_string()),
        );
    }
    let mut bytes =
        serde_json::to_vec_pretty(&value).map_err(|source| SentenceAudioError::Json {
            path: path.clone(),
            source,
        })?;
    bytes.push(b'\n');
    write_atomic(&path, &bytes)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), SentenceAudioError> {
    let temp = temp_path_for(path);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|source| SentenceAudioError::Io {
            path: temp.clone(),
            source,
        })?;
    if let Err(source) = file.write_all(bytes).and_then(|_| file.sync_all()) {
        let _ = fs::remove_file(&temp);
        return Err(SentenceAudioError::Io { path: temp, source });
    }
    if let Err(source) = fs::rename(&temp, path) {
        let _ = fs::remove_file(&temp);
        return Err(SentenceAudioError::Io {
            path: path.to_path_buf(),
            source,
        });
    }
    Ok(())
}

fn temp_path_for(target: &Path) -> PathBuf {
    let file_name = target
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("audio.tmp");
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    target.with_file_name(format!(".{file_name}.tmp-{}-{nanos}", std::process::id()))
}

fn current_dir() -> Result<PathBuf, SentenceAudioError> {
    std::env::current_dir().map_err(|source| SentenceAudioError::Io {
        path: PathBuf::from("."),
        source,
    })
}

fn slug(value: &str) -> String {
    let mut output = String::new();
    let mut last_was_separator = false;
    for ch in value.chars().flat_map(ascii_chars) {
        if ch.is_ascii_alphanumeric() {
            output.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else if !last_was_separator && !output.is_empty() {
            output.push('_');
            last_was_separator = true;
        }
    }
    while output.ends_with('_') {
        output.pop();
    }
    if output.is_empty() {
        "item".to_string()
    } else {
        output
    }
}

fn ascii_chars(ch: char) -> Vec<char> {
    match ch {
        'ā' | 'á' | 'à' | 'â' | 'ã' | 'ä' | 'å' | 'Ā' => vec!['a'],
        'ī' | 'ĩ' | 'í' | 'ì' | 'î' | 'ï' | 'Ī' | 'Ĩ' => vec!['i'],
        'ū' | 'ú' | 'ù' | 'û' | 'ü' | 'Ū' => vec!['u'],
        'ō' | 'õ' | 'ó' | 'ò' | 'ô' | 'ö' | 'Ō' | 'Õ' => vec!['o'],
        'ṛ' | 'Ṛ' => vec!['r'],
        'ṭ' | 'Ṭ' => vec!['t'],
        'ḍ' | 'Ḍ' => vec!['d'],
        'ṇ' | 'Ṇ' => vec!['n'],
        'ṅ' | 'Ṅ' => vec!['n'],
        'ñ' | 'Ñ' => vec!['n'],
        'ś' | 'ṣ' | 'Ś' | 'Ṣ' => vec!['s'],
        'ḥ' | 'Ḥ' => vec!['h'],
        '\u{0300}'..='\u{036f}' => Vec::new(),
        _ if ch.is_ascii() => vec![ch],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{audio_from, plan_sentence_audio, slug};
    use crate::project::ProjectRoot;
    use crate::tts::{TtsBackend, TtsError};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct FakeTts {
        fail: bool,
    }

    impl TtsBackend for FakeTts {
        fn synthesize_hindi(&self, _text: &str, target: &Path) -> Result<(), TtsError> {
            if self.fail {
                return Err(TtsError::Failed {
                    status: Some(1),
                    stderr: "fake failure".to_string(),
                });
            }
            fs::write(target, b"fake mp3").map_err(TtsError::Io)
        }
    }

    #[test]
    fn slug_is_ascii_safe() {
        assert_eq!(slug("kyā āp Kamalā jī haĩ?"), "kya_ap_kamala_ji_hai");
        assert_eq!(slug("kyõ?"), "kyo");
    }

    #[test]
    fn scanner_plans_missing_audio_and_skips_existing() {
        let root = fixture_root();
        write_batch(&root, true);
        let project = ProjectRoot::discover_from(&root).unwrap();

        let plan = plan_sentence_audio(&project).unwrap();

        assert_eq!(plan.scanned_cards(), 2);
        assert_eq!(plan.skipped_existing(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn existing_audio_field_with_missing_file_regenerates_without_patch() {
        let root = fixture_root();
        write_batch(&root, true);
        fs::remove_file(root.join("audio/sentences/example_batch_01/02_existing.mp3")).unwrap();
        let project = ProjectRoot::discover_from(&root).unwrap();

        let outcome = audio_from(&project, &FakeTts { fail: false }).unwrap();

        assert_eq!(outcome.generated_mp3s.len(), 2);
        assert_eq!(outcome.patched_cards, 1);
        let after =
            fs::read_to_string(root.join("output/sentences/example_batch_01.json")).unwrap();
        assert!(after.contains("audio/sentences/example_batch_01/02_existing.mp3"));
        assert!(root
            .join("audio/sentences/example_batch_01/02_existing.mp3")
            .is_file());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn command_generates_audio_and_patches_json() {
        let root = fixture_root();
        write_batch(&root, true);
        let project = ProjectRoot::discover_from(&root).unwrap();

        let outcome = audio_from(&project, &FakeTts { fail: false }).unwrap();

        assert!(outcome.success);
        assert_eq!(outcome.generated_mp3s.len(), 1);
        assert_eq!(outcome.patched_cards, 1);
        let content =
            fs::read_to_string(root.join("output/sentences/example_batch_01.json")).unwrap();
        assert!(content.contains("\"audio\""));
        assert!(root.join(&outcome.generated_mp3s[0]).is_file());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_tts_leaves_json_unpatched() {
        let root = fixture_root();
        write_batch(&root, false);
        let before =
            fs::read_to_string(root.join("output/sentences/example_batch_01.json")).unwrap();
        let project = ProjectRoot::discover_from(&root).unwrap();

        let error = audio_from(&project, &FakeTts { fail: true }).unwrap_err();

        assert!(error.to_string().contains("fake failure"));
        let after =
            fs::read_to_string(root.join("output/sentences/example_batch_01.json")).unwrap();
        assert_eq!(before, after);
        fs::remove_dir_all(root).unwrap();
    }

    fn fixture_root() -> PathBuf {
        let root = temp_path("hindi-audio");
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::create_dir_all(root.join("input")).unwrap();
        fs::create_dir_all(root.join("output/sentences")).unwrap();
        fs::create_dir_all(root.join("audio")).unwrap();
        fs::write(root.join("docs/DESIGN.md"), "").unwrap();
        fs::write(root.join("docs/ROADMAP.md"), "").unwrap();
        root
    }

    fn write_batch(root: &Path, existing_audio: bool) {
        let second_audio = if existing_audio {
            fs::create_dir_all(root.join("audio/sentences/example_batch_01")).unwrap();
            fs::write(
                root.join("audio/sentences/example_batch_01/02_existing.mp3"),
                b"existing mp3",
            )
            .unwrap();
            r#""audio":"audio/sentences/example_batch_01/02_existing.mp3","#
        } else {
            ""
        };
        fs::write(
            root.join("output/sentences/example_batch_01.json"),
            format!(
                r#"{{
  "title":"Test",
  "subtitle":"Unit",
  "sentences":[
    {{"hindi":"यहाँ","romanisation":"yahā̃","english":"Here."}},
    {{{second_audio}"hindi":"क्या आप कमला जी हैं?","romanisation":"kyā āp Kamalā jī haĩ?","english":"Are you Kamala?"}}
  ]
}}"#
            ),
        )
        .unwrap();
    }

    fn temp_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{label}-{}-{nanos}", std::process::id()))
    }
}
