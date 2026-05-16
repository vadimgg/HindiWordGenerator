use crate::project::{ProjectRoot, ProjectRootError};
use crate::sentence_schema::{parse_sentence_batch, SentenceBatch, SentenceCard};
use std::fs::{self, OpenOptions};
use std::io::{self, IsTerminal, Write};
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
        source: Option<String>,
        topic: Option<String>,
    },
    Cancelled,
    EmptySelection,
    Interactive(io::Error),
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
            ExportError::NoMatches { source, topic } => {
                let source = source.as_deref().unwrap_or("any");
                let topic = topic.as_deref().unwrap_or("any");
                write!(
                    formatter,
                    "No accepted sentence cards matched source {source:?} and topic {topic:?}."
                )
            }
            ExportError::Cancelled => write!(formatter, "Export cancelled."),
            ExportError::EmptySelection => write!(formatter, "No export groups selected."),
            ExportError::Interactive(error) => {
                write!(formatter, "Could not read selection.\n\n{error}")
            }
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
    pub groups: Vec<String>,
    pub sentences: usize,
    pub missing_audio: usize,
    pub artifact: PathBuf,
}

impl ExportOutcome {
    pub fn render(&self) -> String {
        let mut output = String::from("Anki Export\n\n");
        output.push_str(&format!("  groups          {}\n", self.groups.len()));
        output.push_str(&format!("  sentences       {}\n", self.sentences));
        output.push_str(&format!("  missing audio   {}\n", self.missing_audio));
        output.push_str(&format!("  artifact        {}\n", self.artifact.display()));
        if !self.groups.is_empty() {
            output.push_str("\nGroups\n");
            for group in &self.groups {
                output.push_str(&format!("  {group}\n"));
            }
        }
        output.push_str(
            "\nNext\n  Import the artifact into Anki or use hindi viewer for interactive export.",
        );
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExportSentence {
    sentence: SentenceCard,
    group_label: String,
}

#[derive(Debug, Clone)]
struct ExportGroup {
    title: String,
    subtitle: String,
    label: String,
    latest_modified: u128,
    sentences: Vec<ExportSentence>,
}

impl ExportGroup {
    fn missing_audio(&self) -> usize {
        self.sentences
            .iter()
            .filter(|entry| audio_sound_tag(&entry.sentence).is_empty())
            .count()
    }
}

pub fn export_from_current_dir(
    source: Option<&str>,
    topic: Option<&str>,
) -> Result<ExportOutcome, ExportError> {
    let root = ProjectRoot::discover_from(current_dir()?)?;
    export_from_filters(&root, source, topic)
}

#[cfg(test)]
pub fn export_from(
    root: &ProjectRoot,
    source: &str,
    topic: &str,
) -> Result<ExportOutcome, ExportError> {
    export_from_filters(root, Some(source), Some(topic))
}

pub fn export_from_filters(
    root: &ProjectRoot,
    source: Option<&str>,
    topic: Option<&str>,
) -> Result<ExportOutcome, ExportError> {
    let groups = load_export_groups(root, source, topic)?;
    if groups.is_empty() {
        return Err(ExportError::NoMatches {
            source: source.map(ToString::to_string),
            topic: topic.map(ToString::to_string),
        });
    }
    let selected = select_groups(groups)?;
    write_groups(root, selected)
}

fn write_groups(
    root: &ProjectRoot,
    groups: Vec<ExportGroup>,
) -> Result<ExportOutcome, ExportError> {
    if groups.is_empty() {
        return Err(ExportError::EmptySelection);
    }
    let mut sentences = Vec::new();
    let mut group_labels = Vec::new();
    let mut missing_audio = 0usize;
    for group in &groups {
        group_labels.push(group.label.clone());
        missing_audio += group.missing_audio();
        sentences.extend(group.sentences.clone());
    }
    let artifact = export_artifact_for(&groups);
    let bytes = build_tsv(&sentences).into_bytes();
    write_export(root, &artifact, &bytes)?;
    Ok(ExportOutcome {
        groups: group_labels,
        sentences: sentences.len(),
        missing_audio,
        artifact,
    })
}

fn export_artifact_for(groups: &[ExportGroup]) -> PathBuf {
    let name = if groups.len() == 1 {
        format!(
            "{}_{}_sentences.tsv",
            slug(&groups[0].title),
            slug(&groups[0].subtitle)
        )
    } else {
        "sentences.tsv".to_string()
    };
    Path::new(EXPORT_DIR).join(name)
}

fn load_export_groups(
    root: &ProjectRoot,
    source: Option<&str>,
    topic: Option<&str>,
) -> Result<Vec<ExportGroup>, ExportError> {
    let mut by_group: std::collections::BTreeMap<(String, String), ExportGroup> =
        std::collections::BTreeMap::new();
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
        let title = batch.title.clone().unwrap_or_default();
        let subtitle = batch.subtitle.clone().unwrap_or_default();
        if source.is_some_and(|source| source != title.as_str())
            || topic.is_some_and(|topic| topic != subtitle.as_str())
        {
            continue;
        }
        let latest_modified = modified_nanos(&path);
        let key = (title.clone(), subtitle.clone());
        let group = by_group.entry(key).or_insert_with(|| ExportGroup {
            label: group_label(Some(&title), Some(&subtitle)),
            title,
            subtitle,
            latest_modified,
            sentences: Vec::new(),
        });
        group.latest_modified = group.latest_modified.max(latest_modified);
        group.sentences.extend(batch_to_export_sentences(batch));
    }
    let mut groups = by_group.into_values().collect::<Vec<_>>();
    groups.sort_by(|left, right| {
        right
            .latest_modified
            .cmp(&left.latest_modified)
            .then_with(|| left.label.cmp(&right.label))
    });
    Ok(groups)
}

fn batch_to_export_sentences(batch: SentenceBatch) -> Vec<ExportSentence> {
    let group_label = group_label(batch.title.as_deref(), batch.subtitle.as_deref());
    batch
        .sentences
        .into_iter()
        .map(|sentence| ExportSentence {
            sentence,
            group_label: group_label.clone(),
        })
        .collect()
}

fn group_label(title: Option<&str>, subtitle: Option<&str>) -> String {
    let label = [title, subtitle]
        .into_iter()
        .flatten()
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    if label.is_empty() {
        "Untitled".to_string()
    } else {
        label
    }
}

fn modified_nanos(path: &Path) -> u128 {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

fn select_groups(groups: Vec<ExportGroup>) -> Result<Vec<ExportGroup>, ExportError> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Ok(groups);
    }

    let mut selected = vec![true; groups.len()];
    let mut page = 0usize;
    loop {
        print_export_menu(&groups, &selected, page)?;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(ExportError::Interactive)?;
        let input = input.trim();
        if input.is_empty() {
            if selected.iter().any(|value| *value) {
                break;
            }
            println!("Select at least one group before exporting.");
            continue;
        }
        match input {
            "q" | "Q" => return Err(ExportError::Cancelled),
            "a" | "A" => selected.fill(true),
            "n" | "N" => selected.fill(false),
            "0" => page = (page + 1) % groups.len().div_ceil(9).max(1),
            value => {
                for ch in value.chars() {
                    if let Some(digit) = ch.to_digit(10) {
                        if (1..=9).contains(&digit) {
                            let index = page * 9 + digit as usize - 1;
                            if index < selected.len() {
                                selected[index] = !selected[index];
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(groups
        .into_iter()
        .enumerate()
        .filter_map(|(index, group)| selected[index].then_some(group))
        .collect())
}

fn print_export_menu(
    groups: &[ExportGroup],
    selected: &[bool],
    page: usize,
) -> Result<(), ExportError> {
    println!("Anki Export\n");
    println!("Select groups to export. Everything is selected by default.");
    println!("Press 1-9 to toggle, 0 for more, a for all, n for none, Enter to export.\n");
    let start = page * 9;
    let end = (start + 9).min(groups.len());
    for (offset, group) in groups[start..end].iter().enumerate() {
        let index = start + offset;
        let marker = if selected[index] { "x" } else { " " };
        println!(
            "  {}. [{}] {}  ({} sentence{}, {} missing audio)",
            offset + 1,
            marker,
            group.label,
            group.sentences.len(),
            plural(group.sentences.len()),
            group.missing_audio()
        );
    }
    if groups.len() > 9 {
        println!("\n  0. show more");
    }
    print!("\nExport selection > ");
    io::stdout().flush().map_err(ExportError::Interactive)
}

fn plural(count: usize) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
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
    use super::{audio_sound_tag, export_from, export_from_filters};
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
        write_sentence_batch(&root, "example.json", "Complete Hindi", "Chapter 02");
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
    fn exports_all_groups_by_default_in_non_interactive_mode() {
        let root = fixture_root();
        write_sentence_batch(&root, "chapter_02.json", "Complete Hindi", "Chapter 02");
        write_sentence_batch(&root, "chapter_03.json", "Complete Hindi", "Chapter 03");
        let project = ProjectRoot::discover_from(&root).unwrap();

        let outcome = export_from_filters(&project, None, None).unwrap();

        assert_eq!(outcome.groups.len(), 2);
        assert_eq!(outcome.sentences, 2);
        assert_eq!(outcome.artifact, PathBuf::from("exports/sentences.tsv"));
        let content = fs::read_to_string(root.join(outcome.artifact)).unwrap();
        assert!(content.contains("Complete Hindi Chapter 02"));
        assert!(content.contains("Complete Hindi Chapter 03"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn no_matching_source_topic_is_error() {
        let root = fixture_root();
        write_sentence_batch(&root, "example.json", "Complete Hindi", "Chapter 02");
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

    fn write_sentence_batch(root: &std::path::Path, file_name: &str, title: &str, subtitle: &str) {
        fs::write(
            root.join("output/sentences").join(file_name),
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
