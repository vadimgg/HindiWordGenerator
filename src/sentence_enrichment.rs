use crate::sentence_plan::{PlannedSentenceBatch, PlannedSentenceRow};
use crate::sentence_schema::{SentenceBatch, SentenceCard, SentenceToken, SentenceWord, SourceRef};
use crate::source_identity::content_fingerprint;
use handlebars::Handlebars;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::BTreeMap;

pub const REGISTER_STAGE_ID: &str = "sentence/register";
pub const LITERAL_STAGE_ID: &str = "sentence/literal";
pub const WORD_BREAKDOWN_FROM_TRANSLATION_STAGE_ID: &str =
    "sentence/word-breakdown-from-translation";

#[derive(Debug)]
pub enum EnrichmentError {
    StructuredNotFound,
    StructuredParse { yaml: String, json: String },
    UnknownStage(String),
    TemplateRegistration(handlebars::TemplateError),
    Template(handlebars::RenderError),
    MissingStageItem { stage_id: String, item_id: String },
    DuplicateStageItem { stage_id: String, item_id: String },
    ExtraStageItem { stage_id: String, item_id: String },
}

impl std::fmt::Display for EnrichmentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnrichmentError::StructuredNotFound => {
                write!(formatter, "Model response did not contain YAML or JSON.")
            }
            EnrichmentError::StructuredParse { yaml, json } => write!(
                formatter,
                "Could not parse model response as YAML or JSON.\n\nYAML: {yaml}\nJSON: {json}"
            ),
            EnrichmentError::UnknownStage(stage_id) => {
                write!(formatter, "Unknown sentence generation stage {stage_id:?}.")
            }
            EnrichmentError::TemplateRegistration(error) => {
                write!(
                    formatter,
                    "Could not register sentence stage template: {error}"
                )
            }
            EnrichmentError::Template(error) => {
                write!(formatter, "Could not render sentence stage prompt: {error}")
            }
            EnrichmentError::MissingStageItem { stage_id, item_id } => {
                write!(formatter, "Stage {stage_id} did not return item {item_id}.")
            }
            EnrichmentError::DuplicateStageItem { stage_id, item_id } => write!(
                formatter,
                "Stage {stage_id} returned duplicate item {item_id}."
            ),
            EnrichmentError::ExtraStageItem { stage_id, item_id } => {
                write!(
                    formatter,
                    "Stage {stage_id} returned unexpected item {item_id}."
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

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagePromptDefinition {
    pub id: &'static str,
    pub version: &'static str,
    pub template: &'static str,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedStagePrompt {
    pub stage_id: String,
    pub version: String,
    pub fingerprint: String,
    pub prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RegisterStageRecord {
    pub id: String,
    pub register: String,
    #[serde(default)]
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LiteralStageRecord {
    pub id: String,
    pub literal: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct WordBreakdownStageRecord {
    pub id: String,
    #[serde(default)]
    pub words: Vec<SentenceWord>,
    #[serde(default)]
    pub anki_tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct StageResponse<T> {
    results: Vec<T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedEnrichment {
    pub register: Vec<RegisterStageRecord>,
    pub literal: Vec<LiteralStageRecord>,
    pub word_breakdown: Vec<WordBreakdownStageRecord>,
}

#[cfg(test)]
pub fn generation_stage_prompts() -> Vec<StagePromptDefinition> {
    generation_stage_prompt_templates()
        .into_iter()
        .map(|template| StagePromptDefinition {
            id: template.id,
            version: template.version,
            template: template.template,
            fingerprint: content_fingerprint(template.template.as_bytes()),
        })
        .collect()
}

pub fn render_stage_prompt(
    stage_id: &str,
    rows: &[PlannedSentenceRow],
) -> Result<RenderedStagePrompt, EnrichmentError> {
    let template = generation_stage_prompt_templates()
        .into_iter()
        .find(|template| template.id == stage_id)
        .ok_or_else(|| EnrichmentError::UnknownStage(stage_id.to_string()))?;
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
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string(stage_id, template.template)
        .map_err(EnrichmentError::TemplateRegistration)?;
    let prompt = handlebars
        .render(stage_id, &input)
        .map_err(EnrichmentError::Template)?;

    Ok(RenderedStagePrompt {
        stage_id: stage_id.to_string(),
        version: template.version.to_string(),
        fingerprint: content_fingerprint(template.template.as_bytes()),
        prompt,
    })
}

pub fn parse_register_stage(
    response_text: &str,
) -> Result<Vec<RegisterStageRecord>, EnrichmentError> {
    parse_stage_response::<RegisterStageRecord>(response_text)
}

pub fn parse_literal_stage(
    response_text: &str,
) -> Result<Vec<LiteralStageRecord>, EnrichmentError> {
    parse_stage_response::<LiteralStageRecord>(response_text)
}

pub fn parse_word_breakdown_stage(
    response_text: &str,
) -> Result<Vec<WordBreakdownStageRecord>, EnrichmentError> {
    parse_stage_response::<WordBreakdownStageRecord>(response_text)
}

pub fn merge_staged_enrichment(
    batch: &PlannedSentenceBatch,
    staged: StagedEnrichment,
) -> Result<SentenceBatch, EnrichmentError> {
    let mut register_by_id = stage_index(REGISTER_STAGE_ID, staged.register, |record| &record.id)?;
    let mut literal_by_id = stage_index(LITERAL_STAGE_ID, staged.literal, |record| &record.id)?;
    let mut word_by_id = stage_index(
        WORD_BREAKDOWN_FROM_TRANSLATION_STAGE_ID,
        staged.word_breakdown,
        |record| &record.id,
    )?;

    let mut sentences = Vec::new();
    for row in &batch.rows {
        let register = take_stage_item(REGISTER_STAGE_ID, &mut register_by_id, &row.id)?;
        let literal = take_stage_item(LITERAL_STAGE_ID, &mut literal_by_id, &row.id)?;
        let words = take_stage_item(
            WORD_BREAKDOWN_FROM_TRANSLATION_STAGE_ID,
            &mut word_by_id,
            &row.id,
        )?;
        let (tokens, words, anki_tags) = word_breakdown_to_tokens_and_words(words);
        sentences.push(SentenceCard {
            hindi: Some(row.hindi.clone()),
            romanisation: Some(row.romanisation.clone()),
            english: Some(row.english.clone()),
            literal: Some(literal.literal),
            register: Some(register.register),
            source_ref: Some(SourceRef {
                file: batch.source_file.to_string_lossy().to_string(),
                item_id: row.id.clone(),
                fingerprint: row.fingerprint.clone(),
            }),
            tokens,
            words,
            anki_tags,
            audio: None,
        });
    }

    reject_extra_stage_items(REGISTER_STAGE_ID, &register_by_id)?;
    reject_extra_stage_items(LITERAL_STAGE_ID, &literal_by_id)?;
    reject_extra_stage_items(WORD_BREAKDOWN_FROM_TRANSLATION_STAGE_ID, &word_by_id)?;

    Ok(SentenceBatch {
        title: batch.title.clone(),
        subtitle: batch.subtitle.clone(),
        sentences,
    })
}

#[derive(Debug, Clone, Copy)]
struct StagePromptTemplate {
    id: &'static str,
    version: &'static str,
    template: &'static str,
}

fn generation_stage_prompt_templates() -> Vec<StagePromptTemplate> {
    vec![
        StagePromptTemplate {
            id: REGISTER_STAGE_ID,
            version: "v3",
            template: include_str!("eval_prompts/sentence_register.yaml.hbs"),
        },
        StagePromptTemplate {
            id: LITERAL_STAGE_ID,
            version: "v3",
            template: include_str!("eval_prompts/sentence_literal.yaml.hbs"),
        },
        StagePromptTemplate {
            id: WORD_BREAKDOWN_FROM_TRANSLATION_STAGE_ID,
            version: "v3",
            template: include_str!(
                "eval_prompts/sentence_word_breakdown_from_translation.yaml.hbs"
            ),
        },
    ]
}

fn parse_stage_response<T>(response_text: &str) -> Result<Vec<T>, EnrichmentError>
where
    T: DeserializeOwned,
{
    let structured =
        extract_structured_response(response_text).ok_or(EnrichmentError::StructuredNotFound)?;
    let yaml = serde_yaml::from_str::<StageResponse<T>>(structured);
    match yaml {
        Ok(response) => Ok(response.results),
        Err(yaml_error) => serde_json::from_str::<StageResponse<T>>(structured)
            .map(|response| response.results)
            .map_err(|json_error| EnrichmentError::StructuredParse {
                yaml: yaml_error.to_string(),
                json: json_error.to_string(),
            }),
    }
}

fn extract_structured_response(response_text: &str) -> Option<&str> {
    if let Some(fenced) = extract_fenced_block(response_text) {
        return Some(fenced);
    }
    if let Some(json) = extract_json_object(response_text) {
        return Some(json);
    }
    let trimmed = response_text.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}

fn extract_fenced_block(response_text: &str) -> Option<&str> {
    let fence_start = response_text.find("```")?;
    let after_fence = response_text.get(fence_start + 3..)?;
    let fence_end = after_fence.find("```")?;
    let mut body = after_fence.get(..fence_end)?.trim();
    if let Some(newline) = body.find('\n') {
        let first_line = body[..newline].trim();
        if matches!(first_line, "json" | "yaml" | "yml") {
            body = body[newline + 1..].trim();
        }
    }
    Some(body)
}

fn stage_index<T, F>(
    stage_id: &str,
    records: Vec<T>,
    id_for: F,
) -> Result<BTreeMap<String, T>, EnrichmentError>
where
    F: Fn(&T) -> &str,
{
    let mut by_id = BTreeMap::new();
    for record in records {
        let id = id_for(&record).to_string();
        if by_id.contains_key(&id) {
            return Err(EnrichmentError::DuplicateStageItem {
                stage_id: stage_id.to_string(),
                item_id: id,
            });
        }
        by_id.insert(id, record);
    }
    Ok(by_id)
}

fn take_stage_item<T>(
    stage_id: &str,
    records: &mut BTreeMap<String, T>,
    item_id: &str,
) -> Result<T, EnrichmentError> {
    records
        .remove(item_id)
        .ok_or_else(|| EnrichmentError::MissingStageItem {
            stage_id: stage_id.to_string(),
            item_id: item_id.to_string(),
        })
}

fn reject_extra_stage_items<T>(
    stage_id: &str,
    records: &BTreeMap<String, T>,
) -> Result<(), EnrichmentError> {
    if let Some(item_id) = records.keys().next() {
        return Err(EnrichmentError::ExtraStageItem {
            stage_id: stage_id.to_string(),
            item_id: item_id.clone(),
        });
    }
    Ok(())
}

fn word_breakdown_to_tokens_and_words(
    breakdown: WordBreakdownStageRecord,
) -> (Vec<SentenceToken>, Vec<SentenceWord>, Vec<String>) {
    let mut tokens = Vec::new();
    let mut words = Vec::new();
    for (index, mut word) in breakdown
        .words
        .into_iter()
        .filter(|word| !is_punctuation_only_word(word))
        .enumerate()
    {
        let word_id = word.id.clone().unwrap_or_else(|| format!("w{}", index + 1));
        word.id = Some(word_id.clone());
        word.kind = word.kind.or_else(|| Some("word".to_string()));
        word.hindi = word.hindi.map(strip_attached_punctuation);
        word.roman = word.roman.map(strip_attached_punctuation);
        word.meaning = word.meaning.map(strip_attached_punctuation);
        tokens.push(SentenceToken {
            hindi: word.hindi.clone(),
            roman: word.roman.clone(),
            kind: Some("word".to_string()),
            word_id: Some(word_id),
            word_index: None,
        });
        words.push(word);
    }
    (tokens, words, breakdown.anki_tags)
}

fn strip_attached_punctuation(value: String) -> String {
    value
        .trim()
        .trim_matches(is_punctuation_char)
        .trim()
        .to_string()
}

fn is_punctuation_only_word(word: &SentenceWord) -> bool {
    let hindi = word.hindi.as_deref().unwrap_or_default().trim();
    let roman = word.roman.as_deref().unwrap_or_default().trim();
    !hindi.is_empty()
        && !roman.is_empty()
        && hindi.chars().all(is_punctuation_char)
        && roman.chars().all(is_punctuation_char)
}

fn is_punctuation_char(ch: char) -> bool {
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
                | '\''
        )
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
    use super::{
        generation_stage_prompts, merge_staged_enrichment, parse_literal_stage,
        parse_register_stage, parse_word_breakdown_stage, render_stage_prompt, EnrichmentError,
        LiteralStageRecord, RegisterStageRecord, StagedEnrichment, WordBreakdownStageRecord,
        LITERAL_STAGE_ID, REGISTER_STAGE_ID, WORD_BREAKDOWN_FROM_TRANSLATION_STAGE_ID,
    };
    use crate::sentence_plan::{PlannedSentenceBatch, PlannedSentenceRow};
    use crate::sentence_schema::SentenceWord;
    use std::path::PathBuf;

    #[test]
    fn generation_stage_registry_contains_default_stages() {
        let prompts = generation_stage_prompts();
        let ids = prompts.iter().map(|prompt| prompt.id).collect::<Vec<_>>();

        assert_eq!(
            ids,
            vec![
                REGISTER_STAGE_ID,
                LITERAL_STAGE_ID,
                WORD_BREAKDOWN_FROM_TRANSLATION_STAGE_ID
            ]
        );
        assert!(prompts.iter().all(|prompt| !prompt.fingerprint.is_empty()));
    }

    #[test]
    fn renders_stage_prompt_from_source_rows() {
        let prompt = render_stage_prompt(REGISTER_STAGE_ID, &[row("0001")]).unwrap();

        assert_eq!(prompt.stage_id, REGISTER_STAGE_ID);
        assert_eq!(prompt.version, "v3");
        assert!(prompt.prompt.contains("task: sentence_register_detection"));
        assert!(prompt.prompt.contains("id: \"0001\""));
        assert!(prompt.prompt.contains("romanisation: \"yahā̃\""));
        assert!(!prompt.prompt.contains("source_ref"));
    }

    #[test]
    fn parses_fenced_yaml_stage_responses() {
        let register = parse_register_stage(
            r#"```yaml
results:
  - id: "0001"
    register: standard
    rationale: "Neutral classroom sentence."
```"#,
        )
        .unwrap();
        let literal = parse_literal_stage(
            r#"results:
  - id: "0001"
    literal: "here"
"#,
        )
        .unwrap();
        let words = parse_word_breakdown_stage(
            r#"```yaml
results:
  - id: "0001"
    words:
      - hindi: "यहाँ"
        roman: "yahā̃"
        meaning: "here"
```"#,
        )
        .unwrap();

        assert_eq!(register[0].register, "standard");
        assert_eq!(literal[0].literal, "here");
        assert_eq!(words[0].words[0].meaning.as_deref(), Some("here"));
    }

    #[test]
    fn merges_staged_outputs_by_id_and_copies_source_fields() {
        let batch = batch(vec![row("0001")]);
        let staged = StagedEnrichment {
            register: vec![RegisterStageRecord {
                id: "0001".to_string(),
                register: "standard".to_string(),
                rationale: None,
            }],
            literal: vec![LiteralStageRecord {
                id: "0001".to_string(),
                literal: "here".to_string(),
            }],
            word_breakdown: vec![WordBreakdownStageRecord {
                id: "0001".to_string(),
                words: vec![SentenceWord {
                    id: None,
                    hindi: Some("यहाँ".to_string()),
                    roman: Some("yahā̃".to_string()),
                    meaning: Some("here".to_string()),
                    kind: None,
                    gender: None,
                    number: None,
                    note: None,
                }],
                anki_tags: vec!["chapter-02".to_string()],
            }],
        };

        let merged = merge_staged_enrichment(&batch, staged).unwrap();

        assert_eq!(merged.title.as_deref(), Some("Title"));
        assert_eq!(merged.sentences[0].english.as_deref(), Some("Here."));
        assert_eq!(merged.sentences[0].literal.as_deref(), Some("here"));
        assert_eq!(merged.sentences[0].register.as_deref(), Some("standard"));
        assert_eq!(merged.sentences[0].words[0].id.as_deref(), Some("w1"));
        assert_eq!(merged.sentences[0].tokens[0].word_id.as_deref(), Some("w1"));
        assert_eq!(
            merged.sentences[0].source_ref.as_ref().unwrap().file,
            "input/sentences/example.yaml"
        );
    }

    #[test]
    fn staged_merge_drops_punctuation_only_word_entries() {
        let batch = batch(vec![row("0001")]);
        let staged = StagedEnrichment {
            register: vec![register("0001")],
            literal: vec![literal("0001")],
            word_breakdown: vec![WordBreakdownStageRecord {
                id: "0001".to_string(),
                words: vec![
                    SentenceWord {
                        id: None,
                        hindi: Some("यहाँ".to_string()),
                        roman: Some("yahā̃".to_string()),
                        meaning: Some("here".to_string()),
                        kind: None,
                        gender: None,
                        number: None,
                        note: None,
                    },
                    SentenceWord {
                        id: None,
                        hindi: Some("?".to_string()),
                        roman: Some("?".to_string()),
                        meaning: Some("question mark".to_string()),
                        kind: None,
                        gender: None,
                        number: None,
                        note: None,
                    },
                ],
                anki_tags: Vec::new(),
            }],
        };

        let merged = merge_staged_enrichment(&batch, staged).unwrap();

        assert_eq!(merged.sentences[0].words.len(), 1);
        assert_eq!(merged.sentences[0].tokens.len(), 1);
        assert_eq!(merged.sentences[0].words[0].hindi.as_deref(), Some("यहाँ"));
    }

    #[test]
    fn staged_merge_strips_attached_punctuation_from_word_fields() {
        let mut source_row = row("0001");
        source_row.hindi = "है".to_string();
        source_row.romanisation = "hai?".to_string();
        source_row.english = "Is it?".to_string();
        let batch = batch(vec![source_row]);
        let staged = StagedEnrichment {
            register: vec![register("0001")],
            literal: vec![literal("0001")],
            word_breakdown: vec![WordBreakdownStageRecord {
                id: "0001".to_string(),
                words: vec![SentenceWord {
                    id: None,
                    hindi: Some("है?".to_string()),
                    roman: Some("hai?".to_string()),
                    meaning: Some("is?".to_string()),
                    kind: None,
                    gender: None,
                    number: None,
                    note: None,
                }],
                anki_tags: Vec::new(),
            }],
        };

        let merged = merge_staged_enrichment(&batch, staged).unwrap();

        assert_eq!(merged.sentences[0].tokens[0].hindi.as_deref(), Some("है"));
        assert_eq!(merged.sentences[0].tokens[0].roman.as_deref(), Some("hai"));
        assert_eq!(merged.sentences[0].words[0].meaning.as_deref(), Some("is"));
    }

    #[test]
    fn staged_merge_rejects_missing_duplicate_and_extra_ids() {
        let batch = batch(vec![row("0001")]);
        let missing = merge_staged_enrichment(
            &batch,
            StagedEnrichment {
                register: Vec::new(),
                literal: vec![literal("0001")],
                word_breakdown: vec![words("0001")],
            },
        )
        .unwrap_err();
        assert!(matches!(missing, EnrichmentError::MissingStageItem { .. }));

        let duplicate = merge_staged_enrichment(
            &batch,
            StagedEnrichment {
                register: vec![register("0001"), register("0001")],
                literal: vec![literal("0001")],
                word_breakdown: vec![words("0001")],
            },
        )
        .unwrap_err();
        assert!(matches!(
            duplicate,
            EnrichmentError::DuplicateStageItem { .. }
        ));

        let extra = merge_staged_enrichment(
            &batch,
            StagedEnrichment {
                register: vec![register("0001"), register("0002")],
                literal: vec![literal("0001")],
                word_breakdown: vec![words("0001")],
            },
        )
        .unwrap_err();
        assert!(matches!(extra, EnrichmentError::ExtraStageItem { .. }));
    }

    fn batch(rows: Vec<PlannedSentenceRow>) -> PlannedSentenceBatch {
        PlannedSentenceBatch {
            source_file: PathBuf::from("input/sentences/example.yaml"),
            title: Some("Title".to_string()),
            subtitle: Some("Chapter".to_string()),
            target_path: PathBuf::from("output/sentences/example_batch_01.json"),
            rows,
        }
    }

    fn register(id: &str) -> RegisterStageRecord {
        RegisterStageRecord {
            id: id.to_string(),
            register: "standard".to_string(),
            rationale: None,
        }
    }

    fn literal(id: &str) -> LiteralStageRecord {
        LiteralStageRecord {
            id: id.to_string(),
            literal: "here".to_string(),
        }
    }

    fn words(id: &str) -> WordBreakdownStageRecord {
        WordBreakdownStageRecord {
            id: id.to_string(),
            words: vec![SentenceWord {
                id: None,
                hindi: Some("यहाँ".to_string()),
                roman: Some("yahā̃".to_string()),
                meaning: Some("here".to_string()),
                kind: None,
                gender: None,
                number: None,
                note: None,
            }],
            anki_tags: Vec::new(),
        }
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
