use crate::project::{ProjectRoot, ProjectRootError};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const SENTENCE_SOURCE_DIR: &str = "input/sentences";
const WORD_SOURCE_DIR: &str = "input/words";

#[derive(Debug)]
pub enum SourceIdError {
    Project(ProjectRootError),
    Io { path: PathBuf, source: io::Error },
}

impl std::fmt::Display for SourceIdError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceIdError::Project(error) => write!(formatter, "{error}"),
            SourceIdError::Io { path, source } => {
                write!(
                    formatter,
                    "Could not read or write {}\n\n{source}",
                    path.display()
                )
            }
        }
    }
}

impl From<ProjectRootError> for SourceIdError {
    fn from(error: ProjectRootError) -> Self {
        SourceIdError::Project(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceIdReport {
    files: Vec<FileReport>,
    errors: Vec<SourceIdProblem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceDocument {
    relative_path: PathBuf,
    original: String,
    items: Vec<SourceItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceItem {
    start_line: usize,
    end_line: usize,
    id: Option<ParsedId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedId {
    value: String,
    quoted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileReport {
    relative_path: PathBuf,
    items: usize,
    missing: usize,
    added: usize,
    changed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceIdProblem {
    relative_path: PathBuf,
    line: Option<usize>,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    source: SourceIdReport,
    dry_run: bool,
    writes: Vec<FileWrite>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileWrite {
    relative_path: PathBuf,
    ids_added: usize,
}

pub fn check_from_current_dir() -> Result<SourceIdReport, SourceIdError> {
    let root = ProjectRoot::discover_from(current_dir()?)?;
    check(&root)
}

pub fn migrate_from_current_dir(dry_run: bool) -> Result<MigrationReport, SourceIdError> {
    let root = ProjectRoot::discover_from(current_dir()?)?;
    migrate(&root, dry_run)
}

fn current_dir() -> Result<PathBuf, SourceIdError> {
    std::env::current_dir().map_err(|source| SourceIdError::Io {
        path: PathBuf::from("."),
        source,
    })
}

fn check(root: &ProjectRoot) -> Result<SourceIdReport, SourceIdError> {
    let documents = load_documents(root)?;
    Ok(report_for_documents(&documents))
}

fn migrate(root: &ProjectRoot, dry_run: bool) -> Result<MigrationReport, SourceIdError> {
    let documents = load_documents(root)?;
    let source = report_for_documents(&documents);
    if source.has_errors() {
        return Ok(MigrationReport {
            source,
            dry_run,
            writes: Vec::new(),
        });
    }

    let mut writes = Vec::new();
    let mut rendered = Vec::new();
    for document in &documents {
        let migrated = migrate_document(document);
        if migrated.ids_added > 0 {
            writes.push(FileWrite {
                relative_path: document.relative_path.clone(),
                ids_added: migrated.ids_added,
            });
            rendered.push((document.relative_path.clone(), migrated.content));
        }
    }

    if !dry_run {
        for (relative_path, content) in rendered {
            let path = root.join(&relative_path);
            fs::write(&path, content).map_err(|source| SourceIdError::Io { path, source })?;
        }
    }

    Ok(MigrationReport {
        source,
        dry_run,
        writes,
    })
}

impl SourceIdReport {
    pub fn is_complete(&self) -> bool {
        !self.has_errors() && self.missing_count() == 0
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    fn file_count(&self) -> usize {
        self.files.len()
    }

    fn item_count(&self) -> usize {
        self.files.iter().map(|file| file.items).sum()
    }

    fn missing_count(&self) -> usize {
        self.files.iter().map(|file| file.missing).sum()
    }

    fn duplicate_count(&self) -> usize {
        self.errors
            .iter()
            .filter(|problem| problem.message.starts_with("Duplicate source id"))
            .count()
    }

    fn malformed_count(&self) -> usize {
        self.errors
            .iter()
            .filter(|problem| problem.message.starts_with("Malformed source id"))
            .count()
    }

    pub fn render_check(&self) -> String {
        let mut output = String::from("Source IDs\n\nScope\n");
        output.push_str(&format!("  sentences  {SENTENCE_SOURCE_DIR}/*.yaml\n"));
        output.push_str(&format!("  words      {WORD_SOURCE_DIR}/*.yaml\n\n"));
        output.push_str("Result\n");
        output.push_str(&format!("  files      {}\n", self.file_count()));
        output.push_str(&format!("  items      {}\n", self.item_count()));
        output.push_str(&format!("  missing    {}\n", self.missing_count()));
        output.push_str(&format!("  duplicate  {}\n", self.duplicate_count()));
        output.push_str(&format!("  malformed  {}\n", self.malformed_count()));

        if self.has_errors() {
            output.push_str("\nProblems\n");
            append_problems(&mut output, &self.errors);
            output
                .push_str("\nNext\n  Fix source YAML IDs, then run: cargo run -- source ids check");
        } else if self.missing_count() > 0 {
            output.push_str("\nNext\n  cargo run -- source ids migrate");
        } else {
            output.push_str("\nReady\n  Source YAML has stable item IDs.");
        }

        output
    }
}

impl MigrationReport {
    pub fn has_errors(&self) -> bool {
        self.source.has_errors()
    }

    pub fn render_migration(&self) -> String {
        let mut output = if self.dry_run {
            String::from("Source ID Migration Preview\n\n")
        } else {
            String::from("Source ID Migration\n\n")
        };

        if self.has_errors() {
            output.push_str("Problems\n");
            append_problems(&mut output, &self.source.errors);
            output.push_str("\nNext\n  Fix source YAML IDs, then rerun migration.");
            return output;
        }

        if self.writes.is_empty() {
            output.push_str("Result\n  files changed  0\n  ids added      0\n\n");
            output.push_str("Ready\n  Source YAML already has stable item IDs.");
            return output;
        }

        output.push_str(if self.dry_run {
            "Planned Files\n"
        } else {
            "Changed Files\n"
        });
        for write in &self.writes {
            output.push_str(&format!(
                "  {:<60} added {} ids\n",
                write.relative_path.display(),
                write.ids_added
            ));
        }

        output.push_str("\nResult\n");
        output.push_str(&format!("  files changed  {}\n", self.writes.len()));
        output.push_str(&format!(
            "  ids added      {}\n",
            self.writes
                .iter()
                .map(|write| write.ids_added)
                .sum::<usize>()
        ));
        output.push_str("\nNext\n  cargo run -- source ids check");

        output
    }
}

fn append_problems(output: &mut String, problems: &[SourceIdProblem]) {
    for problem in problems {
        match problem.line {
            Some(line) => output.push_str(&format!(
                "  {}:{}  {}\n",
                problem.relative_path.display(),
                line,
                problem.message
            )),
            None => output.push_str(&format!(
                "  {}  {}\n",
                problem.relative_path.display(),
                problem.message
            )),
        }
    }
}

fn load_documents(root: &ProjectRoot) -> Result<Vec<SourceDocument>, SourceIdError> {
    let mut paths = Vec::new();
    collect_yaml_paths(root, SENTENCE_SOURCE_DIR, &mut paths)?;
    collect_yaml_paths(root, WORD_SOURCE_DIR, &mut paths)?;

    let mut documents = Vec::new();
    for relative_path in paths {
        let path = root.join(&relative_path);
        let original =
            fs::read_to_string(&path).map_err(|source| SourceIdError::Io { path, source })?;
        let items = parse_items(&relative_path, &original);
        documents.push(SourceDocument {
            relative_path,
            original,
            items,
        });
    }

    Ok(documents)
}

fn collect_yaml_paths(
    root: &ProjectRoot,
    relative_dir: &str,
    paths: &mut Vec<PathBuf>,
) -> Result<(), SourceIdError> {
    let dir = root.join(relative_dir);
    let entries = fs::read_dir(&dir).map_err(|source| SourceIdError::Io {
        path: dir.clone(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| SourceIdError::Io {
            path: dir.clone(),
            source,
        })?;
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "yaml")
        {
            paths.push(Path::new(relative_dir).join(entry.file_name()));
        }
    }

    paths.sort();
    Ok(())
}

fn report_for_documents(documents: &[SourceDocument]) -> SourceIdReport {
    let mut files = Vec::new();
    let mut errors = Vec::new();

    for document in documents {
        let mut missing = 0;
        let mut seen: BTreeMap<String, usize> = BTreeMap::new();
        for item in &document.items {
            match &item.id {
                Some(id) => {
                    if !id.quoted || !is_valid_id(&id.value) {
                        errors.push(SourceIdProblem {
                            relative_path: document.relative_path.clone(),
                            line: Some(item.start_line + 1),
                            message: format!(
                                "Malformed source id {:?}; expected a quoted zero-padded numeric string like \"0001\".",
                                id.value
                            ),
                        });
                    }
                    if let Some(first_line) = seen.insert(id.value.clone(), item.start_line + 1) {
                        errors.push(SourceIdProblem {
                            relative_path: document.relative_path.clone(),
                            line: Some(item.start_line + 1),
                            message: format!(
                                "Duplicate source id {:?}; first seen on line {first_line}.",
                                id.value
                            ),
                        });
                    }
                }
                None => missing += 1,
            }
        }

        files.push(FileReport {
            relative_path: document.relative_path.clone(),
            items: document.items.len(),
            missing,
            added: 0,
            changed: false,
        });
    }

    SourceIdReport { files, errors }
}

fn parse_items(_relative_path: &Path, content: &str) -> Vec<SourceItem> {
    let lines: Vec<&str> = content.lines().collect();
    let mut starts = Vec::new();
    let mut in_items = false;

    for (index, line) in lines.iter().enumerate() {
        if *line == "items:" {
            in_items = true;
            continue;
        }
        if in_items && line.starts_with("  - ") {
            starts.push(index);
        }
    }

    let mut items = Vec::new();
    for (position, start_line) in starts.iter().enumerate() {
        let end_line = starts.get(position + 1).copied().unwrap_or(lines.len());
        let id = find_id(&lines[*start_line..end_line]);
        items.push(SourceItem {
            start_line: *start_line,
            end_line,
            id,
        });
    }

    items
}

fn find_id(block: &[&str]) -> Option<ParsedId> {
    let first = block.first()?;
    if let Some(value) = first.strip_prefix("  - id: ") {
        return Some(parse_id_value(value.trim()));
    }

    for line in block.iter().skip(1) {
        if let Some(value) = line.strip_prefix("    id: ") {
            return Some(parse_id_value(value.trim()));
        }
    }

    None
}

fn parse_id_value(value: &str) -> ParsedId {
    let quoted = value.starts_with('"') && value.ends_with('"') && value.len() >= 2;
    let value = if quoted {
        value
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .unwrap_or(value)
    } else {
        value
    };

    ParsedId {
        value: value.to_string(),
        quoted,
    }
}

fn is_valid_id(id: &str) -> bool {
    id.len() == 4 && id.bytes().all(|byte| byte.is_ascii_digit())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MigratedDocument {
    content: String,
    ids_added: usize,
}

fn migrate_document(document: &SourceDocument) -> MigratedDocument {
    let mut existing: BTreeSet<String> = document
        .items
        .iter()
        .filter_map(|item| item.id.as_ref().map(|id| id.value.clone()))
        .collect();
    let mut next = 1usize;
    let mut item_ids = BTreeMap::new();
    for item in &document.items {
        if item.id.is_none() {
            let id = next_available_id(&existing, &mut next);
            existing.insert(id.clone());
            item_ids.insert(item.start_line, id);
        }
    }

    if item_ids.is_empty() {
        return MigratedDocument {
            content: document.original.clone(),
            ids_added: 0,
        };
    }

    let mut output = Vec::new();
    for (index, line) in document.original.lines().enumerate() {
        if let Some(id) = item_ids.get(&index) {
            if let Some(rest) = line.strip_prefix("  - ") {
                output.push(format!("  - id: \"{id}\""));
                output.push(format!("    {rest}"));
            } else {
                output.push(line.to_string());
            }
        } else {
            output.push(line.to_string());
        }
    }

    let mut content = output.join("\n");
    if document.original.ends_with('\n') {
        content.push('\n');
    }

    MigratedDocument {
        content,
        ids_added: item_ids.len(),
    }
}

fn next_available_id(existing: &BTreeSet<String>, next: &mut usize) -> String {
    loop {
        let candidate = format!("{:04}", *next);
        *next += 1;
        if !existing.contains(&candidate) {
            return candidate;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_valid_id, migrate_document, parse_items, report_for_documents, SourceDocument};
    use std::path::PathBuf;

    #[test]
    fn allocates_ids_for_missing_items_in_order() {
        let document = document(
            "title: Test\nitems:\n  - hindi: \"एक\"\n    romanisation: \"ek\"\n    english: \"one\"\n  - hindi: \"दो\"\n    romanisation: \"do\"\n    english: \"two\"\n",
        );

        let migrated = migrate_document(&document);

        assert!(migrated.content.contains("  - id: \"0001\"\n    hindi:"));
        assert!(migrated.content.contains("  - id: \"0002\"\n    hindi:"));
        assert_eq!(migrated.ids_added, 2);
    }

    #[test]
    fn preserves_existing_ids_and_fills_gaps() {
        let document = document(
            "title: Test\nitems:\n  - id: \"0002\"\n    hindi: \"दो\"\n    romanisation: \"do\"\n    english: \"two\"\n  - hindi: \"तीन\"\n    romanisation: \"tīn\"\n    english: \"three\"\n",
        );

        let migrated = migrate_document(&document);

        assert!(migrated.content.contains("  - id: \"0002\"\n"));
        assert!(migrated.content.contains("  - id: \"0001\"\n    hindi:"));
        assert_eq!(migrated.ids_added, 1);
    }

    #[test]
    fn migration_is_idempotent() {
        let document = document(
            "title: Test\nitems:\n  - id: \"0001\"\n    hindi: \"एक\"\n    romanisation: \"ek\"\n    english: \"one\"\n",
        );

        let migrated = migrate_document(&document);

        assert_eq!(migrated.content, document.original);
        assert_eq!(migrated.ids_added, 0);
    }

    #[test]
    fn detects_duplicate_ids_within_one_file() {
        let document = document(
            "title: Test\nitems:\n  - id: \"0001\"\n    hindi: \"एक\"\n  - id: \"0001\"\n    hindi: \"दो\"\n",
        );

        let report = report_for_documents(&[document]);

        assert!(report.has_errors());
        assert_eq!(report.duplicate_count(), 1);
    }

    #[test]
    fn detects_malformed_ids() {
        let document = document("title: Test\nitems:\n  - id: \"chapter-1\"\n    hindi: \"एक\"\n");

        let report = report_for_documents(&[document]);

        assert!(report.has_errors());
        assert_eq!(report.malformed_count(), 1);
    }

    #[test]
    fn detects_unquoted_ids() {
        let document = document("title: Test\nitems:\n  - id: 0001\n    hindi: \"एक\"\n");

        let report = report_for_documents(&[document]);

        assert!(report.has_errors());
        assert_eq!(report.malformed_count(), 1);
    }

    #[test]
    fn validates_id_shape() {
        assert!(is_valid_id("0001"));
        assert!(!is_valid_id("1"));
        assert!(!is_valid_id("chapter-1"));
    }

    fn document(content: &str) -> SourceDocument {
        SourceDocument {
            relative_path: PathBuf::from("input/sentences/example.yaml"),
            original: content.to_string(),
            items: parse_items(
                PathBuf::from("input/sentences/example.yaml").as_path(),
                content,
            ),
        }
    }
}
