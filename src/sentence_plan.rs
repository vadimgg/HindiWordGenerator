use crate::project::{ProjectRoot, ProjectRootError};
use crate::source_identity::source_fingerprint;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const SENTENCE_SOURCE_DIR: &str = "input/sentences";
const SENTENCE_OUTPUT_DIR: &str = "output/sentences";
const DEFAULT_BATCH_SIZE: usize = 1;

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
    legacy_format: usize,
    content_duplicates: usize,
    invalid_output: usize,
    source_changed: usize,
    pending_items: usize,
    planned_items: usize,
    deferred_items: usize,
    max_batches: usize,
    batch_size: usize,
    planned_files: Vec<PathBuf>,
    warnings: Vec<String>,
    errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceFile {
    relative_path: PathBuf,
    stem: String,
    title: Option<String>,
    subtitle: Option<String>,
    items: Vec<SourceRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceRow {
    id: String,
    hindi: String,
    romanisation: String,
    english: String,
    tags: Vec<String>,
    fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AcceptedCard {
    path: PathBuf,
    hindi: String,
    romanisation: String,
    english: String,
    source_ref: Option<SourceRef>,
    has_legacy_word_index: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceRef {
    file: String,
    item_id: String,
    fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AcceptedOutputScan {
    cards: Vec<AcceptedCard>,
    errors: Vec<String>,
}

pub fn plan_from_current_dir(max_batches: usize) -> Result<SentencePlan, SentencePlanError> {
    let root = ProjectRoot::discover_from(current_dir()?)?;
    plan(&root, max_batches)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentenceGenerationPlan {
    pub batches: Vec<PlannedSentenceBatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedSentenceBatch {
    pub source_file: PathBuf,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub target_path: PathBuf,
    pub rows: Vec<PlannedSentenceRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedSentenceRow {
    pub id: String,
    pub hindi: String,
    pub romanisation: String,
    pub english: String,
    pub tags: Vec<String>,
    pub fingerprint: String,
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
    let output_scan = load_accepted_output(root, &output_paths)?;
    Ok(build_plan(
        source_files,
        output_paths,
        output_scan.cards,
        output_scan.errors,
        max_batches,
    ))
}

pub fn generation_plan(
    root: &ProjectRoot,
    max_batches: usize,
) -> Result<(SentencePlan, SentenceGenerationPlan), SentencePlanError> {
    let source_files = load_source_files(root)?;
    let output_paths = collect_json_paths(root)?;
    let output_scan = load_accepted_output(root, &output_paths)?;
    let summary = build_plan(
        source_files.clone(),
        output_paths.clone(),
        output_scan.cards.clone(),
        output_scan.errors,
        max_batches,
    );
    let generation =
        build_generation_plan(source_files, output_paths, output_scan.cards, max_batches);
    Ok((summary, generation))
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
        output.push_str(&format!("  legacy format      {}\n", self.legacy_format));
        output.push_str(&format!(
            "  content duplicate  {}\n",
            self.content_duplicates
        ));
        output.push_str(&format!("  invalid output     {}\n", self.invalid_output));
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

        if !self.warnings.is_empty() {
            output.push_str("\nWarnings\n");
            for warning in &self.warnings {
                for line in warning.lines() {
                    output.push_str(&format!("  {line}\n"));
                }
            }
        }

        if self.has_errors() {
            output.push_str("\nProblems\n");
            for error in &self.errors {
                for line in error.lines() {
                    output.push_str(&format!("  {line}\n"));
                }
            }
            output.push_str("\nNext\n  Fix source/output issues, then rerun the planner.");
        } else {
            output.push_str("\nNext\n  hindi sentences generate --max-batches 1");
        }

        output
    }
}

fn build_plan(
    source_files: Vec<SourceFile>,
    output_paths: Vec<PathBuf>,
    accepted_cards: Vec<AcceptedCard>,
    output_errors: Vec<String>,
    max_batches: usize,
) -> SentencePlan {
    let mut source_errors = Vec::new();
    let mut source_index = BTreeMap::new();
    let mut content_index = BTreeMap::new();
    for source_file in &source_files {
        let mut ids = BTreeSet::new();
        for row in &source_file.items {
            if row.id.len() != 4 || !row.id.bytes().all(|byte| byte.is_ascii_digit()) {
                source_errors.push(format!(
                    "Malformed source id {:?} in {}.",
                    row.id,
                    source_file.relative_path.display()
                ));
            }
            if !ids.insert(row.id.clone()) {
                source_errors.push(format!(
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
            content_index.insert(row.fingerprint.clone(), (source_file, row));
        }
    }

    let mut errors = source_errors.clone();
    errors.extend(output_errors.clone());
    let mut warnings = Vec::new();
    let mut done_keys = BTreeSet::new();
    let mut done = 0;
    let mut missing_lineage = 0;
    let mut legacy_format = 0;
    let mut content_duplicates = 0;
    let mut content_duplicate_problems = Vec::new();
    let mut source_changed = 0;
    for card in &accepted_cards {
        if card.has_legacy_word_index {
            legacy_format += 1;
        }
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

        let fingerprint = source_fingerprint(&card.hindi, &card.romanisation, &card.english);
        let Some((source_file, source_row)) = content_index.get(&fingerprint) else {
            continue;
        };
        if card_matches_source_ref(card, source_file, source_row) {
            continue;
        }
        content_duplicates += 1;
        if card_has_current_lineage(card, &source_index) {
            warnings.push(content_repeat_warning(card, source_file, source_row));
        } else {
            content_duplicate_problems.push(content_duplicate_problem(
                card,
                source_file,
                source_row,
            ));
        }
    }
    let duplicate_problem_count = content_duplicate_problems.len();
    errors.extend(content_duplicate_problems.into_iter().take(5));
    if duplicate_problem_count > 5 {
        errors.push(format!(
            "Additional content duplicates\n\nProblem\n  {remaining} more accepted cards match current source rows by Hindi/Roman/English content but do not have current source_ref lineage.\n\nAction\n  Archive or repair legacy output before generating more cards.",
            remaining = duplicate_problem_count - 5
        ));
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
            .saturating_sub(source_errors.len()),
        output_files: output_paths.len(),
        accepted_cards: accepted_cards.len(),
        done,
        missing_lineage,
        legacy_format,
        content_duplicates,
        invalid_output: output_errors.len(),
        source_changed,
        pending_items,
        planned_items,
        deferred_items,
        max_batches,
        batch_size: DEFAULT_BATCH_SIZE,
        planned_files,
        warnings,
        errors,
    }
}

fn build_generation_plan(
    source_files: Vec<SourceFile>,
    output_paths: Vec<PathBuf>,
    accepted_cards: Vec<AcceptedCard>,
    max_batches: usize,
) -> SentenceGenerationPlan {
    let done_keys = done_keys(&source_files, &accepted_cards);
    let mut remaining_batches = max_batches;
    let mut planned_files = Vec::new();
    let mut batches = Vec::new();

    for source_file in source_files {
        if remaining_batches == 0 {
            break;
        }
        let pending_rows = source_file
            .items
            .iter()
            .filter(|row| {
                !done_keys.contains(&(
                    source_file.relative_path.to_string_lossy().to_string(),
                    row.id.clone(),
                ))
            })
            .collect::<Vec<_>>();

        for chunk in pending_rows.chunks(DEFAULT_BATCH_SIZE) {
            if remaining_batches == 0 {
                break;
            }
            let target_path = next_batch_path(
                &source_file.stem,
                &output_paths,
                planned_files_for_stem(&planned_files, &source_file.stem),
            );
            planned_files.push(target_path.clone());
            batches.push(PlannedSentenceBatch {
                source_file: source_file.relative_path.clone(),
                title: source_file.title.clone(),
                subtitle: source_file.subtitle.clone(),
                target_path,
                rows: chunk
                    .iter()
                    .map(|row| PlannedSentenceRow {
                        id: row.id.clone(),
                        hindi: row.hindi.clone(),
                        romanisation: row.romanisation.clone(),
                        english: row.english.clone(),
                        tags: row.tags.clone(),
                        fingerprint: row.fingerprint.clone(),
                    })
                    .collect(),
            });
            remaining_batches -= 1;
        }
    }

    SentenceGenerationPlan { batches }
}

fn card_matches_source_ref(
    card: &AcceptedCard,
    source_file: &SourceFile,
    source_row: &SourceRow,
) -> bool {
    let Some(source_ref) = &card.source_ref else {
        return false;
    };
    source_ref.file == source_file.relative_path.to_string_lossy().as_ref()
        && source_ref.item_id == source_row.id
        && source_ref.fingerprint == source_row.fingerprint
}

fn card_has_current_lineage(
    card: &AcceptedCard,
    source_index: &BTreeMap<(String, String), String>,
) -> bool {
    let Some(source_ref) = &card.source_ref else {
        return false;
    };
    let key = (source_ref.file.clone(), source_ref.item_id.clone());
    source_index
        .get(&key)
        .is_some_and(|current| current == &source_ref.fingerprint)
}

fn content_repeat_warning(
    card: &AcceptedCard,
    source_file: &SourceFile,
    source_row: &SourceRow,
) -> String {
    format!(
        "Repeated source content\n\nHindi\n  {}\n\nRoman\n  {}\n\nEnglish\n  {}\n\nAlready accepted in\n  {}\n\nAlso appears in source\n  {} item {}\n\nAction\n  This is valid Rust output from another source row, so generation may continue.",
        card.hindi,
        card.romanisation,
        card.english,
        card.path.display(),
        source_file.relative_path.display(),
        source_row.id,
    )
}

fn content_duplicate_problem(
    card: &AcceptedCard,
    source_file: &SourceFile,
    source_row: &SourceRow,
) -> String {
    let source_ref_problem = if card.source_ref.is_some() {
        "Existing card matches the source text but its source_ref does not match the current source row."
    } else {
        "Existing card matches the source text but has no source_ref."
    };
    let format_problem = if card.has_legacy_word_index {
        "\n  Existing card also uses legacy word_index output instead of Rust word_id output."
    } else {
        ""
    };

    format!(
        "Possible duplicate accepted output\n\nHindi\n  {}\n\nRoman\n  {}\n\nEnglish\n  {}\n\nFound in\n  {}\n\nMatches source\n  {} item {}\n\nProblem\n  {}{}\n\nAction\n  Archive or repair legacy output before generating more cards.",
        card.hindi,
        card.romanisation,
        card.english,
        card.path.display(),
        source_file.relative_path.display(),
        source_row.id,
        source_ref_problem,
        format_problem,
    )
}

fn done_keys(
    source_files: &[SourceFile],
    accepted_cards: &[AcceptedCard],
) -> BTreeSet<(String, String)> {
    let mut source_index = BTreeMap::new();
    for source_file in source_files {
        for row in &source_file.items {
            source_index.insert(
                (
                    source_file.relative_path.to_string_lossy().to_string(),
                    row.id.clone(),
                ),
                row.fingerprint.clone(),
            );
        }
    }

    let mut done = BTreeSet::new();
    for card in accepted_cards {
        let Some(source_ref) = &card.source_ref else {
            continue;
        };
        let key = (source_ref.file.clone(), source_ref.item_id.clone());
        if source_index
            .get(&key)
            .is_some_and(|current| current == &source_ref.fingerprint)
        {
            done.insert(key);
        }
    }
    done
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
        let title = top_level_field(&content, "title");
        let subtitle = top_level_field(&content, "subtitle");
        files.push(SourceFile {
            relative_path,
            stem,
            title,
            subtitle,
            items: parse_source_rows(&content),
        });
    }
    Ok(files)
}

fn top_level_field(content: &str, name: &str) -> Option<String> {
    let prefix = format!("{name}: ");
    content.lines().find_map(|line| {
        line.strip_prefix(&prefix)
            .map(|value| unquote(value.trim()).to_string())
    })
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
        let tags = list_field(block, "tags");
        let fingerprint = source_fingerprint(&hindi, &romanisation, &english);
        rows.push(SourceRow {
            id,
            hindi,
            romanisation,
            english,
            tags,
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

fn list_field(block: &[&str], name: &str) -> Vec<String> {
    let field_prefix = format!("    {name}:");
    let Some(start) = block.iter().position(|line| *line == field_prefix) else {
        return Vec::new();
    };
    block[start + 1..]
        .iter()
        .take_while(|line| line.starts_with("      - "))
        .filter_map(|line| {
            line.strip_prefix("      - ")
                .map(|value| unquote(value.trim()).to_string())
        })
        .collect()
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

fn load_accepted_output(
    root: &ProjectRoot,
    paths: &[PathBuf],
) -> Result<AcceptedOutputScan, SentencePlanError> {
    let mut cards = Vec::new();
    let mut errors = Vec::new();
    for relative_path in paths {
        let path = root.join(relative_path);
        let content =
            fs::read_to_string(&path).map_err(|source| SentencePlanError::Io { path, source })?;
        match parse_accepted_cards(relative_path, &content) {
            Ok(parsed) => cards.extend(parsed),
            Err(error) => errors.push(error),
        }
    }
    Ok(AcceptedOutputScan { cards, errors })
}

fn parse_accepted_cards(path: &Path, content: &str) -> Result<Vec<AcceptedCard>, String> {
    let value: Value = serde_json::from_str(content).map_err(|source| {
        format!(
            "Invalid accepted output\n\nFile\n  {}\n\nProblem\n  JSON parse failed: {source}\n\nAction\n  Archive or repair this file before generating more cards.",
            path.display()
        )
    })?;
    let Some(sentences) = value.get("sentences").and_then(Value::as_array) else {
        return Err(format!(
            "Invalid accepted output\n\nFile\n  {}\n\nProblem\n  Missing sentences array.\n\nAction\n  Archive or repair this file before generating more cards.",
            path.display()
        ));
    };

    let mut cards = Vec::new();
    for (index, sentence) in sentences.iter().enumerate() {
        let Some(object) = sentence.as_object() else {
            return Err(invalid_sentence_problem(
                path,
                index,
                "sentence entry is not an object",
            ));
        };
        let Some(hindi) = object.get("hindi").and_then(Value::as_str) else {
            return Err(invalid_sentence_problem(path, index, "missing hindi"));
        };
        let Some(romanisation) = object.get("romanisation").and_then(Value::as_str) else {
            return Err(invalid_sentence_problem(
                path,
                index,
                "missing romanisation",
            ));
        };
        let Some(english) = object.get("english").and_then(Value::as_str) else {
            return Err(invalid_sentence_problem(path, index, "missing english"));
        };

        cards.push(AcceptedCard {
            path: path.to_path_buf(),
            hindi: hindi.to_string(),
            romanisation: romanisation.to_string(),
            english: english.to_string(),
            source_ref: parse_source_ref_value(sentence),
            has_legacy_word_index: has_legacy_word_index(sentence),
        });
    }

    Ok(cards)
}

fn invalid_sentence_problem(path: &Path, index: usize, problem: &str) -> String {
    format!(
        "Invalid accepted output\n\nFile\n  {}\n\nProblem\n  Sentence {} {problem}.\n\nAction\n  Archive or repair this file before generating more cards.",
        path.display(),
        index + 1,
    )
}

fn parse_source_ref_value(sentence: &Value) -> Option<SourceRef> {
    let source_ref = sentence.get("source_ref")?.as_object()?;
    Some(SourceRef {
        file: source_ref.get("file")?.as_str()?.to_string(),
        item_id: source_ref.get("item_id")?.as_str()?.to_string(),
        fingerprint: source_ref.get("fingerprint")?.as_str()?.to_string(),
    })
}

fn has_legacy_word_index(sentence: &Value) -> bool {
    sentence
        .get("tokens")
        .and_then(Value::as_array)
        .is_some_and(|tokens| {
            tokens.iter().any(|token| {
                let has_numeric_word_index = token
                    .get("word_index")
                    .is_some_and(|value| value.as_u64().is_some());
                let has_word_id = token.get("word_id").and_then(Value::as_str).is_some();
                has_numeric_word_index && !has_word_id
            })
        })
}

#[cfg(test)]
mod tests {
    use super::{
        build_plan, next_batch_path, parse_accepted_cards, parse_source_rows, AcceptedCard,
        SourceFile, SourceRef,
    };
    use crate::source_identity::source_fingerprint;
    use std::path::{Path, PathBuf};

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
            vec![accepted_card(None)],
            vec![],
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
            vec![accepted_card(Some(SourceRef {
                file: "input/sentences/example.yaml".to_string(),
                item_id: "0001".to_string(),
                fingerprint: "fp".to_string(),
            }))],
            vec![],
            1,
        );

        assert_eq!(plan.done, 1);
        assert_eq!(plan.pending_items, 0);
    }

    #[test]
    fn flags_legacy_content_duplicate_without_source_ref() {
        let hindi = "अध्यापक जी, यहाँ कितने विद्यार्थी हैं?";
        let romanisation = "adhyāpak jī, yahā̃ kitne vidyārthī haĩ?";
        let english = "Teacher ji, how many students are here?";
        let plan = build_plan(
            vec![source_file_with_text("0001", hindi, romanisation, english)],
            vec![],
            vec![AcceptedCard {
                path: PathBuf::from("output/sentences/old.json"),
                hindi: hindi.to_string(),
                romanisation: romanisation.to_string(),
                english: english.to_string(),
                source_ref: None,
                has_legacy_word_index: true,
            }],
            vec![],
            1,
        );

        assert_eq!(plan.content_duplicates, 1);
        assert_eq!(plan.legacy_format, 1);
        assert!(plan.has_errors());
        assert!(plan.render().contains("Possible duplicate accepted output"));
        assert!(plan.render().contains("Roman"));
        assert!(plan.render().contains("adhyāpak"));
    }

    #[test]
    fn ignores_content_duplicate_when_source_ref_is_current() {
        let hindi = "अध्यापक जी, यहाँ कितने विद्यार्थी हैं?";
        let romanisation = "adhyāpak jī, yahā̃ kitne vidyārthī haĩ?";
        let english = "Teacher ji, how many students are here?";
        let source = source_file_with_text("0001", hindi, romanisation, english);
        let fingerprint = source.items[0].fingerprint.clone();
        let plan = build_plan(
            vec![source],
            vec![],
            vec![AcceptedCard {
                path: PathBuf::from("output/sentences/rust.json"),
                hindi: hindi.to_string(),
                romanisation: romanisation.to_string(),
                english: english.to_string(),
                source_ref: Some(SourceRef {
                    file: "input/sentences/example.yaml".to_string(),
                    item_id: "0001".to_string(),
                    fingerprint,
                }),
                has_legacy_word_index: false,
            }],
            vec![],
            1,
        );

        assert_eq!(plan.content_duplicates, 0);
        assert_eq!(plan.done, 1);
        assert_eq!(plan.pending_items, 0);
    }

    #[test]
    fn repeated_current_content_warns_without_blocking() {
        let hindi = "क्यों?";
        let romanisation = "kyõ?";
        let english = "Why?";
        let accepted_source = source_file_with_text("0001", hindi, romanisation, english);
        let accepted_fingerprint = accepted_source.items[0].fingerprint.clone();
        let repeated_source = SourceFile {
            relative_path: PathBuf::from("input/sentences/another.yaml"),
            stem: "another".to_string(),
            title: Some("Example".to_string()),
            subtitle: Some("Another".to_string()),
            items: vec![super::SourceRow {
                id: "0002".to_string(),
                hindi: hindi.to_string(),
                romanisation: romanisation.to_string(),
                english: english.to_string(),
                tags: Vec::new(),
                fingerprint: source_fingerprint(hindi, romanisation, english),
            }],
        };

        let plan = build_plan(
            vec![accepted_source, repeated_source],
            vec![],
            vec![AcceptedCard {
                path: PathBuf::from("output/sentences/example.json"),
                hindi: hindi.to_string(),
                romanisation: romanisation.to_string(),
                english: english.to_string(),
                source_ref: Some(SourceRef {
                    file: "input/sentences/example.yaml".to_string(),
                    item_id: "0001".to_string(),
                    fingerprint: accepted_fingerprint,
                }),
                has_legacy_word_index: false,
            }],
            vec![],
            1,
        );

        assert_eq!(plan.content_duplicates, 1);
        assert_eq!(plan.warnings.len(), 1);
        assert!(!plan.has_errors());
    }

    #[test]
    fn classifies_source_changed_when_fingerprint_differs() {
        let plan = build_plan(
            vec![source_file("0001", "current")],
            vec![],
            vec![accepted_card(Some(SourceRef {
                file: "input/sentences/example.yaml".to_string(),
                item_id: "0001".to_string(),
                fingerprint: "old".to_string(),
            }))],
            vec![],
            1,
        );

        assert_eq!(plan.source_changed, 1);
        assert_eq!(plan.pending_items, 1);
    }

    #[test]
    fn reports_invalid_output_errors_without_reducing_valid_source_ids() {
        let plan = build_plan(
            vec![source_file("0001", "fp")],
            vec![],
            vec![accepted_card(Some(SourceRef {
                file: "input/sentences/example.yaml".to_string(),
                item_id: "0001".to_string(),
                fingerprint: "fp".to_string(),
            }))],
            vec!["Invalid accepted output".to_string()],
            1,
        );

        assert_eq!(plan.invalid_output, 1);
        assert_eq!(plan.valid_ids, 1);
        assert!(plan.has_errors());
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
            title: Some("Example".to_string()),
            subtitle: Some("Chapter".to_string()),
            items: (1..=12)
                .map(|index| super::SourceRow {
                    id: format!("{index:04}"),
                    hindi: String::new(),
                    romanisation: String::new(),
                    english: String::new(),
                    tags: Vec::new(),
                    fingerprint: format!("fp-{index}"),
                })
                .collect(),
        };
        let plan = build_plan(vec![source], vec![], vec![], vec![], 2);

        assert_eq!(plan.planned_files.len(), 2);
        assert_eq!(plan.planned_items, 2);
        assert_eq!(plan.deferred_items, 10);
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
            Path::new("output/sentences/example.json"),
            r#"{"sentences":[{"hindi":"यहाँ","romanisation":"yahā̃","english":"Here.","tokens":[{"word_index":0}],"source_ref":{"file":"input/sentences/example.yaml","item_id":"0001","fingerprint":"fp"}}]}"#,
        )
        .unwrap();

        assert_eq!(cards.len(), 1);
        assert!(cards[0].source_ref.is_some());
        assert!(cards[0].has_legacy_word_index);
    }

    #[test]
    fn rejects_accepted_output_without_sentence_array() {
        let error =
            parse_accepted_cards(Path::new("output/sentences/broken.json"), "{}").unwrap_err();

        assert!(error.contains("Missing sentences array"));
    }

    fn accepted_card(source_ref: Option<SourceRef>) -> AcceptedCard {
        AcceptedCard {
            path: PathBuf::from("output/sentences/example.json"),
            hindi: String::new(),
            romanisation: String::new(),
            english: String::new(),
            source_ref,
            has_legacy_word_index: false,
        }
    }

    fn source_file(id: &str, fingerprint: &str) -> SourceFile {
        SourceFile {
            relative_path: PathBuf::from("input/sentences/example.yaml"),
            stem: "example".to_string(),
            title: Some("Example".to_string()),
            subtitle: Some("Chapter".to_string()),
            items: vec![super::SourceRow {
                id: id.to_string(),
                hindi: String::new(),
                romanisation: String::new(),
                english: String::new(),
                tags: Vec::new(),
                fingerprint: fingerprint.to_string(),
            }],
        }
    }

    fn source_file_with_text(
        id: &str,
        hindi: &str,
        romanisation: &str,
        english: &str,
    ) -> SourceFile {
        SourceFile {
            relative_path: PathBuf::from("input/sentences/example.yaml"),
            stem: "example".to_string(),
            title: Some("Example".to_string()),
            subtitle: Some("Chapter".to_string()),
            items: vec![super::SourceRow {
                id: id.to_string(),
                hindi: hindi.to_string(),
                romanisation: romanisation.to_string(),
                english: english.to_string(),
                tags: Vec::new(),
                fingerprint: source_fingerprint(hindi, romanisation, english),
            }],
        }
    }
}
