use crate::{
    BatchId, Gloss, Romanisation, SourceFingerprint, SourceItemId, SourceSubtitle, SourceTitle,
    TargetText,
};
use serde::Serialize;
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum SourceFormat {
    #[serde(rename = "lingo.source/v1")]
    V1,
}

impl SourceFormat {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::V1 => "lingo.source/v1",
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SourceTags(Vec<String>);

impl SourceTags {
    pub fn try_from_values(values: impl IntoIterator<Item = String>) -> Result<Self, SourceError> {
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
                return Err(SourceError::InvalidTag(value));
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
pub struct SourceItem {
    id: SourceItemId,
    target: TargetText,
    #[serde(skip_serializing_if = "Option::is_none")]
    romanisation: Option<Romanisation>,
    english: Gloss,
    tags: SourceTags,
    fingerprint: SourceFingerprint,
}

impl SourceItem {
    pub fn new(
        id: SourceItemId,
        target: TargetText,
        romanisation: Option<Romanisation>,
        english: Gloss,
        tags: SourceTags,
        fingerprint: SourceFingerprint,
    ) -> Self {
        Self {
            id,
            target,
            romanisation,
            english,
            tags,
            fingerprint,
        }
    }

    pub fn id(&self) -> &SourceItemId {
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

    pub fn tags(&self) -> &SourceTags {
        &self.tags
    }

    pub fn fingerprint(&self) -> &SourceFingerprint {
        &self.fingerprint
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SourceBatch {
    format: SourceFormat,
    batch: BatchId,
    title: SourceTitle,
    #[serde(skip_serializing_if = "Option::is_none")]
    subtitle: Option<SourceSubtitle>,
    items: Vec<SourceItem>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SourceError {
    #[error("source batch must contain at least one item")]
    EmptyBatch,
    #[error("duplicate source item id: {0}")]
    DuplicateItem(SourceItemId),
    #[error("invalid source tag: {0:?}")]
    InvalidTag(String),
}

impl SourceBatch {
    pub fn try_new(
        batch: BatchId,
        title: SourceTitle,
        subtitle: Option<SourceSubtitle>,
        items: Vec<SourceItem>,
    ) -> Result<Self, SourceError> {
        if items.is_empty() {
            return Err(SourceError::EmptyBatch);
        }
        let mut ids = BTreeSet::new();
        for item in &items {
            if !ids.insert(item.id().clone()) {
                return Err(SourceError::DuplicateItem(item.id().clone()));
            }
        }
        Ok(Self {
            format: SourceFormat::V1,
            batch,
            title,
            subtitle,
            items,
        })
    }

    pub const fn format(&self) -> SourceFormat {
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

    pub fn items(&self) -> &[SourceItem] {
        &self.items
    }

    pub fn item(&self, id: &SourceItemId) -> Option<&SourceItem> {
        self.items.iter().find(|item| item.id() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::{SourceBatch, SourceError, SourceItem, SourceTags};
    use crate::{BatchId, Gloss, SourceFingerprint, SourceItemId, SourceTitle, TargetText};

    fn item(id: &str) -> SourceItem {
        SourceItem::new(
            SourceItemId::parse(id).unwrap(),
            TargetText::parse("यहाँ").unwrap(),
            None,
            Gloss::parse("Here").unwrap(),
            SourceTags::default(),
            SourceFingerprint::parse(
                "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .unwrap(),
        )
    }

    #[test]
    fn rejects_empty_and_duplicate_batches() {
        let batch = BatchId::parse("one").unwrap();
        let title = SourceTitle::parse("One").unwrap();
        assert_eq!(
            SourceBatch::try_new(batch.clone(), title.clone(), None, vec![]).unwrap_err(),
            SourceError::EmptyBatch
        );
        assert!(matches!(
            SourceBatch::try_new(batch, title, None, vec![item("s-a-01"), item("s-a-01")]),
            Err(SourceError::DuplicateItem(_))
        ));
    }
}
