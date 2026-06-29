use crate::{CollectionId, Gloss, Romanisation, SentenceId, TargetText, WordId, normalize_text};
use serde::{Deserialize, Serialize};
use std::fmt;
use unicode_normalization::UnicodeNormalization;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WordKey(String);

impl WordKey {
    pub fn from_surface(surface: &TargetText) -> Self {
        let normalized = surface
            .as_str()
            .nfc()
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        Self(normalized)
    }

    pub fn from_meaning(meaning: &str) -> Self {
        Self(normalize_text(meaning).to_lowercase())
    }

    pub fn parse(raw: impl Into<String>) -> Self {
        Self(normalize_text(raw.into()).to_lowercase())
    }

    pub fn as_str(&self) -> &str { &self.0 }
}

impl fmt::Display for WordKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WordEntry {
    id: WordId,
    collection: CollectionId,
    key: WordKey,
    form: TargetText,
    roman: Option<Romanisation>,
    kind: Option<String>,
}

impl WordEntry {
    pub fn new(
        id: WordId,
        collection: CollectionId,
        key: WordKey,
        form: TargetText,
        roman: Option<Romanisation>,
        kind: Option<String>,
    ) -> Self {
        Self { id, collection, key, form, roman, kind }
    }
    pub fn id(&self) -> &WordId { &self.id }
    pub fn collection(&self) -> &CollectionId { &self.collection }
    pub fn key(&self) -> &WordKey { &self.key }
    pub fn form(&self) -> &TargetText { &self.form }
    pub fn roman(&self) -> Option<&Romanisation> { self.roman.as_ref() }
    pub fn kind(&self) -> Option<&str> { self.kind.as_deref() }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WordMeaning {
    meaning: Gloss,
    first_seen_sentence: Option<SentenceId>,
}

impl WordMeaning {
    pub fn new(meaning: Gloss, first_seen_sentence: Option<SentenceId>) -> Self {
        Self { meaning, first_seen_sentence }
    }
    pub fn meaning(&self) -> &Gloss { &self.meaning }
    pub fn first_seen_sentence(&self) -> Option<&SentenceId> { self.first_seen_sentence.as_ref() }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WordOccurrence {
    sentence: SentenceId,
    position: usize,
    surface_form: TargetText,
    gloss: Option<Gloss>,
}

impl WordOccurrence {
    pub fn new(sentence: SentenceId, position: usize, surface_form: TargetText, gloss: Option<Gloss>) -> Self {
        Self { sentence, position, surface_form, gloss }
    }
    pub fn sentence(&self) -> &SentenceId { &self.sentence }
    pub const fn position(&self) -> usize { self.position }
    pub fn surface_form(&self) -> &TargetText { &self.surface_form }
    pub fn gloss(&self) -> Option<&Gloss> { self.gloss.as_ref() }
}

#[cfg(test)]
mod tests {
    use super::WordKey;
    use crate::TargetText;

    #[test]
    fn normalizes_surface_identity() {
        let key = WordKey::from_surface(&TargetText::parse("  जी  ").unwrap());
        assert_eq!(key.as_str(), "जी");
    }
}
