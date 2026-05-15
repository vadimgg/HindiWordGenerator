#![allow(dead_code)]

use crate::sentence_schema::{SentenceBatch, SentenceCard, SourceRef};
use crate::source_identity::normalize_nfc;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedSource {
    pub file: String,
    pub item_id: String,
    pub fingerprint: String,
}

impl ExpectedSource {
    pub fn new(
        file: impl Into<String>,
        item_id: impl Into<String>,
        fingerprint: impl Into<String>,
    ) -> Self {
        Self {
            file: file.into(),
            item_id: item_id.into(),
            fingerprint: fingerprint.into(),
        }
    }

    fn key(&self) -> SourceKey {
        SourceKey {
            file: self.file.clone(),
            item_id: self.item_id.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    errors: Vec<String>,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    fn push(&mut self, message: impl Into<String>) {
        self.errors.push(message.into());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SourceKey {
    file: String,
    item_id: String,
}

pub fn validate_sentence_batch(
    batch: &SentenceBatch,
    expected_sources: &[ExpectedSource],
) -> ValidationReport {
    let mut report = ValidationReport { errors: Vec::new() };

    required_batch_field(&mut report, "title", batch.title.as_deref());
    required_batch_field(&mut report, "subtitle", batch.subtitle.as_deref());
    if batch.sentences.is_empty() {
        report.push("batch.sentences must contain at least one sentence.");
    }

    let expected_by_key = expected_sources
        .iter()
        .map(|source| (source.key(), source.fingerprint.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();

    for (index, sentence) in batch.sentences.iter().enumerate() {
        let label = format!("sentences[{index}]");
        validate_sentence_fields(&mut report, &label, sentence);
        validate_source_ref(
            &mut report,
            &label,
            sentence.source_ref.as_ref(),
            &expected_by_key,
            &mut seen,
        );
        validate_words_and_tokens(&mut report, &label, sentence);
        validate_roman_reconstruction(&mut report, &label, sentence);
    }

    for source in expected_sources {
        let key = source.key();
        if !seen.contains(&key) {
            report.push(format!(
                "missing candidate sentence for source {}#{}.",
                source.file, source.item_id
            ));
        }
    }

    report
}

fn required_batch_field(report: &mut ValidationReport, name: &str, value: Option<&str>) {
    if value.is_none_or(is_blank) {
        report.push(format!("batch.{name} is required."));
    }
}

fn validate_sentence_fields(report: &mut ValidationReport, label: &str, sentence: &SentenceCard) {
    for (name, value) in [
        ("hindi", sentence.hindi.as_deref()),
        ("romanisation", sentence.romanisation.as_deref()),
        ("english", sentence.english.as_deref()),
        ("literal", sentence.literal.as_deref()),
        ("register", sentence.register.as_deref()),
    ] {
        if value.is_none_or(is_blank) {
            report.push(format!("{label}.{name} is required."));
        }
    }

    if let Some(register) = sentence.register.as_deref() {
        if !matches!(register, "informal" | "standard" | "formal") {
            report.push(format!(
                "{label}.register has unsupported value {register:?}."
            ));
        }
    }
}

fn validate_source_ref(
    report: &mut ValidationReport,
    label: &str,
    source_ref: Option<&SourceRef>,
    expected_by_key: &BTreeMap<SourceKey, &str>,
    seen: &mut BTreeSet<SourceKey>,
) {
    let Some(source_ref) = source_ref else {
        report.push(format!("{label}.source_ref is required."));
        return;
    };
    let key = SourceKey {
        file: source_ref.file.clone(),
        item_id: source_ref.item_id.clone(),
    };
    if !seen.insert(key.clone()) {
        report.push(format!(
            "duplicate candidate sentence for source {}#{}.",
            source_ref.file, source_ref.item_id
        ));
    }
    match expected_by_key.get(&key) {
        Some(expected_fingerprint) if *expected_fingerprint == source_ref.fingerprint => {}
        Some(_) => report.push(format!(
            "{label}.source_ref.fingerprint does not match current source for {}#{}.",
            source_ref.file, source_ref.item_id
        )),
        None => report.push(format!(
            "{label}.source_ref does not match a planned source row: {}#{}.",
            source_ref.file, source_ref.item_id
        )),
    }
}

fn validate_words_and_tokens(report: &mut ValidationReport, label: &str, sentence: &SentenceCard) {
    if sentence.words.is_empty() {
        report.push(format!("{label}.words must contain at least one word."));
    }
    if sentence.tokens.is_empty() {
        report.push(format!("{label}.tokens must contain at least one token."));
    }

    let mut word_ids = BTreeSet::new();
    for (index, word) in sentence.words.iter().enumerate() {
        let word_label = format!("{label}.words[{index}]");
        let Some(id) = present(word.id.as_deref()) else {
            report.push(format!("{word_label}.id is required."));
            continue;
        };
        if !word_ids.insert(id.to_string()) {
            report.push(format!("{word_label}.id duplicates {id:?}."));
        }
        for (name, value) in [
            ("hindi", word.hindi.as_deref()),
            ("roman", word.roman.as_deref()),
            ("meaning", word.meaning.as_deref()),
        ] {
            if value.is_none_or(is_blank) {
                report.push(format!("{word_label}.{name} is required."));
            }
        }
        if let Some(kind) = word.kind.as_deref() {
            if kind != "word" {
                report.push(format!("{word_label}.kind must be \"word\" when present."));
            }
        }
    }

    let mut referenced = BTreeSet::new();
    for (index, token) in sentence.tokens.iter().enumerate() {
        let token_label = format!("{label}.tokens[{index}]");
        for (name, value) in [
            ("hindi", token.hindi.as_deref()),
            ("roman", token.roman.as_deref()),
            ("kind", token.kind.as_deref()),
        ] {
            if value.is_none_or(is_blank) {
                report.push(format!("{token_label}.{name} is required."));
            }
        }
        if token.kind.as_deref() != Some("word") {
            report.push(format!("{token_label}.kind must be \"word\"."));
        }
        if token.word_index.is_some() {
            report.push(format!(
                "{token_label}.word_index is legacy-only and invalid for new Rust output."
            ));
        }
        let Some(word_id) = present(token.word_id.as_deref()) else {
            report.push(format!("{token_label}.word_id is required."));
            continue;
        };
        if !word_ids.contains(word_id) {
            report.push(format!(
                "{token_label}.word_id references unknown word id {word_id:?}."
            ));
        } else {
            referenced.insert(word_id.to_string());
        }
    }

    for word_id in word_ids {
        if !referenced.contains(&word_id) {
            report.push(format!(
                "{label}.words id {word_id:?} is not referenced by any token."
            ));
        }
    }
}

fn validate_roman_reconstruction(
    report: &mut ValidationReport,
    label: &str,
    sentence: &SentenceCard,
) {
    let Some(romanisation) = present(sentence.romanisation.as_deref()) else {
        return;
    };
    let expected = roman_word_segments(&normalize_nfc(romanisation));
    let actual = sentence
        .tokens
        .iter()
        .filter_map(|token| token.roman.as_deref().map(normalize_nfc))
        .collect::<Vec<_>>();
    if !actual.is_empty() && expected != actual {
        report.push(format!(
            "{label}.tokens roman sequence does not reconstruct romanisation. expected {:?}, got {:?}.",
            expected, actual
        ));
    }
}

fn roman_word_segments(value: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    for ch in value.chars() {
        if is_roman_separator(ch) {
            if !current.is_empty() {
                segments.push(std::mem::take(&mut current));
            }
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }
    segments
}

fn is_roman_separator(ch: char) -> bool {
    ch.is_whitespace()
        || matches!(
            ch,
            ',' | '.'
                | '?'
                | '!'
                | ';'
                | ':'
                | '-'
                | '–'
                | '—'
                | '।'
                | '॥'
                | '('
                | ')'
                | '['
                | ']'
                | '"'
                | '“'
                | '”'
        )
}

fn present(value: Option<&str>) -> Option<&str> {
    value.filter(|value| !is_blank(value))
}

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::{validate_sentence_batch, ExpectedSource};
    use crate::sentence_schema::parse_sentence_batch;

    #[test]
    fn valid_batch_passes() {
        let batch = valid_batch();
        let report = validate_sentence_batch(&batch, &[expected()]);

        assert!(report.is_valid(), "{:?}", report.errors());
    }

    #[test]
    fn required_fields_and_register_are_validated() {
        let mut batch = valid_batch();
        batch.sentences[0].literal = Some(" ".to_string());
        batch.sentences[0].register = Some("neutral".to_string());

        let report = validate_sentence_batch(&batch, &[expected()]);

        assert!(errors_contain(&report, "literal is required"));
        assert!(errors_contain(&report, "unsupported value"));
    }

    #[test]
    fn rejects_space_tokens_word_index_unknown_links_and_unused_words() {
        let mut batch = valid_batch();
        batch.sentences[0].tokens[0].kind = Some("space".to_string());
        batch.sentences[0].tokens[0].word_index = Some(0);
        batch.sentences[0].tokens[0].word_id = Some("missing".to_string());
        batch.sentences[0]
            .words
            .push(crate::sentence_schema::SentenceWord {
                id: Some("w2".to_string()),
                hindi: Some("है".to_string()),
                roman: Some("hai".to_string()),
                meaning: Some("is".to_string()),
                kind: None,
                gender: None,
                number: None,
                note: None,
            });

        let report = validate_sentence_batch(&batch, &[expected()]);

        assert!(errors_contain(&report, "kind must be"));
        assert!(errors_contain(&report, "word_index is legacy-only"));
        assert!(errors_contain(&report, "unknown word id"));
        assert!(errors_contain(&report, "is not referenced"));
    }

    #[test]
    fn rejects_duplicate_word_ids() {
        let mut batch = valid_batch();
        let duplicate = batch.sentences[0].words[0].clone();
        batch.sentences[0].words.push(duplicate);

        let report = validate_sentence_batch(&batch, &[expected()]);

        assert!(errors_contain(&report, "duplicates"));
    }

    #[test]
    fn validates_exact_source_coverage_and_fingerprint() {
        let mut batch = valid_batch();
        batch.sentences[0].source_ref.as_mut().unwrap().fingerprint = "sha256:old".to_string();

        let report = validate_sentence_batch(&batch, &[expected()]);

        assert!(errors_contain(&report, "fingerprint does not match"));
    }

    #[test]
    fn rejects_extra_and_duplicate_source_rows() {
        let mut batch = valid_batch();
        batch.sentences.push(batch.sentences[0].clone());

        let report = validate_sentence_batch(&batch, &[expected()]);

        assert!(errors_contain(&report, "duplicate candidate sentence"));
    }

    #[test]
    fn validates_romanisation_reconstruction() {
        let mut batch = valid_batch();
        batch.sentences[0].romanisation = Some("yahā̃ hai?".to_string());
        batch.sentences[0]
            .tokens
            .push(crate::sentence_schema::SentenceToken {
                hindi: Some("है".to_string()),
                roman: Some("hai".to_string()),
                kind: Some("word".to_string()),
                word_id: Some("w2".to_string()),
                word_index: None,
            });
        batch.sentences[0]
            .words
            .push(crate::sentence_schema::SentenceWord {
                id: Some("w2".to_string()),
                hindi: Some("है".to_string()),
                roman: Some("hai".to_string()),
                meaning: Some("is".to_string()),
                kind: None,
                gender: None,
                number: None,
                note: None,
            });
        assert!(validate_sentence_batch(&batch, &[expected()]).is_valid());

        batch.sentences[0].tokens[1].roman = Some("he".to_string());
        let report = validate_sentence_batch(&batch, &[expected()]);
        assert!(errors_contain(&report, "does not reconstruct"));
    }

    #[test]
    fn romanisation_reconstruction_ignores_standalone_dash_punctuation() {
        let mut batch = valid_batch();
        batch.sentences[0].romanisation = Some("yahā̃ – hai.".to_string());
        batch.sentences[0]
            .tokens
            .push(crate::sentence_schema::SentenceToken {
                hindi: Some("है".to_string()),
                roman: Some("hai".to_string()),
                kind: Some("word".to_string()),
                word_id: Some("w2".to_string()),
                word_index: None,
            });
        batch.sentences[0]
            .words
            .push(crate::sentence_schema::SentenceWord {
                id: Some("w2".to_string()),
                hindi: Some("है".to_string()),
                roman: Some("hai".to_string()),
                meaning: Some("is".to_string()),
                kind: None,
                gender: None,
                number: None,
                note: None,
            });

        assert!(validate_sentence_batch(&batch, &[expected()]).is_valid());
    }

    fn valid_batch() -> crate::sentence_schema::SentenceBatch {
        parse_sentence_batch(
            r#"{
              "title": "Complete Hindi",
              "subtitle": "Chapter 02",
              "sentences": [{
                "hindi": "यहाँ",
                "romanisation": "yahā̃",
                "english": "Here.",
                "literal": "here",
                "register": "standard",
                "source_ref": {
                  "file": "input/sentences/example.yaml",
                  "item_id": "0001",
                  "fingerprint": "sha256:test"
                },
                "tokens": [{"hindi":"यहाँ","roman":"yahā̃","kind":"word","word_id":"w1"}],
                "words": [{"id":"w1","hindi":"यहाँ","roman":"yahā̃","meaning":"here"}]
              }]
            }"#,
        )
        .unwrap()
    }

    fn expected() -> ExpectedSource {
        ExpectedSource::new("input/sentences/example.yaml", "0001", "sha256:test")
    }

    fn errors_contain(report: &super::ValidationReport, needle: &str) -> bool {
        report.errors().iter().any(|error| error.contains(needle))
    }
}
