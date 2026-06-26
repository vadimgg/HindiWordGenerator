use lingo_domain::{
    AudioRef, BatchId, Card, CardBatch, CardId, CardTags, CardToken, Gloss, GrammarTag, Register,
    Romanisation, SourceBatch, SourceFingerprint, SourceItem, SourceItemId, SourceRef,
    SourceSubtitle, SourceTags, SourceTextParts, SourceTitle, TargetText, Word, WordId, WordKind,
    source_fingerprint,
};
use serde::Deserialize;
use std::path::PathBuf;
use thiserror::Error;

const SOURCE_FORMAT: &str = "lingo.source/v1";
const CARD_FORMAT: &str = "lingo.cards/v1";

#[derive(Debug, Error)]
pub enum CodecError {
    #[error("YAML decoding failed: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("JSON decoding failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported format {actual:?}; expected {expected}")]
    UnsupportedFormat {
        expected: &'static str,
        actual: String,
    },
    #[error("canonical data is invalid: {0}")]
    Invalid(String),
}

#[derive(Deserialize)]
struct SourceBatchFileDto {
    format: String,
    batch: String,
    title: String,
    subtitle: Option<String>,
    items: Vec<SourceItemFileDto>,
}

#[derive(Deserialize)]
struct SourceItemFileDto {
    id: String,
    target: String,
    romanisation: Option<String>,
    english: String,
    #[serde(default)]
    tags: Vec<String>,
    fingerprint: String,
}

pub fn encode_source(source: &SourceBatch) -> Result<Vec<u8>, CodecError> {
    let mut text = serde_yaml::to_string(source)?;
    if !text.ends_with('\n') {
        text.push('\n');
    }
    Ok(text.into_bytes())
}

pub fn decode_source(bytes: &[u8]) -> Result<SourceBatch, CodecError> {
    let dto: SourceBatchFileDto = serde_yaml::from_slice(bytes)?;
    require_format(&dto.format, SOURCE_FORMAT)?;
    let items = dto
        .items
        .into_iter()
        .map(source_item_from_dto)
        .collect::<Result<Vec<_>, _>>()?;
    SourceBatch::try_new(
        BatchId::parse(dto.batch).map_err(invalid)?,
        SourceTitle::parse(dto.title).map_err(invalid)?,
        dto.subtitle
            .map(SourceSubtitle::parse)
            .transpose()
            .map_err(invalid)?,
        items,
    )
    .map_err(invalid)
}

fn source_item_from_dto(dto: SourceItemFileDto) -> Result<SourceItem, CodecError> {
    let target = TargetText::parse(dto.target).map_err(invalid)?;
    let romanisation = dto
        .romanisation
        .map(Romanisation::parse)
        .transpose()
        .map_err(invalid)?;
    let english = Gloss::parse(dto.english).map_err(invalid)?;
    let stored_fingerprint = SourceFingerprint::parse(dto.fingerprint).map_err(invalid)?;
    let computed_fingerprint = source_fingerprint(&SourceTextParts {
        target: target.as_str(),
        romanisation: romanisation.as_ref().map(Romanisation::as_str),
        english: english.as_str(),
    });
    if stored_fingerprint != computed_fingerprint {
        return Err(CodecError::Invalid(format!(
            "source item {} fingerprint is stale: expected {}, found {}",
            dto.id, computed_fingerprint, stored_fingerprint
        )));
    }
    Ok(SourceItem::new(
        SourceItemId::parse(dto.id).map_err(invalid)?,
        target,
        romanisation,
        english,
        SourceTags::try_from_values(dto.tags).map_err(invalid)?,
        stored_fingerprint,
    ))
}

#[derive(Deserialize)]
struct CardBatchFileDto {
    format: String,
    batch: String,
    title: String,
    subtitle: Option<String>,
    cards: Vec<CardFileDto>,
}

#[derive(Deserialize)]
struct CardFileDto {
    id: String,
    target: String,
    romanisation: Option<String>,
    english: String,
    literal: String,
    register: String,
    tokens: Vec<CardTokenFileDto>,
    words: Vec<WordFileDto>,
    #[serde(default)]
    tags: Vec<String>,
    audio: Option<AudioRef>,
    source: SourceRefFileDto,
}

#[derive(Deserialize)]
struct CardTokenFileDto {
    target: String,
    romanisation: Option<String>,
    word_id: String,
}

#[derive(Deserialize)]
struct WordFileDto {
    id: String,
    target: String,
    romanisation: Option<String>,
    meaning: String,
    kind: String,
    #[serde(default)]
    grammar: Vec<String>,
}

#[derive(Deserialize)]
struct SourceRefFileDto {
    batch: String,
    item: String,
    fingerprint: String,
}

pub fn encode_cards(cards: &CardBatch) -> Result<Vec<u8>, CodecError> {
    let mut bytes = serde_json::to_vec_pretty(cards)?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub fn decode_cards(bytes: &[u8]) -> Result<CardBatch, CodecError> {
    let dto: CardBatchFileDto = serde_json::from_slice(bytes)?;
    require_format(&dto.format, CARD_FORMAT)?;
    let batch = BatchId::parse(dto.batch).map_err(invalid)?;
    let cards = dto
        .cards
        .into_iter()
        .map(card_from_dto)
        .collect::<Result<Vec<_>, _>>()?;
    CardBatch::try_new(
        batch,
        SourceTitle::parse(dto.title).map_err(invalid)?,
        dto.subtitle
            .map(SourceSubtitle::parse)
            .transpose()
            .map_err(invalid)?,
        cards,
    )
    .map_err(invalid)
}

fn card_from_dto(dto: CardFileDto) -> Result<Card, CodecError> {
    let id = CardId::parse(&dto.id).map_err(invalid)?;
    let source = SourceRef::new(
        BatchId::parse(dto.source.batch).map_err(invalid)?,
        SourceItemId::parse(dto.source.item).map_err(invalid)?,
        SourceFingerprint::parse(dto.source.fingerprint).map_err(invalid)?,
    );
    let tokens = dto
        .tokens
        .into_iter()
        .map(|token| {
            Ok(CardToken::new(
                TargetText::parse(token.target).map_err(invalid)?,
                token
                    .romanisation
                    .map(Romanisation::parse)
                    .transpose()
                    .map_err(invalid)?,
                WordId::parse(token.word_id).map_err(invalid)?,
            ))
        })
        .collect::<Result<Vec<_>, CodecError>>()?;
    let words = dto
        .words
        .into_iter()
        .map(word_from_dto)
        .collect::<Result<Vec<_>, _>>()?;
    let mut card = Card::try_new(
        id,
        TargetText::parse(dto.target).map_err(invalid)?,
        dto.romanisation
            .map(Romanisation::parse)
            .transpose()
            .map_err(invalid)?,
        Gloss::parse(dto.english).map_err(invalid)?,
        Gloss::parse(dto.literal).map_err(invalid)?,
        Register::parse(&dto.register).map_err(invalid)?,
        tokens,
        words,
        CardTags::try_from_values(dto.tags).map_err(invalid)?,
        source,
    )
    .map_err(invalid)?;
    if let Some(audio) = dto.audio {
        card.attach_audio(audio).map_err(invalid)?;
    }
    Ok(card)
}

fn word_from_dto(dto: WordFileDto) -> Result<Word, CodecError> {
    let grammar = dto
        .grammar
        .iter()
        .map(|tag| GrammarTag::parse(tag).map_err(invalid))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Word::new(
        WordId::parse(dto.id).map_err(invalid)?,
        TargetText::parse(dto.target).map_err(invalid)?,
        dto.romanisation
            .map(Romanisation::parse)
            .transpose()
            .map_err(invalid)?,
        Gloss::parse(dto.meaning).map_err(invalid)?,
        WordKind::parse(&dto.kind).map_err(invalid)?,
        grammar,
    ))
}

fn require_format(actual: &str, expected: &'static str) -> Result<(), CodecError> {
    if actual == expected {
        Ok(())
    } else {
        Err(CodecError::UnsupportedFormat {
            expected,
            actual: actual.to_string(),
        })
    }
}

fn invalid(error: impl std::fmt::Display) -> CodecError {
    CodecError::Invalid(error.to_string())
}

pub fn map_codec_path(
    path: PathBuf,
    error: CodecError,
) -> lingo_application::ports::WorkspaceFailure {
    lingo_application::ports::WorkspaceFailure::InvalidData(format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::{decode_source, encode_source};
    use lingo_domain::{
        BatchId, Gloss, SourceBatch, SourceItem, SourceItemId, SourceTags, SourceTextParts,
        SourceTitle, TargetText, source_fingerprint,
    };

    #[test]
    fn source_round_trips() {
        let source = SourceBatch::try_new(
            BatchId::parse("chapter-01").unwrap(),
            SourceTitle::parse("Chapter 01").unwrap(),
            None,
            vec![SourceItem::new(
                SourceItemId::parse("s-1234567890abcdef-01").unwrap(),
                TargetText::parse("यहाँ").unwrap(),
                None,
                Gloss::parse("Here").unwrap(),
                SourceTags::default(),
                source_fingerprint(&SourceTextParts {
                    target: "यहाँ",
                    romanisation: None,
                    english: "Here",
                }),
            )],
        )
        .unwrap();
        let decoded = decode_source(&encode_source(&source).unwrap()).unwrap();
        assert_eq!(decoded, source);
    }

    #[test]
    fn source_decode_rejects_stale_fingerprint() {
        let source = SourceBatch::try_new(
            BatchId::parse("chapter-01").unwrap(),
            SourceTitle::parse("Chapter 01").unwrap(),
            None,
            vec![SourceItem::new(
                SourceItemId::parse("s-1234567890abcdef-01").unwrap(),
                TargetText::parse("यहाँ").unwrap(),
                None,
                Gloss::parse("Here").unwrap(),
                SourceTags::default(),
                source_fingerprint(&SourceTextParts {
                    target: "यहाँ",
                    romanisation: None,
                    english: "Here",
                }),
            )],
        )
        .unwrap();
        let mut yaml = String::from_utf8(encode_source(&source).unwrap()).unwrap();
        yaml = yaml.replace("Here", "There");

        let error = decode_source(yaml.as_bytes()).unwrap_err();
        assert!(error.to_string().contains("fingerprint is stale"));
    }
}
