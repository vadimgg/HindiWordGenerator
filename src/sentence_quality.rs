use crate::project::{ProjectRoot, ProjectRootError};
use crate::sentence_schema::{parse_sentence_batch, SentenceBatch, SentenceCard};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const SENTENCE_OUTPUT_DIR: &str = "output/sentences";

#[derive(Debug)]
pub enum SentenceQualityError {
    Project(ProjectRootError),
    Io { path: PathBuf, source: io::Error },
}

impl std::fmt::Display for SentenceQualityError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SentenceQualityError::Project(error) => write!(formatter, "{error}"),
            SentenceQualityError::Io { path, source } => {
                write!(formatter, "Could not read {}\n\n{source}", path.display())
            }
        }
    }
}

impl From<ProjectRootError> for SentenceQualityError {
    fn from(error: ProjectRootError) -> Self {
        SentenceQualityError::Project(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentenceQualityReport {
    files: usize,
    cards: usize,
    problems: Vec<QualityFinding>,
    warnings: Vec<QualityFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QualityFinding {
    file: PathBuf,
    item_id: Option<String>,
    hindi: String,
    romanisation: String,
    english: String,
    issue: String,
    action: String,
}

pub fn quality_from_current_dir() -> Result<SentenceQualityReport, SentenceQualityError> {
    let root = ProjectRoot::discover_from(current_dir()?)?;
    quality_from(&root)
}

fn current_dir() -> Result<PathBuf, SentenceQualityError> {
    std::env::current_dir().map_err(|source| SentenceQualityError::Io {
        path: PathBuf::from("."),
        source,
    })
}

fn quality_from(root: &ProjectRoot) -> Result<SentenceQualityReport, SentenceQualityError> {
    let paths = collect_json_paths(root)?;
    let mut report = SentenceQualityReport {
        files: paths.len(),
        cards: 0,
        problems: Vec::new(),
        warnings: Vec::new(),
    };

    for relative_path in paths {
        let path = root.join(&relative_path);
        let content = fs::read_to_string(&path)
            .map_err(|source| SentenceQualityError::Io { path, source })?;
        match parse_sentence_batch(&content) {
            Ok(batch) => scan_batch(&relative_path, &batch, &mut report),
            Err(error) => report.problems.push(QualityFinding {
                file: relative_path,
                item_id: None,
                hindi: String::new(),
                romanisation: String::new(),
                english: String::new(),
                issue: format!("Accepted output is not valid JSON: {error}"),
                action: "Repair or archive this output file before studying/exporting.".to_string(),
            }),
        }
    }

    Ok(report)
}

impl SentenceQualityReport {
    pub fn has_problems(&self) -> bool {
        !self.problems.is_empty()
    }

    pub fn render(&self) -> String {
        let mut output = String::from("Sentence Quality\n\n");
        output.push_str("Accepted Output\n");
        output.push_str(&format!("  batch files  {}\n", self.files));
        output.push_str(&format!("  cards        {}\n", self.cards));
        output.push_str(&format!("  problems     {}\n", self.problems.len()));
        output.push_str(&format!("  warnings     {}\n", self.warnings.len()));

        if !self.problems.is_empty() {
            output.push_str("\nProblems\n");
            render_findings(&mut output, &self.problems);
        }

        if !self.warnings.is_empty() {
            output.push_str("\nWarnings\n");
            render_findings(&mut output, &self.warnings);
        }

        if self.problems.is_empty() && self.warnings.is_empty() {
            output.push_str("\nNext\n  No obvious learner-quality issues found.\n");
        } else if self.problems.is_empty() {
            output.push_str("\nNext\n  Review warnings before generating many more cards.\n");
        } else {
            output.push_str("\nNext\n  Fix problems before studying/exporting these cards.\n");
        }

        output
    }
}

fn render_findings(output: &mut String, findings: &[QualityFinding]) {
    for finding in findings {
        output.push_str("\nFile\n");
        output.push_str(&format!("  {}\n", finding.file.display()));
        if let Some(item_id) = &finding.item_id {
            output.push_str(&format!("Item\n  {item_id}\n"));
        }
        if !finding.hindi.is_empty() || !finding.romanisation.is_empty() {
            output.push_str("Hindi\n");
            output.push_str(&format!("  {}\n", finding.hindi));
            output.push_str("Roman\n");
            output.push_str(&format!("  {}\n", finding.romanisation));
        }
        if !finding.english.is_empty() {
            output.push_str("English\n");
            output.push_str(&format!("  {}\n", finding.english));
        }
        output.push_str("Issue\n");
        output.push_str(&format!("  {}\n", finding.issue));
        output.push_str("Action\n");
        output.push_str(&format!("  {}\n", finding.action));
    }
}

fn scan_batch(path: &Path, batch: &SentenceBatch, report: &mut SentenceQualityReport) {
    for card in &batch.sentences {
        report.cards += 1;
        scan_card(path, card, report);
    }
}

fn scan_card(path: &Path, card: &SentenceCard, report: &mut SentenceQualityReport) {
    let hindi = card.hindi.as_deref().unwrap_or_default();
    let romanisation = card.romanisation.as_deref().unwrap_or_default();
    let english = card.english.as_deref().unwrap_or_default();

    if card
        .literal
        .as_deref()
        .is_some_and(|literal| contains_devanagari(literal) || literal.trim() == hindi.trim())
    {
        report.problems.push(finding(
            path,
            card,
            "Literal contains Hindi text instead of an English word-order rendering.",
            "Regenerate or repair the literal field.",
        ));
    }
    if is_yes_no_kya_context(romanisation, english)
        && card.literal.as_deref().is_some_and(|literal| {
            literal
                .trim_start()
                .to_ascii_lowercase()
                .starts_with("what ")
        })
    {
        report.problems.push(finding(
            path,
            card,
            "`kyā` is a yes/no question marker here, but the literal starts with \"What\".",
            "Repair the literal so it shows yes/no question word order instead of treating `kyā` as \"what\".",
        ));
    }

    if card.tokens.is_empty() || card.words.is_empty() {
        report.problems.push(finding(
            path,
            card,
            "Card has no token/word breakdown.",
            "Regenerate or repair the word breakdown before studying/exporting.",
        ));
    }

    for word in &card.words {
        let meaning = word.meaning.as_deref().unwrap_or_default().trim();
        if meaning.is_empty() {
            report.problems.push(finding(
                path,
                card,
                "A word entry has an empty learner meaning.",
                "Add a learner-facing meaning or regenerate the word breakdown.",
            ));
        }

        if is_yes_no_kya_context(romanisation, english)
            && word.roman.as_deref() == Some("kyā")
            && meaning_says_only_what(meaning)
        {
            report.problems.push(finding(
                path,
                card,
                "`kyā` is used as a yes/no question marker, but the learner meaning says only \"what\".",
                "Use a contextual meaning like \"yes/no question marker\" for this card.",
            ));
        }
    }
}

fn meaning_says_only_what(meaning: &str) -> bool {
    let meaning = meaning.to_ascii_lowercase();
    meaning.contains("what")
        && !meaning.contains("yes/no")
        && !meaning.contains("question marker")
        && !meaning.contains("question particle")
}

fn finding(path: &Path, card: &SentenceCard, issue: &str, action: &str) -> QualityFinding {
    QualityFinding {
        file: path.to_path_buf(),
        item_id: card
            .source_ref
            .as_ref()
            .map(|source_ref| source_ref.item_id.clone()),
        hindi: card.hindi.clone().unwrap_or_default(),
        romanisation: card.romanisation.clone().unwrap_or_default(),
        english: card.english.clone().unwrap_or_default(),
        issue: issue.to_string(),
        action: action.to_string(),
    }
}

fn contains_devanagari(value: &str) -> bool {
    value
        .chars()
        .any(|ch| ('\u{0900}'..='\u{097F}').contains(&ch))
}

fn is_yes_no_kya_context(romanisation: &str, english: &str) -> bool {
    let starts_with_kya = romanisation
        .trim_start()
        .to_ascii_lowercase()
        .starts_with("kyā ");
    let english = english.trim_start().to_ascii_lowercase();
    let starts_with_auxiliary = [
        "am ", "are ", "is ", "was ", "were ", "do ", "does ", "did ", "can ", "could ", "will ",
        "would ", "should ", "have ", "has ", "had ",
    ]
    .iter()
    .any(|prefix| english.starts_with(prefix));

    starts_with_kya && starts_with_auxiliary
}

fn collect_json_paths(root: &ProjectRoot) -> Result<Vec<PathBuf>, SentenceQualityError> {
    let dir = root.join(SENTENCE_OUTPUT_DIR);
    let entries = fs::read_dir(&dir).map_err(|source| SentenceQualityError::Io {
        path: dir.clone(),
        source,
    })?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| SentenceQualityError::Io {
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

#[cfg(test)]
mod tests {
    use super::{contains_devanagari, is_yes_no_kya_context, scan_batch, SentenceQualityReport};
    use crate::sentence_schema::{SentenceBatch, SentenceCard, SentenceWord, SourceRef};
    use std::path::Path;

    #[test]
    fn detects_hindi_literal() {
        let batch = SentenceBatch {
            title: None,
            subtitle: None,
            sentences: vec![card(
                "0001",
                "क्या यह अच्छी है?",
                "kyā yah acchī hai?",
                "Is it good?",
            )
            .with_literal("क्या यह अच्छी है?")],
        };
        let mut report = empty_report();

        scan_batch(
            Path::new("output/sentences/example.json"),
            &batch,
            &mut report,
        );

        assert_eq!(report.problems.len(), 1);
        assert!(report.problems[0].issue.contains("Literal contains Hindi"));
    }

    #[test]
    fn detects_yes_no_kya_meaning() {
        let batch = SentenceBatch {
            title: None,
            subtitle: None,
            sentences: vec![card(
                "0001",
                "क्या यह अच्छी है?",
                "kyā yah acchī hai?",
                "Is it good?",
            )
            .with_word("क्या", "kyā", "what")],
        };
        let mut report = empty_report();

        scan_batch(
            Path::new("output/sentences/example.json"),
            &batch,
            &mut report,
        );

        assert_eq!(report.problems.len(), 1);
        assert!(report.problems[0].issue.contains("yes/no question marker"));
    }

    #[test]
    fn does_not_flag_content_question_kya() {
        let batch = SentenceBatch {
            title: None,
            subtitle: None,
            sentences: vec![card(
                "0001",
                "और वह मोटी किताब क्या है?",
                "aur vah moṭī kitāb kyā hai?",
                "And what is that thick book?",
            )
            .with_word("क्या", "kyā", "what")],
        };
        let mut report = empty_report();

        scan_batch(
            Path::new("output/sentences/example.json"),
            &batch,
            &mut report,
        );

        assert!(report.problems.is_empty());
    }

    #[test]
    fn detects_yes_no_kya_literal_starting_with_what() {
        let batch = SentenceBatch {
            title: None,
            subtitle: None,
            sentences: vec![card(
                "0001",
                "क्या यह अच्छी है?",
                "kyā yah acchī hai?",
                "Is it good?",
            )
            .with_literal("What this good is?")],
        };
        let mut report = empty_report();

        scan_batch(
            Path::new("output/sentences/example.json"),
            &batch,
            &mut report,
        );

        assert_eq!(report.problems.len(), 1);
        assert!(report.problems[0].issue.contains("literal starts"));
    }

    #[test]
    fn detects_devanagari() {
        assert!(contains_devanagari("क्यों"));
        assert!(!contains_devanagari("kyõ"));
    }

    #[test]
    fn detects_yes_no_context() {
        assert!(is_yes_no_kya_context("kyā yah acchī hai?", "Is it good?"));
        assert!(!is_yes_no_kya_context(
            "aur vah kitāb kyā hai?",
            "And what is that book?"
        ));
    }

    fn empty_report() -> SentenceQualityReport {
        SentenceQualityReport {
            files: 1,
            cards: 0,
            problems: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn card(id: &str, hindi: &str, romanisation: &str, english: &str) -> SentenceCard {
        SentenceCard {
            hindi: Some(hindi.to_string()),
            romanisation: Some(romanisation.to_string()),
            english: Some(english.to_string()),
            literal: Some(english.to_string()),
            register: Some("standard".to_string()),
            source_ref: Some(SourceRef {
                file: "input/sentences/example.yaml".to_string(),
                item_id: id.to_string(),
                fingerprint: "sha256:test".to_string(),
            }),
            tokens: vec![],
            words: vec![],
            anki_tags: vec![],
            audio: None,
        }
    }

    trait CardBuilder {
        fn with_literal(self, literal: &str) -> Self;
        fn with_word(self, hindi: &str, roman: &str, meaning: &str) -> Self;
    }

    impl CardBuilder for SentenceCard {
        fn with_literal(mut self, literal: &str) -> Self {
            self.literal = Some(literal.to_string());
            self.tokens = vec![crate::sentence_schema::SentenceToken {
                hindi: Some("क्या".to_string()),
                roman: Some("kyā".to_string()),
                kind: Some("word".to_string()),
                word_id: Some("w1".to_string()),
                word_index: None,
            }];
            self.words = vec![SentenceWord {
                id: Some("w1".to_string()),
                hindi: Some("क्या".to_string()),
                roman: Some("kyā".to_string()),
                meaning: Some("yes/no question marker".to_string()),
                kind: Some("word".to_string()),
                gender: None,
                number: None,
                note: None,
            }];
            self
        }

        fn with_word(mut self, hindi: &str, roman: &str, meaning: &str) -> Self {
            self.tokens = vec![crate::sentence_schema::SentenceToken {
                hindi: Some(hindi.to_string()),
                roman: Some(roman.to_string()),
                kind: Some("word".to_string()),
                word_id: Some("w1".to_string()),
                word_index: None,
            }];
            self.words = vec![SentenceWord {
                id: Some("w1".to_string()),
                hindi: Some(hindi.to_string()),
                roman: Some(roman.to_string()),
                meaning: Some(meaning.to_string()),
                kind: Some("word".to_string()),
                gender: None,
                number: None,
                note: None,
            }];
            self
        }
    }
}
