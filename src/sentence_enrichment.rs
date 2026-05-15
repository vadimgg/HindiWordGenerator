use crate::sentence_plan::{PlannedSentenceBatch, PlannedSentenceRow};
use crate::sentence_schema::{SentenceBatch, SentenceCard, SentenceToken, SentenceWord, SourceRef};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum EnrichmentError {
    JsonNotFound,
    Json(serde_json::Error),
    MissingItem(String),
    DuplicateItem(String),
}

impl std::fmt::Display for EnrichmentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnrichmentError::JsonNotFound => {
                write!(formatter, "Model response did not contain a JSON object.")
            }
            EnrichmentError::Json(error) => {
                write!(formatter, "Could not parse model enrichment JSON: {error}")
            }
            EnrichmentError::MissingItem(id) => {
                write!(
                    formatter,
                    "Model response did not include enrichment for source id {id}."
                )
            }
            EnrichmentError::DuplicateItem(id) => {
                write!(
                    formatter,
                    "Model response included duplicate enrichment for source id {id}."
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PromptInput<'a> {
    items: Vec<PromptItem<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PromptItem<'a> {
    id: &'a str,
    hindi: &'a str,
    romanisation: &'a str,
    english: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct EnrichmentResponse {
    #[serde(default)]
    items: Vec<EnrichmentItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct EnrichmentItem {
    id: String,
    literal: String,
    register: String,
    #[serde(default)]
    tokens: Vec<SentenceToken>,
    #[serde(default)]
    words: Vec<SentenceWord>,
    #[serde(default)]
    anki_tags: Vec<String>,
}

pub fn build_prompt(prompt_template: &str, rows: &[PlannedSentenceRow]) -> String {
    let input = PromptInput {
        items: rows
            .iter()
            .map(|row| PromptItem {
                id: &row.id,
                hindi: &row.hindi,
                romanisation: &row.romanisation,
                english: &row.english,
                tags: row.tags.iter().map(String::as_str).collect(),
            })
            .collect(),
    };
    let payload = serde_json::to_string_pretty(&input).expect("prompt input is serializable");
    format!("{prompt_template}\n\nINPUT\n```json\n{payload}\n```")
}

pub fn merge_enrichment(
    batch: &PlannedSentenceBatch,
    response_text: &str,
) -> Result<SentenceBatch, EnrichmentError> {
    let response = parse_response(response_text)?;
    let mut by_id = BTreeMap::new();
    for item in response.items {
        if by_id.contains_key(&item.id) {
            return Err(EnrichmentError::DuplicateItem(item.id));
        }
        by_id.insert(item.id.clone(), item);
    }

    let mut sentences = Vec::new();
    for row in &batch.rows {
        let Some(item) = by_id.remove(&row.id) else {
            return Err(EnrichmentError::MissingItem(row.id.clone()));
        };
        sentences.push(SentenceCard {
            hindi: Some(row.hindi.clone()),
            romanisation: Some(row.romanisation.clone()),
            english: Some(row.english.clone()),
            literal: Some(item.literal),
            register: Some(item.register),
            source_ref: Some(SourceRef {
                file: batch.source_file.to_string_lossy().to_string(),
                item_id: row.id.clone(),
                fingerprint: row.fingerprint.clone(),
            }),
            tokens: word_tokens_only(item.tokens),
            words: item.words,
            anki_tags: item.anki_tags,
            audio: None,
        });
    }

    Ok(SentenceBatch {
        title: batch.title.clone(),
        subtitle: batch.subtitle.clone(),
        sentences,
    })
}

fn word_tokens_only(tokens: Vec<SentenceToken>) -> Vec<SentenceToken> {
    tokens
        .into_iter()
        .filter(|token| token.kind.as_deref() == Some("word"))
        .collect()
}

fn parse_response(response_text: &str) -> Result<EnrichmentResponse, EnrichmentError> {
    let json = extract_json_object(response_text).ok_or(EnrichmentError::JsonNotFound)?;
    serde_json::from_str(json).map_err(EnrichmentError::Json)
}

fn extract_json_object(response_text: &str) -> Option<&str> {
    let bytes = response_text.as_bytes();
    let mut start = None;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (index, byte) in bytes.iter().copied().enumerate() {
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
                    let start = start?;
                    return response_text.get(start..=index);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{build_prompt, merge_enrichment};
    use crate::sentence_plan::{PlannedSentenceBatch, PlannedSentenceRow};
    use std::path::PathBuf;

    #[test]
    fn prompt_contains_only_source_row_payload() {
        let prompt = build_prompt("Template", &[row("0001")]);

        assert!(prompt.contains("\"hindi\""));
        assert!(!prompt.contains("\"source_ref\""));
    }

    #[test]
    fn extracts_fenced_json_and_merges_trusted_fields() {
        let batch = PlannedSentenceBatch {
            source_file: PathBuf::from("input/sentences/example.yaml"),
            title: Some("Title".to_string()),
            subtitle: Some("Chapter".to_string()),
            target_path: PathBuf::from("output/sentences/example_batch_01.json"),
            rows: vec![row("0001")],
        };
        let output = r#"Here:
```json
{"items":[{"id":"0001","literal":"here","register":"standard","tokens":[{"hindi":"यहाँ","roman":"yahā̃","kind":"word","word_id":"w1"}],"words":[{"id":"w1","hindi":"यहाँ","roman":"yahā̃","meaning":"here"}]}]}
```
"#;

        let merged = merge_enrichment(&batch, output).unwrap();

        assert_eq!(merged.title.as_deref(), Some("Title"));
        assert_eq!(merged.sentences[0].english.as_deref(), Some("Here."));
        assert_eq!(
            merged.sentences[0].source_ref.as_ref().unwrap().item_id,
            "0001"
        );
    }

    #[test]
    fn removes_non_word_tokens_from_model_output() {
        let batch = PlannedSentenceBatch {
            source_file: PathBuf::from("input/sentences/example.yaml"),
            title: Some("Title".to_string()),
            subtitle: Some("Chapter".to_string()),
            target_path: PathBuf::from("output/sentences/example_batch_01.json"),
            rows: vec![row("0001")],
        };
        let output = r#"{"items":[{"id":"0001","literal":"here","register":"standard","tokens":[{"hindi":"यहाँ","roman":"yahā̃","kind":"word","word_id":"w1"},{"hindi":"?","roman":"?","kind":"punct","word_id":"w2"}],"words":[{"id":"w1","hindi":"यहाँ","roman":"yahā̃","meaning":"here"}]}]}"#;

        let merged = merge_enrichment(&batch, output).unwrap();

        assert_eq!(merged.sentences[0].tokens.len(), 1);
        assert_eq!(merged.sentences[0].tokens[0].word_id.as_deref(), Some("w1"));
    }

    fn row(id: &str) -> PlannedSentenceRow {
        PlannedSentenceRow {
            id: id.to_string(),
            hindi: "यहाँ".to_string(),
            romanisation: "yahā̃".to_string(),
            english: "Here.".to_string(),
            tags: Vec::new(),
            fingerprint: "sha256:test".to_string(),
        }
    }
}
