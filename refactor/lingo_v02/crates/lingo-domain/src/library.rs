use crate::{BatchId, BatchName, CollectionId, CollectionTitle, LanguageCode, SectionName};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Collection {
    id: CollectionId,
    title: CollectionTitle,
    language: LanguageCode,
}

impl Collection {
    pub fn new(id: CollectionId, title: CollectionTitle, language: LanguageCode) -> Self {
        Self { id, title, language }
    }
    pub fn id(&self) -> &CollectionId { &self.id }
    pub fn title(&self) -> &CollectionTitle { &self.title }
    pub fn language(&self) -> &LanguageCode { &self.language }
}

/// A batch — one raw-text extraction unit. Carries a default title and subtitle
/// that its sentences inherit unless overridden, plus the raw source text
/// (appended to when more is extracted into the same batch).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Batch {
    id: BatchId,
    collection: CollectionId,
    name: BatchName,
    default_title: Option<SectionName>,
    default_subtitle: Option<SectionName>,
    raw_text: String,
    created_at: String,
    updated_at: String,
}

impl Batch {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: BatchId,
        collection: CollectionId,
        name: BatchName,
        default_title: Option<SectionName>,
        default_subtitle: Option<SectionName>,
        raw_text: String,
        created_at: String,
        updated_at: String,
    ) -> Self {
        Self { id, collection, name, default_title, default_subtitle, raw_text, created_at, updated_at }
    }
    pub fn id(&self) -> &BatchId { &self.id }
    pub fn collection(&self) -> &CollectionId { &self.collection }
    pub fn name(&self) -> &BatchName { &self.name }
    pub fn default_title(&self) -> Option<&SectionName> { self.default_title.as_ref() }
    pub fn default_subtitle(&self) -> Option<&SectionName> { self.default_subtitle.as_ref() }
    pub fn raw_text(&self) -> &str { &self.raw_text }
    pub fn created_at(&self) -> &str { &self.created_at }
    pub fn updated_at(&self) -> &str { &self.updated_at }
}
