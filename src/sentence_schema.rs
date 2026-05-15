#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SentenceBatch {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    #[serde(default)]
    pub sentences: Vec<SentenceCard>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SentenceCard {
    pub hindi: Option<String>,
    pub romanisation: Option<String>,
    pub english: Option<String>,
    pub literal: Option<String>,
    pub register: Option<String>,
    pub source_ref: Option<SourceRef>,
    #[serde(default)]
    pub tokens: Vec<SentenceToken>,
    #[serde(default)]
    pub words: Vec<SentenceWord>,
    #[serde(default)]
    pub anki_tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRef {
    pub file: String,
    pub item_id: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SentenceToken {
    pub hindi: Option<String>,
    pub roman: Option<String>,
    pub kind: Option<String>,
    pub word_id: Option<String>,
    pub word_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SentenceWord {
    pub id: Option<String>,
    pub hindi: Option<String>,
    pub roman: Option<String>,
    pub meaning: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

pub fn parse_sentence_batch(json: &str) -> Result<SentenceBatch, serde_json::Error> {
    serde_json::from_str(json)
}

#[cfg(test)]
mod tests {
    use super::parse_sentence_batch;

    #[test]
    fn parses_candidate_sentence_batch() {
        let batch = parse_sentence_batch(
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
        .unwrap();

        assert_eq!(batch.sentences.len(), 1);
        assert_eq!(batch.sentences[0].tokens[0].word_id.as_deref(), Some("w1"));
    }

    #[test]
    fn invalid_json_returns_parse_error() {
        let error = parse_sentence_batch("{bad json").unwrap_err();

        assert!(error.to_string().contains("key"));
    }
}
