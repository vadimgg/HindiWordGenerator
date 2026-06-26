use crate::error::{PromptAdapterError, strip_one_optional_fence};
use lingo_application::ports::{CardBatchDraft, CardDraft, PromptStage, TokenDraft, WordDraft};
use serde::Deserialize;

const FORMAT: &str = "lingo.build-reply/v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BuildReplyDto {
    format: String,
    cards: Vec<CardDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CardDto {
    source_item: String,
    literal: String,
    register: String,
    tokens: Vec<TokenDto>,
    words: Vec<WordDto>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TokenDto {
    target: String,
    romanisation: Option<String>,
    word_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WordDto {
    id: String,
    target: String,
    romanisation: Option<String>,
    meaning: String,
    kind: String,
    #[serde(default)]
    grammar: Vec<String>,
}

pub(crate) fn parse_build_reply(raw: &str) -> Result<CardBatchDraft, PromptAdapterError> {
    let body = strip_one_optional_fence(raw, PromptStage::Build)?;
    let dto: BuildReplyDto = serde_json::from_str(body).map_err(|error| {
        PromptAdapterError::invalid(
            PromptStage::Build,
            format!("line {} column {}", error.line(), error.column()),
            error,
        )
    })?;
    if dto.format != FORMAT {
        return Err(PromptAdapterError::invalid(
            PromptStage::Build,
            "format",
            format!("expected {FORMAT:?}, found {:?}", dto.format),
        ));
    }
    if dto.cards.is_empty() {
        return Err(PromptAdapterError::invalid(
            PromptStage::Build,
            "cards",
            "at least one card is required",
        ));
    }
    let cards = dto
        .cards
        .into_iter()
        .enumerate()
        .map(|(index, card)| convert_card(index, card))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CardBatchDraft { cards })
}

fn convert_card(index: usize, card: CardDto) -> Result<CardDraft, PromptAdapterError> {
    for (field, value) in [
        ("source_item", card.source_item.as_str()),
        ("literal", card.literal.as_str()),
        ("register", card.register.as_str()),
    ] {
        require_non_empty(format!("cards[{index}].{field}"), value)?;
    }
    if card.tokens.is_empty() || card.words.is_empty() {
        return Err(PromptAdapterError::invalid(
            PromptStage::Build,
            format!("cards[{index}]"),
            "tokens and words must both be non-empty",
        ));
    }
    let tokens = card
        .tokens
        .into_iter()
        .enumerate()
        .map(|(token_index, token)| {
            require_non_empty(
                format!("cards[{index}].tokens[{token_index}].target"),
                &token.target,
            )?;
            require_non_empty(
                format!("cards[{index}].tokens[{token_index}].word_id"),
                &token.word_id,
            )?;
            Ok(TokenDraft {
                target: token.target,
                romanisation: non_empty_optional(
                    format!("cards[{index}].tokens[{token_index}].romanisation"),
                    token.romanisation,
                )?,
                word_id: token.word_id,
            })
        })
        .collect::<Result<Vec<_>, PromptAdapterError>>()?;
    let words = card
        .words
        .into_iter()
        .enumerate()
        .map(|(word_index, word)| {
            for (field, value) in [
                ("id", word.id.as_str()),
                ("target", word.target.as_str()),
                ("meaning", word.meaning.as_str()),
                ("kind", word.kind.as_str()),
            ] {
                require_non_empty(format!("cards[{index}].words[{word_index}].{field}"), value)?;
            }
            Ok(WordDraft {
                id: word.id,
                target: word.target,
                romanisation: non_empty_optional(
                    format!("cards[{index}].words[{word_index}].romanisation"),
                    word.romanisation,
                )?,
                meaning: word.meaning,
                kind: word.kind,
                grammar: word.grammar,
            })
        })
        .collect::<Result<Vec<_>, PromptAdapterError>>()?;
    Ok(CardDraft {
        source_item: card.source_item,
        literal: card.literal,
        register: card.register,
        tokens,
        words,
        tags: card.tags,
    })
}

fn non_empty_optional(
    path: String,
    value: Option<String>,
) -> Result<Option<String>, PromptAdapterError> {
    if value
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err(PromptAdapterError::invalid(
            PromptStage::Build,
            path,
            "empty strings must be omitted, not used for absence",
        ));
    }
    Ok(value)
}

fn require_non_empty(path: String, value: &str) -> Result<(), PromptAdapterError> {
    if value.trim().is_empty() {
        Err(PromptAdapterError::invalid(
            PromptStage::Build,
            path,
            "value cannot be empty",
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::parse_build_reply;

    #[test]
    fn rejects_unknown_fields() {
        let reply = r#"{"format":"lingo.build-reply/v1","cards":[],"extra":true}"#;
        assert!(parse_build_reply(reply).is_err());
    }
}
