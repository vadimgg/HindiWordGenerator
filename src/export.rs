use crate::project::{ProjectRoot, ProjectRootError};
use crate::sentence_schema::{parse_sentence_batch, SentenceBatch, SentenceCard};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const SENTENCE_OUTPUT_DIR: &str = "output/sentences";
const EXPORT_DIR: &str = "exports";
const SENTENCE_NOTE_TYPE: &str = "Hindi Sentence";
const SENTENCE_DECK: &str = "Hindi::Sentences";
const SENTENCE_FIELDS: [&str; 9] = [
    "English",
    "Hindi",
    "Audio",
    "Romanisation",
    "Literal",
    "Register",
    "WordBreakdown",
    "Topic",
    "Tags",
];

#[derive(Debug)]
pub enum ExportError {
    Project(ProjectRootError),
    Io {
        path: PathBuf,
        source: io::Error,
    },
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    NoMatches {
        source: String,
        topic: String,
    },
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::Project(error) => write!(formatter, "{error}"),
            ExportError::Io { path, source } => {
                write!(formatter, "Could not access {}\n\n{source}", path.display())
            }
            ExportError::Json { path, source } => {
                write!(formatter, "Could not parse {}\n\n{source}", path.display())
            }
            ExportError::NoMatches { source, topic } => write!(
                formatter,
                "No accepted sentence cards matched source {source:?} and topic {topic:?}."
            ),
        }
    }
}

impl From<ProjectRootError> for ExportError {
    fn from(error: ProjectRootError) -> Self {
        ExportError::Project(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportOutcome {
    pub source: String,
    pub topic: String,
    pub sentences: usize,
    pub missing_audio: usize,
    pub artifact: PathBuf,
}

impl ExportOutcome {
    pub fn render(&self) -> String {
        format!(
            "Anki Export\n\n  source          {}\n  topic           {}\n  sentences       {}\n  missing audio   {}\n  artifact        {}\n\nNext\n  Import the artifact into Anki or use hindi viewer for interactive export.",
            self.source,
            self.topic,
            self.sentences,
            self.missing_audio,
            self.artifact.display()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExportSentence {
    sentence: SentenceCard,
    group_label: String,
}

pub fn export_from_current_dir(source: &str, topic: &str) -> Result<ExportOutcome, ExportError> {
    let root = ProjectRoot::discover_from(current_dir()?)?;
    export_from(&root, source, topic)
}

pub fn export_from(
    root: &ProjectRoot,
    source: &str,
    topic: &str,
) -> Result<ExportOutcome, ExportError> {
    let sentences = load_matching_sentences(root, source, topic)?;
    if sentences.is_empty() {
        return Err(ExportError::NoMatches {
            source: source.to_string(),
            topic: topic.to_string(),
        });
    }
    let missing_audio = sentences
        .iter()
        .filter(|entry| audio_sound_tag(&entry.sentence).is_empty())
        .count();
    let artifact =
        Path::new(EXPORT_DIR).join(format!("{}_{}_sentences.tsv", slug(source), slug(topic)));
    let bytes = build_tsv(&sentences).into_bytes();
    write_export(root, &artifact, &bytes)?;
    Ok(ExportOutcome {
        source: source.to_string(),
        topic: topic.to_string(),
        sentences: sentences.len(),
        missing_audio,
        artifact,
    })
}

fn load_matching_sentences(
    root: &ProjectRoot,
    source: &str,
    topic: &str,
) -> Result<Vec<ExportSentence>, ExportError> {
    let mut matches = Vec::new();
    for relative_path in collect_sentence_batch_paths(root)? {
        let path = root.join(&relative_path);
        let content = fs::read_to_string(&path).map_err(|source| ExportError::Io {
            path: path.clone(),
            source,
        })?;
        let batch = parse_sentence_batch(&content).map_err(|source| ExportError::Json {
            path: path.clone(),
            source,
        })?;
        if batch.title.as_deref() == Some(source) && batch.subtitle.as_deref() == Some(topic) {
            matches.extend(batch_to_export_sentences(batch));
        }
    }
    Ok(matches)
}

fn batch_to_export_sentences(batch: SentenceBatch) -> Vec<ExportSentence> {
    let group_label = [batch.title.as_deref(), batch.subtitle.as_deref()]
        .into_iter()
        .flatten()
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    batch
        .sentences
        .into_iter()
        .map(|sentence| ExportSentence {
            sentence,
            group_label: group_label.clone(),
        })
        .collect()
}

fn collect_sentence_batch_paths(root: &ProjectRoot) -> Result<Vec<PathBuf>, ExportError> {
    let dir = root.join(SENTENCE_OUTPUT_DIR);
    let entries = fs::read_dir(&dir).map_err(|source| ExportError::Io {
        path: dir.clone(),
        source,
    })?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| ExportError::Io {
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

fn build_tsv(sentences: &[ExportSentence]) -> String {
    let mut rows = vec![
        "#separator:tab".to_string(),
        "#html:true".to_string(),
        format!("#notetype:{SENTENCE_NOTE_TYPE}"),
        format!("#deck:{SENTENCE_DECK}"),
        format!("#columns:{}", SENTENCE_FIELDS.join("\t")),
    ];
    for entry in sentences {
        let fields = sentence_fields(entry);
        rows.push(fields.map(|field| sanitize_tsv(&field)).join("\t"));
    }
    rows.join("\n") + "\n"
}

fn sentence_fields(entry: &ExportSentence) -> [String; 9] {
    let sentence = &entry.sentence;
    [
        escape_html(sentence.english.as_deref().unwrap_or_default()),
        escape_html(sentence.hindi.as_deref().unwrap_or_default()),
        audio_sound_tag(sentence),
        escape_html(sentence.romanisation.as_deref().unwrap_or_default()),
        escape_html(sentence.literal.as_deref().unwrap_or_default()),
        escape_html(sentence.register.as_deref().unwrap_or_default()),
        word_breakdown(sentence),
        escape_html(&entry.group_label),
        sentence.anki_tags.join(" "),
    ]
}

fn word_breakdown(sentence: &SentenceCard) -> String {
    if sentence.words.is_empty() {
        return String::new();
    }
    let items = sentence
        .words
        .iter()
        .map(|word| {
            format!(
                "<li><b>{}</b> <i>{}</i> - {}</li>",
                escape_html(word.hindi.as_deref().unwrap_or_default()),
                escape_html(word.roman.as_deref().unwrap_or_default()),
                escape_html(word.meaning.as_deref().unwrap_or_default())
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<ul>{items}</ul>")
}

fn audio_sound_tag(sentence: &SentenceCard) -> String {
    let Some(audio) = sentence.audio.as_deref().and_then(clean_audio_path) else {
        return String::new();
    };
    format!("[sound:{}]", audio_media_filename(audio))
}

fn clean_audio_path(audio: &str) -> Option<&str> {
    let trimmed = audio.trim();
    if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.contains(':') {
        return None;
    }
    let relative = trimmed.strip_prefix('/').unwrap_or(trimmed);
    if !relative.starts_with("audio/")
        || !relative.ends_with(".mp3")
        || relative
            .split('/')
            .any(|part| part.is_empty() || part == "..")
    {
        return None;
    }
    Some(relative)
}

fn audio_media_filename(relative_path: &str) -> String {
    relative_path
        .strip_prefix("audio/")
        .unwrap_or(relative_path)
        .replace('/', "__")
}

fn write_export(root: &ProjectRoot, relative_path: &Path, bytes: &[u8]) -> Result<(), ExportError> {
    let path = root.join(relative_path);
    let Some(parent) = path.parent() else {
        return Err(ExportError::Io {
            path,
            source: io::Error::other("missing export parent"),
        });
    };
    fs::create_dir_all(parent).map_err(|source| ExportError::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    let temp = temp_path_for(&path);
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temp)
        .map_err(|source| ExportError::Io {
            path: temp.clone(),
            source,
        })?;
    if let Err(source) = file.write_all(bytes).and_then(|_| file.sync_all()) {
        let _ = fs::remove_file(&temp);
        return Err(ExportError::Io { path: temp, source });
    }
    fs::rename(&temp, &path).map_err(|source| ExportError::Io { path, source })
}

fn temp_path_for(target: &Path) -> PathBuf {
    let file_name = target
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("export.tsv");
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    target.with_file_name(format!(".{file_name}.tmp-{}-{nanos}", std::process::id()))
}

fn current_dir() -> Result<PathBuf, ExportError> {
    std::env::current_dir().map_err(|source| ExportError::Io {
        path: PathBuf::from("."),
        source,
    })
}

fn sanitize_tsv(value: &str) -> String {
    value.replace('\t', " ").replace(['\n', '\r'], "<br>")
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn slug(value: &str) -> String {
    let mut output = String::new();
    let mut last_separator = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            output.push(ch.to_ascii_lowercase());
            last_separator = false;
        } else if !last_separator && !output.is_empty() {
            output.push('_');
            last_separator = true;
        }
    }
    while output.ends_with('_') {
        output.pop();
    }
    if output.is_empty() {
        "export".to_string()
    } else {
        output
    }
}

#[cfg(test)]
mod tests {
    use super::{audio_sound_tag, export_from};
    use crate::project::ProjectRoot;
    use crate::sentence_schema::SentenceCard;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn converts_audio_path_to_anki_sound_tag() {
        let sentence = SentenceCard {
            hindi: None,
            romanisation: None,
            english: None,
            literal: None,
            register: None,
            source_ref: None,
            tokens: Vec::new(),
            words: Vec::new(),
            anki_tags: Vec::new(),
            audio: Some("audio/sentences/example/01_test.mp3".to_string()),
        };

        assert_eq!(
            audio_sound_tag(&sentence),
            "[sound:sentences__example__01_test.mp3]"
        );
    }

    #[test]
    fn exports_matching_sentence_batch_to_tsv() {
        let root = fixture_root();
        write_sentence_batch(&root, "Complete Hindi", "Chapter 02");
        let project = ProjectRoot::discover_from(&root).unwrap();

        let outcome = export_from(&project, "Complete Hindi", "Chapter 02").unwrap();

        assert_eq!(outcome.sentences, 1);
        assert_eq!(outcome.missing_audio, 0);
        let content = fs::read_to_string(root.join(outcome.artifact)).unwrap();
        assert!(content.contains("#notetype:Hindi Sentence"));
        assert!(content.contains("[sound:sentences__example__01_test.mp3]"));
        assert!(content.contains("Complete Hindi Chapter 02"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn no_matching_source_topic_is_error() {
        let root = fixture_root();
        write_sentence_batch(&root, "Complete Hindi", "Chapter 02");
        let project = ProjectRoot::discover_from(&root).unwrap();

        let error = export_from(&project, "Complete Hindi", "Chapter 03").unwrap_err();

        assert!(error.to_string().contains("No accepted sentence cards"));
        fs::remove_dir_all(root).unwrap();
    }

    fn fixture_root() -> PathBuf {
        let root = temp_path("hindi-export");
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::create_dir_all(root.join("input")).unwrap();
        fs::create_dir_all(root.join("output/sentences")).unwrap();
        fs::write(root.join("docs/DESIGN.md"), "").unwrap();
        fs::write(root.join("docs/ROADMAP.md"), "").unwrap();
        root
    }

    fn write_sentence_batch(root: &std::path::Path, title: &str, subtitle: &str) {
        fs::write(
            root.join("output/sentences/example.json"),
            format!(
                r#"{{
  "title": "{title}",
  "subtitle": "{subtitle}",
  "sentences": [{{
    "hindi": "यहाँ",
    "romanisation": "yahā̃",
    "english": "Here.",
    "literal": "here",
    "register": "standard",
    "audio": "audio/sentences/example/01_test.mp3",
    "words": [{{"id":"w1","hindi":"यहाँ","roman":"yahā̃","meaning":"here"}}],
    "tokens": [{{"hindi":"यहाँ","roman":"yahā̃","kind":"word","word_id":"w1"}}],
    "anki_tags": ["chapter-02"]
  }}]
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
