use crate::project::{ProjectRoot, ProjectRootError};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const RAW_PROMPT_TEMPLATE: &str = include_str!("eval_prompts/sentence_raw_to_yaml.md.hbs");
const RESPONSE_MARKER: &str = "## Paste YAML Response Below";

#[derive(Debug)]
pub enum RawSentenceError {
    Project(ProjectRootError),
    Io { path: PathBuf, source: io::Error },
    Template(handlebars::TemplateError),
    Render(handlebars::RenderError),
    Editor(String),
    Input(String),
    Yaml(serde_yaml::Error),
}

impl std::fmt::Display for RawSentenceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RawSentenceError::Project(error) => write!(formatter, "{error}"),
            RawSentenceError::Io { path, source } => {
                write!(formatter, "{}\n\n{}", source, path.display())
            }
            RawSentenceError::Template(error) => {
                write!(
                    formatter,
                    "Could not register raw sentence prompt.\n\n{error}"
                )
            }
            RawSentenceError::Render(error) => {
                write!(
                    formatter,
                    "Could not render raw sentence prompt.\n\n{error}"
                )
            }
            RawSentenceError::Editor(message) | RawSentenceError::Input(message) => {
                write!(formatter, "{message}")
            }
            RawSentenceError::Yaml(error) => write!(formatter, "Could not parse YAML.\n\n{error}"),
        }
    }
}

impl From<ProjectRootError> for RawSentenceError {
    fn from(error: ProjectRootError) -> Self {
        RawSentenceError::Project(error)
    }
}

impl From<serde_yaml::Error> for RawSentenceError {
    fn from(error: serde_yaml::Error) -> Self {
        RawSentenceError::Yaml(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawSentenceReport {
    pub raw_file: PathBuf,
    pub packet_path: PathBuf,
    pub destination: PathBuf,
    pub title: String,
    pub subtitle: String,
    pub items: usize,
}

impl RawSentenceReport {
    pub fn render(&self) -> String {
        format!(
            "Raw Sentences\n\nSource\n  raw file     {}\n  packet       {}\n\nAccepted YAML\n  destination  {}\n  title        {}\n  subtitle     {}\n  items        {}\n\nNext\n  hindi source ids check\n  hindi sentences plan --max-batches 1",
            self.raw_file.display(),
            self.packet_path.display(),
            self.destination.display(),
            self.title,
            self.subtitle,
            self.items
        )
    }
}

#[derive(Debug, Serialize)]
struct PromptContext {
    raw_file: String,
    raw_text: String,
    suggested_title: String,
    suggested_subtitle: String,
}

#[derive(Debug, Deserialize)]
struct SourceYaml {
    title: String,
    subtitle: String,
    items: Vec<SourceItem>,
}

#[derive(Debug, Deserialize)]
struct SourceItem {
    id: String,
    hindi: String,
    romanisation: String,
    english: String,
}

pub fn raw_from_current_dir(file: Option<&str>) -> Result<RawSentenceReport, RawSentenceError> {
    let root = ProjectRoot::discover_from(current_dir()?)?;
    raw_from(&root, file)
}

pub fn raw_from(
    root: &ProjectRoot,
    file: Option<&str>,
) -> Result<RawSentenceReport, RawSentenceError> {
    let raw_file = resolve_raw_file(root, file)?;
    let raw_text = read_to_string(&raw_file)?;
    let (suggested_title, suggested_subtitle) = suggest_title_subtitle(&raw_file);
    let prompt = render_prompt(PromptContext {
        raw_file: display_path(root, &raw_file),
        raw_text,
        suggested_title,
        suggested_subtitle,
    })?;
    let packet_path = packet_path(root, &raw_file);
    write_file(&packet_path, &packet_content(&prompt))?;

    loop {
        eprintln!("opening raw sentence packet in $EDITOR");
        open_editor(&packet_path)?;
        let packet = read_to_string(&packet_path)?;
        let yaml = extract_yaml_response(&packet);
        match validate_source_yaml(yaml) {
            Ok(document) => {
                let destination = destination_path(root, &document);
                if destination.exists() {
                    return Err(RawSentenceError::Input(format!(
                        "Destination already exists.\n\n{}\n\nMove it aside or change title/subtitle in the YAML.",
                        destination.display()
                    )));
                }
                write_file(&destination, yaml.trim())?;
                return Ok(RawSentenceReport {
                    raw_file,
                    packet_path,
                    destination,
                    title: document.title,
                    subtitle: document.subtitle,
                    items: document.items.len(),
                });
            }
            Err(errors) => {
                eprintln!("YAML needs attention.\n");
                for error in errors {
                    eprintln!("  - {error}");
                }
                eprintln!();
                if !ask_edit_again()? {
                    return Err(RawSentenceError::Input(cancelled_message().to_string()));
                }
            }
        }
    }
}

fn resolve_raw_file(root: &ProjectRoot, file: Option<&str>) -> Result<PathBuf, RawSentenceError> {
    match file {
        Some(file) => {
            let path = Path::new(file);
            Ok(if path.is_absolute() {
                path.to_path_buf()
            } else {
                root.join(path)
            })
        }
        None => select_raw_file(root),
    }
}

fn select_raw_file(root: &ProjectRoot) -> Result<PathBuf, RawSentenceError> {
    let raw_dir = root.join("raw");
    let mut files = Vec::new();
    let entries = fs::read_dir(&raw_dir).map_err(|source| RawSentenceError::Io {
        path: raw_dir.clone(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| RawSentenceError::Io {
            path: raw_dir.clone(),
            source,
        })?;
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        }
    }
    files.sort();
    if files.is_empty() {
        return Err(RawSentenceError::Input(format!(
            "No raw files found under {}.",
            raw_dir.display()
        )));
    }
    if let Some(path) = select_with_fzf(root, &files)? {
        return Ok(path);
    }
    select_with_numbered_menu(root, &files)
}

fn select_with_fzf(
    root: &ProjectRoot,
    files: &[PathBuf],
) -> Result<Option<PathBuf>, RawSentenceError> {
    if Command::new("fzf").arg("--version").output().is_err() {
        return Ok(None);
    }
    let mut child = Command::new("fzf")
        .arg("--prompt")
        .arg("raw> ")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|error| RawSentenceError::Editor(format!("Could not start fzf.\n\n{error}")))?;
    {
        let Some(stdin) = child.stdin.as_mut() else {
            return Ok(None);
        };
        for file in files {
            writeln!(stdin, "{}", display_path(root, file)).map_err(|source| {
                RawSentenceError::Io {
                    path: root.path().to_path_buf(),
                    source,
                }
            })?;
        }
    }
    let output = child
        .wait_with_output()
        .map_err(|error| RawSentenceError::Editor(format!("Could not run fzf.\n\n{error}")))?;
    if !output.status.success() {
        return Err(RawSentenceError::Input(
            "Raw file selection cancelled.".to_string(),
        ));
    }
    let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if selected.is_empty() {
        return Ok(None);
    }
    Ok(Some(root.join(selected)))
}

fn select_with_numbered_menu(
    root: &ProjectRoot,
    files: &[PathBuf],
) -> Result<PathBuf, RawSentenceError> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(RawSentenceError::Input(format!(
            "Raw file is required in non-interactive mode.\n\nRun:\n  hindi sentences raw {}/<file>",
            display_path(root, &root.join("raw"))
        )));
    }
    println!("Raw Files\n");
    for (index, file) in files.iter().enumerate() {
        println!("  {}. {}", index + 1, display_path(root, file));
    }
    print!("\nSelect raw file: ");
    io::stdout()
        .flush()
        .map_err(|source| RawSentenceError::Io {
            path: root.path().to_path_buf(),
            source,
        })?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|source| RawSentenceError::Io {
            path: root.path().to_path_buf(),
            source,
        })?;
    let index = input
        .trim()
        .parse::<usize>()
        .ok()
        .and_then(|value| value.checked_sub(1));
    index
        .and_then(|index| files.get(index).cloned())
        .ok_or_else(|| RawSentenceError::Input("Invalid raw file selection.".to_string()))
}

fn render_prompt(context: PromptContext) -> Result<String, RawSentenceError> {
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);
    handlebars
        .register_template_string("raw", RAW_PROMPT_TEMPLATE)
        .map_err(RawSentenceError::Template)?;
    handlebars
        .render("raw", &context)
        .map_err(RawSentenceError::Render)
}

fn packet_content(prompt: &str) -> String {
    format!(
        "{prompt}\n\n{RESPONSE_MARKER}\n\nReplace this packet with YAML, or paste YAML below this line.\n"
    )
}

fn extract_yaml_response(packet: &str) -> &str {
    match packet.split_once(RESPONSE_MARKER) {
        Some((_, response)) => response
            .trim()
            .strip_prefix("Replace this packet with YAML, or paste YAML below this line.")
            .unwrap_or(response)
            .trim(),
        None => packet.trim(),
    }
}

fn validate_source_yaml(yaml: &str) -> Result<SourceYaml, Vec<String>> {
    if yaml.trim().is_empty() {
        return Err(vec![
            "Paste YAML before saving the editor buffer.".to_string()
        ]);
    }
    let document = serde_yaml::from_str::<SourceYaml>(yaml).map_err(|error| {
        vec![format!(
            "YAML could not be parsed. Fix indentation, quotes, or list structure. {error}"
        )]
    })?;
    let mut errors = Vec::new();
    if document.title.trim().is_empty() {
        errors.push("title is required.".to_string());
    }
    if document.subtitle.trim().is_empty() {
        errors.push("subtitle is required.".to_string());
    }
    if document.items.is_empty() {
        errors.push("items must contain at least one sentence.".to_string());
    }
    let mut ids = BTreeSet::new();
    for (index, item) in document.items.iter().enumerate() {
        let label = format!("items[{}]", index + 1);
        if item.id.trim().is_empty() {
            errors.push(format!("{label}.id is required."));
        } else if !ids.insert(item.id.trim().to_string()) {
            errors.push(format!("duplicate item id {:?}.", item.id));
        }
        if item.hindi.trim().is_empty() {
            errors.push(format!("{label}.hindi is required."));
        }
        if item.romanisation.trim().is_empty() {
            errors.push(format!("{label}.romanisation is required."));
        }
        if item.english.trim().is_empty() {
            errors.push(format!("{label}.english is required."));
        }
    }
    if errors.is_empty() {
        Ok(document)
    } else {
        Err(errors)
    }
}

fn ask_edit_again() -> Result<bool, RawSentenceError> {
    if !io::stdin().is_terminal() {
        return Ok(false);
    }
    loop {
        print!("Edit again? [Y/n] ");
        io::stdout()
            .flush()
            .map_err(|source| RawSentenceError::Io {
                path: PathBuf::from("."),
                source,
            })?;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|source| RawSentenceError::Io {
                path: PathBuf::from("."),
                source,
            })?;
        match input.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" | "" => return Ok(true),
            "n" | "no" | "q" | "quit" => return Ok(false),
            _ => eprintln!("Press Enter or type y to edit again, or n to cancel."),
        }
    }
}

fn cancelled_message() -> &'static str {
    "Raw sentence import cancelled."
}

fn destination_path(root: &ProjectRoot, document: &SourceYaml) -> PathBuf {
    let mut parts = vec![slug(&document.title)];
    let subtitle = slug(&document.subtitle);
    if !subtitle.is_empty() {
        parts.push(subtitle);
    }
    parts.push("sentences".to_string());
    root.join(Path::new("input/sentences").join(format!("{}.yaml", parts.join("_"))))
}

fn packet_path(root: &ProjectRoot, raw_file: &Path) -> PathBuf {
    let stem = raw_file
        .file_stem()
        .and_then(|value| value.to_str())
        .map(slug)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "raw_sentences".to_string());
    root.join(Path::new("runs/raw").join(format!("{stem}_packet.md")))
}

fn suggest_title_subtitle(raw_file: &Path) -> (String, String) {
    let stem = raw_file
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("raw sentences");
    let normalized = stem.replace(['_', '-'], " ");
    let mut words = normalized.split_whitespace().collect::<Vec<_>>();
    if words
        .first()
        .is_some_and(|word| word.eq_ignore_ascii_case("complete"))
        && words
            .get(1)
            .is_some_and(|word| word.eq_ignore_ascii_case("hindi"))
    {
        let title = "Complete Hindi".to_string();
        words.drain(0..2);
        return (title, title_case(&words.join(" ")));
    }
    ("Raw Hindi".to_string(), title_case(&normalized))
}

fn slug(value: &str) -> String {
    let mut slug = String::new();
    let mut last_was_separator = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            last_was_separator = false;
        } else if !last_was_separator && !slug.is_empty() {
            slug.push('_');
            last_was_separator = true;
        }
    }
    while slug.ends_with('_') {
        slug.pop();
    }
    slug
}

fn title_case(value: &str) -> String {
    value
        .split_whitespace()
        .map(|word| {
            let mut characters = word.chars();
            match characters.next() {
                Some(first) => {
                    let mut output = first.to_uppercase().collect::<String>();
                    output.push_str(characters.as_str());
                    output
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn display_path(root: &ProjectRoot, path: &Path) -> String {
    path.strip_prefix(root.path())
        .unwrap_or(path)
        .display()
        .to_string()
}

fn read_to_string(path: &Path) -> Result<String, RawSentenceError> {
    fs::read_to_string(path).map_err(|source| RawSentenceError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn write_file(path: &Path, content: &str) -> Result<(), RawSentenceError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| RawSentenceError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(path, format!("{}\n", content.trim())).map_err(|source| RawSentenceError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn current_dir() -> Result<PathBuf, RawSentenceError> {
    std::env::current_dir().map_err(|source| RawSentenceError::Io {
        path: PathBuf::from("."),
        source,
    })
}

fn open_editor(path: &Path) -> Result<(), RawSentenceError> {
    let editor = std::env::var("EDITOR")
        .map_err(|_| RawSentenceError::Editor("$EDITOR is not set.".to_string()))?;
    let mut parts = editor.split_whitespace();
    let Some(command) = parts.next() else {
        return Err(RawSentenceError::Editor("$EDITOR is not set.".to_string()));
    };
    let status = Command::new(command)
        .args(parts)
        .arg(path)
        .status()
        .map_err(|error| RawSentenceError::Editor(format!("Could not open $EDITOR.\n\n{error}")))?;
    if !status.success() {
        return Err(RawSentenceError::Editor(format!(
            "$EDITOR exited with status {status}."
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        cancelled_message, destination_path, extract_yaml_response, packet_content, slug,
        suggest_title_subtitle, validate_source_yaml,
    };
    use crate::project::ProjectRoot;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn extracts_yaml_from_packet_or_whole_file() {
        let packet = packet_content(
            "Prompt text",
        ) + "\ntitle: Test\nsubtitle: Unit\nitems:\n  - id: \"0001\"\n    hindi: \"यहाँ\"\n    romanisation: \"yahā̃\"\n    english: \"Here.\"\n";

        assert!(extract_yaml_response(&packet).starts_with("title: Test"));
        assert_eq!(
            extract_yaml_response("title: Test\nitems: []"),
            "title: Test\nitems: []"
        );
    }

    #[test]
    fn validates_required_source_yaml_shape() {
        let yaml = r#"
title: Test
subtitle: Unit
items:
  - id: "0001"
    hindi: "यहाँ"
    romanisation: "yahā̃"
    english: "Here."
"#;

        let document = validate_source_yaml(yaml).unwrap();

        assert_eq!(document.title, "Test");
        assert_eq!(document.items.len(), 1);
    }

    #[test]
    fn reports_duplicate_ids() {
        let yaml = r#"
title: Test
subtitle: Unit
items:
  - id: "0001"
    hindi: "यहाँ"
    romanisation: "yahā̃"
    english: "Here."
  - id: "0001"
    hindi: "वहाँ"
    romanisation: "vahā̃"
    english: "There."
"#;

        let errors = validate_source_yaml(yaml).unwrap_err();

        assert!(errors
            .iter()
            .any(|error| error.contains("duplicate item id")));
    }

    #[test]
    fn derives_destination_from_title_and_subtitle() {
        let root = fixture_root();
        let project = ProjectRoot::discover_from(&root).unwrap();
        let document = validate_source_yaml(
            r#"
title: Complete Hindi
subtitle: Chapter 06 Dialog 01
items:
  - id: "0001"
    hindi: "यहाँ"
    romanisation: "yahā̃"
    english: "Here."
"#,
        )
        .unwrap();

        assert_eq!(
            destination_path(&project, &document),
            root.join("input/sentences/complete_hindi_chapter_06_dialog_01_sentences.yaml")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn suggests_complete_hindi_title_from_filename() {
        let (title, subtitle) = suggest_title_subtitle(
            PathBuf::from("raw/complete_hindi_chapter_06_Dialog_01.md").as_path(),
        );

        assert_eq!(title, "Complete Hindi");
        assert_eq!(subtitle, "Chapter 06 Dialog 01");
        assert_eq!(slug("Chapter 06 Dialog 01"), "chapter_06_dialog_01");
    }

    #[test]
    fn cancel_message_is_plain_language() {
        assert_eq!("Raw sentence import cancelled.", cancelled_message());
    }

    fn fixture_root() -> PathBuf {
        let root = temp_path("hindi-raw-root");
        fs::create_dir_all(root.join("input/sentences")).unwrap();
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
