use crate::{
    AudioRef, BatchId, CardId, Gloss, Romanisation, SourceFingerprint, SourceItemId,
    SourceSubtitle, SourceTitle, TargetText, WordId,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum CardFormat {
    #[serde(rename = "lingo.cards/v1")]
    V1,
}

impl CardFormat {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::V1 => "lingo.cards/v1",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Register {
    Informal,
    Standard,
    Formal,
}

impl Register {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Informal => "informal",
            Self::Standard => "standard",
            Self::Formal => "formal",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, CardError> {
        match raw {
            "informal" => Ok(Self::Informal),
            "standard" => Ok(Self::Standard),
            "formal" => Ok(Self::Formal),
            other => Err(CardError::UnknownRegister(other.to_string())),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrammarTag {
    Masculine,
    Feminine,
    Singular,
    Plural,
    Formal,
    Informal,
    Intimate,
    Present,
    Past,
    Future,
    Habitual,
    Perfective,
    Imperfective,
    Subjunctive,
}

impl GrammarTag {
    pub fn parse(raw: &str) -> Result<Self, CardError> {
        match raw {
            "masculine" => Ok(Self::Masculine),
            "feminine" => Ok(Self::Feminine),
            "singular" => Ok(Self::Singular),
            "plural" => Ok(Self::Plural),
            "formal" => Ok(Self::Formal),
            "informal" => Ok(Self::Informal),
            "intimate" => Ok(Self::Intimate),
            "present" => Ok(Self::Present),
            "past" => Ok(Self::Past),
            "future" => Ok(Self::Future),
            "habitual" => Ok(Self::Habitual),
            "perfective" => Ok(Self::Perfective),
            "imperfective" => Ok(Self::Imperfective),
            "subjunctive" => Ok(Self::Subjunctive),
            other => Err(CardError::UnknownGrammarTag(other.to_string())),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WordKind {
    Noun,
    Verb,
    Adjective,
    Adverb,
    Pronoun,
    Postposition,
    Particle,
    Conjunction,
    Interjection,
    Numeral,
    ProperNoun,
    Other,
}

impl WordKind {
    pub fn parse(raw: &str) -> Result<Self, CardError> {
        match raw {
            "noun" => Ok(Self::Noun),
            "verb" => Ok(Self::Verb),
            "adjective" => Ok(Self::Adjective),
            "adverb" => Ok(Self::Adverb),
            "pronoun" => Ok(Self::Pronoun),
            "postposition" => Ok(Self::Postposition),
            "particle" => Ok(Self::Particle),
            "conjunction" => Ok(Self::Conjunction),
            "interjection" => Ok(Self::Interjection),
            "numeral" => Ok(Self::Numeral),
            "proper_noun" => Ok(Self::ProperNoun),
            "other" => Ok(Self::Other),
            other => Err(CardError::UnknownWordKind(other.to_string())),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct CardTags(Vec<String>);

impl CardTags {
    pub fn try_from_values(values: impl IntoIterator<Item = String>) -> Result<Self, CardError> {
        let mut tags = BTreeSet::new();
        for value in values {
            let tag = value.trim().to_ascii_lowercase();
            if tag.is_empty()
                || tag.len() > 80
                || !tag.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'-' | b'_')
                })
            {
                return Err(CardError::InvalidTag(value));
            }
            tags.insert(tag);
        }
        Ok(Self(tags.into_iter().collect()))
    }

    pub fn values(&self) -> &[String] {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SourceRef {
    batch: BatchId,
    item: SourceItemId,
    fingerprint: SourceFingerprint,
}

impl SourceRef {
    pub fn new(batch: BatchId, item: SourceItemId, fingerprint: SourceFingerprint) -> Self {
        Self {
            batch,
            item,
            fingerprint,
        }
    }

    pub fn batch(&self) -> &BatchId {
        &self.batch
    }

    pub fn item(&self) -> &SourceItemId {
        &self.item
    }

    pub fn fingerprint(&self) -> &SourceFingerprint {
        &self.fingerprint
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Word {
    id: WordId,
    target: TargetText,
    #[serde(skip_serializing_if = "Option::is_none")]
    romanisation: Option<Romanisation>,
    meaning: Gloss,
    kind: WordKind,
    grammar: Vec<GrammarTag>,
}

impl Word {
    pub fn new(
        id: WordId,
        target: TargetText,
        romanisation: Option<Romanisation>,
        meaning: Gloss,
        kind: WordKind,
        grammar: impl IntoIterator<Item = GrammarTag>,
    ) -> Self {
        let grammar = grammar
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        Self {
            id,
            target,
            romanisation,
            meaning,
            kind,
            grammar,
        }
    }

    pub fn id(&self) -> &WordId {
        &self.id
    }

    pub fn target(&self) -> &TargetText {
        &self.target
    }

    pub fn romanisation(&self) -> Option<&Romanisation> {
        self.romanisation.as_ref()
    }

    pub fn meaning(&self) -> &Gloss {
        &self.meaning
    }

    pub const fn kind(&self) -> WordKind {
        self.kind
    }

    pub fn grammar(&self) -> &[GrammarTag] {
        &self.grammar
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CardToken {
    target: TargetText,
    #[serde(skip_serializing_if = "Option::is_none")]
    romanisation: Option<Romanisation>,
    word_id: WordId,
}

impl CardToken {
    pub fn new(target: TargetText, romanisation: Option<Romanisation>, word_id: WordId) -> Self {
        Self {
            target,
            romanisation,
            word_id,
        }
    }

    pub fn target(&self) -> &TargetText {
        &self.target
    }

    pub fn romanisation(&self) -> Option<&Romanisation> {
        self.romanisation.as_ref()
    }

    pub fn word_id(&self) -> &WordId {
        &self.word_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Card {
    id: CardId,
    target: TargetText,
    #[serde(skip_serializing_if = "Option::is_none")]
    romanisation: Option<Romanisation>,
    english: Gloss,
    literal: Gloss,
    register: Register,
    tokens: Vec<CardToken>,
    words: Vec<Word>,
    tags: CardTags,
    #[serde(skip_serializing_if = "Option::is_none")]
    audio: Option<AudioRef>,
    source: SourceRef,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CardError {
    #[error("card must contain at least one word")]
    EmptyWords,
    #[error("card must contain at least one token")]
    EmptyTokens,
    #[error("duplicate word id: {0}")]
    DuplicateWord(WordId),
    #[error("token references unknown word id: {0}")]
    UnknownTokenWord(WordId),
    #[error("word is not referenced by any token: {0}")]
    UnusedWord(WordId),
    #[error("card/source identity mismatch")]
    SourceIdentityMismatch,
    #[error("audio belongs to another card")]
    WrongAudioOwner,
    #[error("card batch must contain at least one card")]
    EmptyBatch,
    #[error("duplicate card id: {0}")]
    DuplicateCard(CardId),
    #[error("card belongs to another batch: {0}")]
    WrongBatch(CardId),
    #[error("unknown register: {0:?}")]
    UnknownRegister(String),
    #[error("unknown grammar tag: {0:?}")]
    UnknownGrammarTag(String),
    #[error("unknown word kind: {0:?}")]
    UnknownWordKind(String),
    #[error("invalid card tag: {0:?}")]
    InvalidTag(String),
}

impl Card {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        id: CardId,
        target: TargetText,
        romanisation: Option<Romanisation>,
        english: Gloss,
        literal: Gloss,
        register: Register,
        tokens: Vec<CardToken>,
        words: Vec<Word>,
        tags: CardTags,
        source: SourceRef,
    ) -> Result<Self, CardError> {
        if words.is_empty() {
            return Err(CardError::EmptyWords);
        }
        if tokens.is_empty() {
            return Err(CardError::EmptyTokens);
        }
        if id.batch() != source.batch() || id.source_item() != source.item() {
            return Err(CardError::SourceIdentityMismatch);
        }
        let mut word_ids = BTreeSet::new();
        for word in &words {
            if !word_ids.insert(word.id().clone()) {
                return Err(CardError::DuplicateWord(word.id().clone()));
            }
        }
        for token in &tokens {
            if !word_ids.contains(token.word_id()) {
                return Err(CardError::UnknownTokenWord(token.word_id().clone()));
            }
        }
        let referenced = tokens
            .iter()
            .map(|token| token.word_id().clone())
            .collect::<BTreeSet<_>>();
        for word in &words {
            if !referenced.contains(word.id()) {
                return Err(CardError::UnusedWord(word.id().clone()));
            }
        }
        Ok(Self {
            id,
            target,
            romanisation,
            english,
            literal,
            register,
            tokens,
            words,
            tags,
            audio: None,
            source,
        })
    }

    pub fn id(&self) -> &CardId {
        &self.id
    }

    pub fn target(&self) -> &TargetText {
        &self.target
    }

    pub fn romanisation(&self) -> Option<&Romanisation> {
        self.romanisation.as_ref()
    }

    pub fn english(&self) -> &Gloss {
        &self.english
    }

    pub fn literal(&self) -> &Gloss {
        &self.literal
    }

    pub const fn register(&self) -> Register {
        self.register
    }

    pub fn tokens(&self) -> &[CardToken] {
        &self.tokens
    }

    pub fn words(&self) -> &[Word] {
        &self.words
    }

    pub fn tags(&self) -> &CardTags {
        &self.tags
    }

    pub fn audio(&self) -> Option<&AudioRef> {
        self.audio.as_ref()
    }

    pub fn source(&self) -> &SourceRef {
        &self.source
    }

    pub fn attach_audio(&mut self, audio: AudioRef) -> Result<(), CardError> {
        if audio.card_id() != &self.id {
            return Err(CardError::WrongAudioOwner);
        }
        self.audio = Some(audio);
        Ok(())
    }

    pub fn clear_audio(&mut self) {
        self.audio = None;
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CardBatch {
    format: CardFormat,
    batch: BatchId,
    title: SourceTitle,
    #[serde(skip_serializing_if = "Option::is_none")]
    subtitle: Option<SourceSubtitle>,
    cards: Vec<Card>,
}

impl CardBatch {
    pub fn try_new(
        batch: BatchId,
        title: SourceTitle,
        subtitle: Option<SourceSubtitle>,
        cards: Vec<Card>,
    ) -> Result<Self, CardError> {
        if cards.is_empty() {
            return Err(CardError::EmptyBatch);
        }
        let mut ids = BTreeMap::new();
        for card in &cards {
            if card.id().batch() != &batch {
                return Err(CardError::WrongBatch(card.id().clone()));
            }
            if ids.insert(card.id().clone(), ()).is_some() {
                return Err(CardError::DuplicateCard(card.id().clone()));
            }
        }
        Ok(Self {
            format: CardFormat::V1,
            batch,
            title,
            subtitle,
            cards,
        })
    }

    pub const fn format(&self) -> CardFormat {
        self.format
    }

    pub fn batch_id(&self) -> &BatchId {
        &self.batch
    }

    pub fn title(&self) -> &SourceTitle {
        &self.title
    }

    pub fn subtitle(&self) -> Option<&SourceSubtitle> {
        self.subtitle.as_ref()
    }

    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    pub fn cards_mut(&mut self) -> &mut [Card] {
        &mut self.cards
    }
}

#[cfg(test)]
mod tests {
    use super::{Card, CardTags, CardToken, SourceRef, Word, WordKind};
    use crate::{BatchId, CardId, Gloss, SourceFingerprint, SourceItemId, TargetText, WordId};

    #[test]
    fn token_must_reference_word() {
        let batch = BatchId::parse("chapter-01").unwrap();
        let item = SourceItemId::parse("s-1234567890abcdef-01").unwrap();
        let id = CardId::new(batch.clone(), item.clone());
        let source = SourceRef::new(
            batch,
            item,
            SourceFingerprint::parse(
                "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .unwrap(),
        );
        let word = Word::new(
            WordId::parse("w1").unwrap(),
            TargetText::parse("यह").unwrap(),
            None,
            Gloss::parse("this").unwrap(),
            WordKind::Pronoun,
            [],
        );
        let token = CardToken::new(
            TargetText::parse("यह").unwrap(),
            None,
            WordId::parse("w2").unwrap(),
        );
        assert!(
            Card::try_new(
                id,
                TargetText::parse("यह").unwrap(),
                None,
                Gloss::parse("this").unwrap(),
                Gloss::parse("this").unwrap(),
                super::Register::Standard,
                vec![token],
                vec![word],
                CardTags::default(),
                source,
            )
            .is_err()
        );
    }

    #[test]
    fn every_word_must_be_referenced() {
        let batch = BatchId::parse("chapter-01").unwrap();
        let item = SourceItemId::parse("s-1234567890abcdef-01").unwrap();
        let id = CardId::new(batch.clone(), item.clone());
        let source = SourceRef::new(
            batch,
            item,
            SourceFingerprint::parse(
                "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .unwrap(),
        );
        let words = vec![
            Word::new(
                WordId::parse("w1").unwrap(),
                TargetText::parse("यह").unwrap(),
                None,
                Gloss::parse("this").unwrap(),
                WordKind::Pronoun,
                [],
            ),
            Word::new(
                WordId::parse("w2").unwrap(),
                TargetText::parse("किताब").unwrap(),
                None,
                Gloss::parse("book").unwrap(),
                WordKind::Noun,
                [],
            ),
        ];
        let tokens = vec![CardToken::new(
            TargetText::parse("यह").unwrap(),
            None,
            WordId::parse("w1").unwrap(),
        )];
        assert!(matches!(
            Card::try_new(
                id,
                TargetText::parse("यह").unwrap(),
                None,
                Gloss::parse("this").unwrap(),
                Gloss::parse("this").unwrap(),
                super::Register::Standard,
                tokens,
                words,
                CardTags::default(),
                source,
            ),
            Err(super::CardError::UnusedWord(_))
        ));
    }
}
