use crate::project::{ProjectRoot, ProjectRootError};
use crate::source_identity::source_fingerprint;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const SENTENCE_SOURCE_DIR: &str = "input/sentences";
const SENTENCE_OUTPUT_DIR: &str = "output/sentences";
const DEFAULT_BATCH_SIZE: usize = 5;

#[derive(Debug)]
pub enum SentencePlanError {
    Project(ProjectRootError),
    Io { path: PathBuf, source: io::Error },
}

impl std::fmt::Display for SentencePlanError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SentencePlanError::Project(error) => write!(formatter, "{error}"),
            SentencePlanError::Io { path, source } => {
                write!(formatter, "Could not read {}\n\n{source}", path.display())
            }
        }
    }
}

impl From<ProjectRootError> for SentencePlanError {
    fn from(error: ProjectRootError) -> Self {
        SentencePlanError::Project(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentencePlan {
    source_files: usize,
    source_items: usize,
    valid_ids: usize,
    output_files: usize,
    accepted_cards: usize,
    done: usize,
    missing_lineage: usize,
    source_changed: usize,
    pending_items: usize,
    planned_items: usize,
    deferred_items: usize,
    max_batches: usize,
    batch_size: usize,
    planned_files: Vec<PathBuf>,
    errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceFile {
    relative_path: PathBuf,
    stem: String,
    items: Vec<SourceRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceRow {
    id: String,
    hindi: String,
    romanisation: String,
    english: String,
    fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AcceptedCard {
    source_ref: Option<SourceRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceRef {
    file: String,
    item_id: String,
    fingerprint: String,
}

pub fn plan_from_current_dir(max_batches: usize) -> Result<SentencePlan, SentencePlanError> {
    let root = ProjectRoot::discover_from(current_dir()?)?;
    plan(&root, max_batches)
}

fn current_dir() -> Result<PathBuf, SentencePlanError> {
    std::env::current_dir().map_err(|source| SentencePlanError::Io {
        path: PathBuf::from("."),
        source,
    })
}

fn plan(root: &ProjectRoot, max_batches: usize) -> Result<SentencePlan, SentencePlanError> {
    let source_files = load_source_files(root)?;
    let output_paths = collect_json_paths(root)?;
    let accepted_cards = load_accepted_cards(root, &output_paths)?;
    Ok(build_plan(
        source_files,
        output_paths,
        accepted_cards,
        max_batches,
    ))
}

impl SentencePlan {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn render(&self) -> String {
        let mut output = String::from("Sentence Plan\n\n");

        output.push_str("Sources\n");
        output.push_str(&format!("  files              {}\n", self.source_files));
        output.push_str(&format!("  source items       {}\n", self.source_items));
        output.push_str(&format!("  valid ids          {}\n\n", self.valid_ids));

        output.push_str("Accepted Output\n");
        output.push_str(&format!("  batch files        {}\n", self.output_files));
        output.push_str(&format!("  accepted cards     {}\n", self.accepted_cards));
        output.push_str(&format!("  done               {}\n", self.done));
        output.push_str(&format!("  missing lineage    {}\n", self.missing_lineage));
        output.push_str(&format!("  source changed     {}\n\n", self.source_changed));

        output.push_str("Plan\n");
        output.push_str(&format!("  max batches        {}\n", self.max_batches));
        output.push_str(&format!("  batch size         {}\n", self.batch_size));
        output.push_str(&format!(
            "  planned batches    {}\n",
            self.planned_files.len()
        ));
        output.push_str(&format!("  planned items      {}\n", self.planned_items));
        output.push_str(&format!("  pending items      {}\n", self.pending_items));
        output.push_str(&format!("  deferred items     {}\n", self.deferred_items));

        if !self.planned_files.is_empty() {
            output.push_str("\nPlanned Files\n");
            for file in &self.planned_files {
                output.push_str(&format!("  {}\n", file.display()));
            }
        }

        if self.has_errors() {
            output.push_str("\nProblems\n");
            for error in &self.errors {
                output.push_str(&format!("  {error}\n"));
            }
            output.push_str("\nNext\n  Fix source/output issues, then rerun the planner.");
        } else {
            output.push_str("\nNext\n  M4 adds: hindi sentences generate --max-batches 1");
        }

        output
    }
}

fn build_plan(
    source_files: Vec<SourceFile>,
    output_paths: Vec<PathBuf>,
    accepted_cards: Vec<AcceptedCard>,
    max_batches: usize,
) -> SentencePlan {
    let mut errors = Vec::new();
    let mut source_index = BTreeMap::new();
    for source_file in &source_files {
        let mut ids = BTreeSet::new();
        for row in &source_file.items {
            if row.id.len() != 4 || !row.id.bytes().all(|byte| byte.is_ascii_digit()) {
                errors.push(format!(
                    "Malformed source id {:?} in {}.",
                    row.id,
                    source_file.relative_path.display()
                ));
            }
            if !ids.insert(row.id.clone()) {
                errors.push(format!(
                    "Duplicate source id {:?} in {}.",
                    row.id,
                    source_file.relative_path.display()
                ));
            }
            source_index.insert(
                (
                    source_file.relative_path.to_string_lossy().to_string(),
                    row.id.clone(),
                ),
                row.fingerprint.clone(),
            );
        }
    }

    let mut done_keys = BTreeSet::new();
    let mut done = 0;
    let mut missing_lineage = 0;
    let mut source_changed = 0;
    for card in &accepted_cards {
        match &card.source_ref {
            Some(source_ref) => {
                let key = (source_ref.file.clone(), source_ref.item_id.clone());
                match source_index.get(&key) {
                    Some(current) if current == &source_ref.fingerprint => {
                        done += 1;
                        done_keys.insert(key);
                    }
                    Some(_) => source_changed += 1,
                    None => source_changed += 1,
                }
            }
            None => missing_lineage += 1,
        }
    }

    let pending_by_file: Vec<(&SourceFile, usize)> = source_files
        .iter()
        .map(|source_file| {
            let pending = source_file
                .items
                .iter()
                .filter(|row| {
                    !done_keys.contains(&(
                        source_file.relative_path.to_string_lossy().to_string(),
                        row.id.clone(),
                    ))
                })
                .count();
            (source_file, pending)
        })
        .collect();
    let mut remaining_batches = max_batches;
    let mut planned_files = Vec::new();
    let mut planned_items = 0usize;
    for (source_file, pending) in pending_by_file {
        if pending == 0 || remaining_batches == 0 {
            continue;
        }
        let needed_batches = pending.div_ceil(DEFAULT_BATCH_SIZE);
        let batches_for_file = needed_batches.min(remaining_batches);
        planned_items += pending.min(batches_for_file * DEFAULT_BATCH_SIZE);
        for _ in 0..batches_for_file {
            planned_files.push(next_batch_path(
                &source_file.stem,
                &output_paths,
                planned_files_for_stem(&planned_files, &source_file.stem),
            ));
        }
        remaining_batches -= batches_for_file;
    }
    let pending_items = source_files
        .iter()
        .map(|source_file| {
            source_file
                .items
                .iter()
                .filter(|row| {
                    !done_keys.contains(&(
                        source_file.relative_path.to_string_lossy().to_string(),
                        row.id.clone(),
                    ))
                })
                .count()
        })
        .sum::<usize>();
    let deferred_items = pending_items.saturating_sub(planned_items);

    SentencePlan {
        source_files: source_files.len(),
        source_items: source_files
            .iter()
            .map(|source_file| source_file.items.len())
            .sum(),
        valid_ids: source_files
            .iter()
            .map(|source_file| source_file.items.len())
            .sum::<usize>()
            .saturating_sub(errors.len()),
        output_files: output_paths.len(),
        accepted_cards: accepted_cards.len(),
        done,
        missing_lineage,
        source_changed,
        pending_items,
        planned_items,
        deferred_items,
        max_batches,
        batch_size: DEFAULT_BATCH_SIZE,
        planned_files,
        errors,
    }
}

fn next_batch_path(stem: &str, existing: &[PathBuf], offset: usize) -> PathBuf {
    let mut existing_numbers = BTreeSet::new();
    let prefix = format!("{stem}_batch_");
    for path in existing {
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let Some(number) = file_name
            .strip_prefix(&prefix)
            .and_then(|value| value.strip_suffix(".json"))
            .and_then(|value| value.parse::<usize>().ok())
        else {
            continue;
        };
        existing_numbers.insert(number);
    }

    let mut next = 1usize;
    let mut skipped = 0usize;
    loop {
        if !existing_numbers.contains(&next) {
            if skipped == offset {
                return PathBuf::from(SENTENCE_OUTPUT_DIR)
                    .join(format!("{stem}_batch_{next:02}.json"));
            }
            skipped += 1;
        }
        next += 1;
    }
}

fn planned_files_for_stem(planned_files: &[PathBuf], stem: &str) -> usize {
    let prefix = format!("{stem}_batch_");
    planned_files
        .iter()
        .filter(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|file_name| file_name.starts_with(&prefix))
        })
        .count()
}

fn load_source_files(root: &ProjectRoot) -> Result<Vec<SourceFile>, SentencePlanError> {
    let mut paths = Vec::new();
    let dir = root.join(SENTENCE_SOURCE_DIR);
    let entries = fs::read_dir(&dir).map_err(|source| SentencePlanError::Io {
        path: dir.clone(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| SentencePlanError::Io {
            path: dir.clone(),
            source,
        })?;
        if entry
            .path()
            .extension()
            .is_some_and(|extension| extension == "yaml")
        {
            paths.push(Path::new(SENTENCE_SOURCE_DIR).join(entry.file_name()));
        }
    }
    paths.sort();

    let mut files = Vec::new();
    for relative_path in paths {
        let path = root.join(&relative_path);
        let content =
            fs::read_to_string(&path).map_err(|source| SentencePlanError::Io { path, source })?;
        let stem = relative_path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string();
        files.push(SourceFile {
            relative_path,
            stem,
            items: parse_source_rows(&content),
        });
    }
    Ok(files)
}

fn parse_source_rows(content: &str) -> Vec<SourceRow> {
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

    let mut rows = Vec::new();
    for (position, start) in starts.iter().enumerate() {
        let end = starts.get(position + 1).copied().unwrap_or(lines.len());
        let block = &lines[*start..end];
        let id = field(block, "id").unwrap_or_default();
        let hindi = field(block, "hindi").unwrap_or_default();
        let romanisation = field(block, "romanisation").unwrap_or_default();
        let english = field(block, "english").unwrap_or_default();
        let fingerprint = source_fingerprint(&hindi, &romanisation, &english);
        rows.push(SourceRow {
            id,
            hindi,
            romanisation,
            english,
            fingerprint,
        });
    }
    rows
}

fn field(block: &[&str], name: &str) -> Option<String> {
    let inline_prefix = format!("  - {name}: ");
    let field_prefix = format!("    {name}: ");
    for (index, line) in block.iter().enumerate() {
        if index == 0 {
            if let Some(value) = line.strip_prefix(&inline_prefix) {
                return Some(unquote(value.trim()).to_string());
            }
        }
        if let Some(value) = line.strip_prefix(&field_prefix) {
            return Some(unquote(value.trim()).to_string());
        }
    }
    None
}

fn unquote(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
}

fn collect_json_paths(root: &ProjectRoot) -> Result<Vec<PathBuf>, SentencePlanError> {
    let dir = root.join(SENTENCE_OUTPUT_DIR);
    let entries = fs::read_dir(&dir).map_err(|source| SentencePlanError::Io {
        path: dir.clone(),
        source,
    })?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| SentencePlanError::Io {
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

fn load_accepted_cards(
    root: &ProjectRoot,
    paths: &[PathBuf],
) -> Result<Vec<AcceptedCard>, SentencePlanError> {
    let mut cards = Vec::new();
    for relative_path in paths {
        let path = root.join(relative_path);
        let content =
            fs::read_to_string(&path).map_err(|source| SentencePlanError::Io { path, source })?;
        cards.extend(parse_accepted_cards(&content));
    }
    Ok(cards)
}

fn parse_accepted_cards(content: &str) -> Vec<AcceptedCard> {
    let Some(array_start) = content.find("\"sentences\"") else {
        return Vec::new();
    };
    let Some(bracket_start) = content[array_start..]
        .find('[')
        .map(|index| array_start + index)
    else {
        return Vec::new();
    };
    let Some(bracket_end) = matching_bracket(content, bracket_start, '[', ']') else {
        return Vec::new();
    };
    let array = &content[bracket_start + 1..bracket_end];

    top_level_objects(array)
        .into_iter()
        .map(|object| AcceptedCard {
            source_ref: parse_source_ref(object),
        })
        .collect()
}

fn parse_source_ref(object: &str) -> Option<SourceRef> {
    let source_ref_index = object.find("\"source_ref\"")?;
    let brace_start = object[source_ref_index..]
        .find('{')
        .map(|index| source_ref_index + index)?;
    let brace_end = matching_bracket(object, brace_start, '{', '}')?;
    let source_ref = &object[brace_start + 1..brace_end];
    Some(SourceRef {
        file: json_string_field(source_ref, "file")?,
        item_id: json_string_field(source_ref, "item_id")?,
        fingerprint: json_string_field(source_ref, "fingerprint")?,
    })
}

fn json_string_field(object: &str, name: &str) -> Option<String> {
    let pattern = format!("\"{name}\"");
    let field_index = object.find(&pattern)?;
    let colon_index =
        object[field_index + pattern.len()..].find(':')? + field_index + pattern.len();
    let after_colon = &object[colon_index + 1..];
    let quote_start = after_colon.find('"')?;
    let value_start = quote_start + 1;
    let value_end = find_string_end(after_colon, value_start)?;
    Some(unescape_json_string(&after_colon[value_start..value_end]))
}

fn top_level_objects(array: &str) -> Vec<&str> {
    let mut objects = Vec::new();
    let mut start = None;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (index, byte) in array.bytes().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'{' => {
                if depth == 0 {
                    start = Some(index);
                }
                depth += 1;
            }
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    if let Some(start_index) = start.take() {
                        objects.push(&array[start_index..=index]);
                    }
                }
            }
            _ => {}
        }
    }
    objects
}

fn matching_bracket(content: &str, start: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, ch) in content[start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if ch == '"' {
            in_string = true;
        } else if ch == open {
            depth += 1;
        } else if ch == close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(start + offset);
            }
        }
    }
    None
}

fn find_string_end(content: &str, start: usize) -> Option<usize> {
    let mut escaped = false;
    for (offset, byte) in content[start..].bytes().enumerate() {
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'"' {
            return Some(start + offset);
        }
    }
    None
}

fn unescape_json_string(value: &str) -> String {
    value
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
        .replace("\\n", "\n")
}

#[cfg(test)]
mod tests {
    use super::{
        build_plan, next_batch_path, parse_accepted_cards, parse_source_rows, AcceptedCard,
        SourceFile, SourceRef,
    };
    use crate::source_identity::source_fingerprint;
    use std::path::PathBuf;

    #[test]
    fn computes_source_fingerprint_with_normalized_whitespace() {
        let a = source_fingerprint(" यहाँ ", "yahā̃", "Here.");
        let b = source_fingerprint("यहाँ", "yahā̃", "Here.");

        assert_eq!(a, b);
        assert!(a.starts_with("sha256:"));
    }

    #[test]
    fn classifies_missing_lineage_output() {
        let plan = build_plan(
            vec![source_file("0001", "fp")],
            vec![],
            vec![AcceptedCard { source_ref: None }],
            1,
        );

        assert_eq!(plan.done, 0);
        assert_eq!(plan.missing_lineage, 1);
        assert_eq!(plan.pending_items, 1);
    }

    #[test]
    fn classifies_done_when_source_ref_matches() {
        let source = source_file("0001", "fp");
        let plan = build_plan(
            vec![source],
            vec![],
            vec![AcceptedCard {
                source_ref: Some(SourceRef {
                    file: "input/sentences/example.yaml".to_string(),
                    item_id: "0001".to_string(),
                    fingerprint: "fp".to_string(),
                }),
            }],
            1,
        );

        assert_eq!(plan.done, 1);
        assert_eq!(plan.pending_items, 0);
    }

    #[test]
    fn classifies_source_changed_when_fingerprint_differs() {
        let plan = build_plan(
            vec![source_file("0001", "current")],
            vec![],
            vec![AcceptedCard {
                source_ref: Some(SourceRef {
                    file: "input/sentences/example.yaml".to_string(),
                    item_id: "0001".to_string(),
                    fingerprint: "old".to_string(),
                }),
            }],
            1,
        );

        assert_eq!(plan.source_changed, 1);
        assert_eq!(plan.pending_items, 1);
    }

    #[test]
    fn plans_next_unused_batch_filename() {
        let path = next_batch_path(
            "complete_hindi_chapter_02_sentences",
            &[
                PathBuf::from("output/sentences/complete_hindi_chapter_02_sentences_batch_01.json"),
                PathBuf::from("output/sentences/complete_hindi_chapter_02_sentences_batch_04.json"),
            ],
            0,
        );

        assert_eq!(
            path,
            PathBuf::from("output/sentences/complete_hindi_chapter_02_sentences_batch_02.json")
        );
    }

    #[test]
    fn max_batches_limits_output_files() {
        let source = SourceFile {
            relative_path: PathBuf::from("input/sentences/example.yaml"),
            stem: "example".to_string(),
            items: (1..=12)
                .map(|index| super::SourceRow {
                    id: format!("{index:04}"),
                    hindi: String::new(),
                    romanisation: String::new(),
                    english: String::new(),
                    fingerprint: format!("fp-{index}"),
                })
                .collect(),
        };
        let plan = build_plan(vec![source], vec![], vec![], 2);

        assert_eq!(plan.planned_files.len(), 2);
        assert_eq!(plan.planned_items, 10);
        assert_eq!(plan.deferred_items, 2);
    }

    #[test]
    fn parses_source_rows() {
        let rows = parse_source_rows(
            "title: Test\nitems:\n  - id: \"0001\"\n    hindi: \"यहाँ\"\n    romanisation: \"yahā̃\"\n    english: \"Here.\"\n",
        );

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "0001");
        assert_eq!(rows[0].hindi, "यहाँ");
    }

    #[test]
    fn parses_accepted_cards_with_source_ref() {
        let cards = parse_accepted_cards(
            r#"{"sentences":[{"source_ref":{"file":"input/sentences/example.yaml","item_id":"0001","fingerprint":"fp"}}]}"#,
        );

        assert_eq!(cards.len(), 1);
        assert!(cards[0].source_ref.is_some());
    }

    fn source_file(id: &str, fingerprint: &str) -> SourceFile {
        SourceFile {
            relative_path: PathBuf::from("input/sentences/example.yaml"),
            stem: "example".to_string(),
            items: vec![super::SourceRow {
                id: id.to_string(),
                hindi: String::new(),
                romanisation: String::new(),
                english: String::new(),
                fingerprint: fingerprint.to_string(),
            }],
        }
    }
}
