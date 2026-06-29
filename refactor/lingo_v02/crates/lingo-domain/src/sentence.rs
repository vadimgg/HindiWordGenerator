use crate::{
    BatchId, CollectionId, Gloss, LiteralGloss, Romanisation, RunId, SectionName, SentenceAudio,
    SentenceId, SentenceOrder, SentenceTags, TargetText,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
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

    pub fn parse(raw: &str) -> Result<Self, SentenceError> {
        match raw {
            "informal" => Ok(Self::Informal),
            "standard" => Ok(Self::Standard),
            "formal" => Ok(Self::Formal),
            other => Err(SentenceError::UnknownRegister(other.to_string())),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SentenceField {
    Target,
    Romanisation,
    English,
    Literal,
    Register,
    Breakdown,
}

impl SentenceField {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Target => "target",
            Self::Romanisation => "romanisation",
            Self::English => "english",
            Self::Literal => "literal",
            Self::Register => "register",
            Self::Breakdown => "breakdown",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "target" => Some(Self::Target),
            "romanisation" => Some(Self::Romanisation),
            "english" => Some(Self::English),
            "literal" => Some(Self::Literal),
            "register" => Some(Self::Register),
            "breakdown" => Some(Self::Breakdown),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldAuthority {
    Human,
    Ai,
}

impl FieldAuthority {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Ai => "ai",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "human" => Some(Self::Human),
            "ai" => Some(Self::Ai),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FieldAuthoritySet(BTreeMap<SentenceField, FieldAuthority>);

impl FieldAuthoritySet {
    pub fn empty() -> Self { Self::default() }

    pub fn mark(&mut self, field: SentenceField, authority: FieldAuthority) {
        self.0.insert(field, authority);
    }

    pub fn authority(&self, field: SentenceField) -> Option<FieldAuthority> {
        self.0.get(&field).copied()
    }

    pub fn fields(&self) -> &BTreeMap<SentenceField, FieldAuthority> { &self.0 }

    pub fn reject_human_field_changes(
        &self,
        before: &Sentence,
        proposed: &SentenceEnrichment,
    ) -> Result<(), AuthorityError> {
        for (field, authority) in &self.0 {
            if *authority != FieldAuthority::Human { continue; }
            let changed = match field {
                SentenceField::Target => proposed.target.as_ref().is_some_and(|value| value != before.target()),
                SentenceField::Romanisation => proposed.romanisation.as_ref().is_some_and(|value| Some(value) != before.romanisation()),
                SentenceField::English => proposed.english.as_ref().is_some_and(|value| Some(value) != before.english()),
                SentenceField::Literal => proposed.literal.as_ref().is_some_and(|value| Some(value) != before.literal()),
                SentenceField::Register => proposed.register.is_some_and(|value| Some(value) != before.register()),
                SentenceField::Breakdown => proposed.breakdown.as_ref().is_some_and(|value| Some(value) != before.breakdown()),
            };
            if changed {
                return Err(AuthorityError::HumanFieldChanged(*field));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum AuthorityError {
    #[error("human-authored field {0:?} was changed")]
    HumanFieldChanged(SentenceField),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SentenceStatus {
    Draft,
    Enriching,
    Enriched,
    /// Reviewed and confirmed by the learner; the only status shown in Organize
    /// and included in exports by default.
    Active,
}

impl SentenceStatus {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Enriching => "enriching",
            Self::Enriched => "enriched",
            Self::Active => "active",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, SentenceError> {
        match raw {
            "draft" => Ok(Self::Draft),
            "enriching" => Ok(Self::Enriching),
            "enriched" => Ok(Self::Enriched),
            "active" => Ok(Self::Active),
            other => Err(SentenceError::UnknownStatus(other.to_string())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SentenceProvenance {
    Generated { run: RunId },
    Imported { package: String, source_batch: Option<String>, source_item: Option<String> },
    Manual,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BreakdownItem {
    surface: TargetText,
    #[serde(skip_serializing_if = "Option::is_none")]
    roman: Option<Romanisation>,
    gloss: Gloss,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<String>,
}

impl BreakdownItem {
    pub fn new(surface: TargetText, roman: Option<Romanisation>, gloss: Gloss, kind: Option<String>) -> Self {
        Self { surface, roman, gloss, kind }
    }
    pub fn surface(&self) -> &TargetText { &self.surface }
    pub fn roman(&self) -> Option<&Romanisation> { self.roman.as_ref() }
    pub fn gloss(&self) -> &Gloss { &self.gloss }
    pub fn kind(&self) -> Option<&str> { self.kind.as_deref() }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TokenBreakdown(Vec<BreakdownItem>);

impl TokenBreakdown {
    pub fn try_new(items: Vec<BreakdownItem>) -> Result<Self, SentenceError> {
        if items.is_empty() { return Err(SentenceError::EmptyBreakdown); }
        Ok(Self(items))
    }
    pub fn items(&self) -> &[BreakdownItem] { &self.0 }
    pub fn into_items(self) -> Vec<BreakdownItem> { self.0 }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Sentence {
    id: SentenceId,
    collection: CollectionId,
    batch_id: Option<BatchId>,
    title: Option<SectionName>,
    section: Option<SectionName>,
    order: SentenceOrder,
    target: TargetText,
    romanisation: Option<Romanisation>,
    english: Option<Gloss>,
    literal: Option<LiteralGloss>,
    register: Option<Register>,
    authority: FieldAuthoritySet,
    status: SentenceStatus,
    enrich_run: Option<RunId>,
    provenance: SentenceProvenance,
    tags: SentenceTags,
    breakdown: Option<TokenBreakdown>,
    audio: Option<SentenceAudio>,
}


#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SentenceRecordInput {
    pub id: SentenceId,
    pub collection: CollectionId,
    pub batch_id: Option<BatchId>,
    pub title: Option<SectionName>,
    pub section: Option<SectionName>,
    pub order: SentenceOrder,
    pub target: TargetText,
    pub romanisation: Option<Romanisation>,
    pub english: Option<Gloss>,
    pub literal: Option<LiteralGloss>,
    pub register: Option<Register>,
    pub authority: FieldAuthoritySet,
    pub status: SentenceStatus,
    pub enrich_run: Option<RunId>,
    pub provenance: SentenceProvenance,
    pub tags: SentenceTags,
    pub breakdown: Option<TokenBreakdown>,
    pub audio: Option<SentenceAudio>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftSentenceInput {
    pub id: SentenceId,
    pub collection: CollectionId,
    pub batch_id: Option<BatchId>,
    pub title: Option<SectionName>,
    pub section: Option<SectionName>,
    pub order: SentenceOrder,
    pub target: TargetText,
    pub romanisation: Option<Romanisation>,
    pub english: Option<Gloss>,
    pub authority: FieldAuthoritySet,
    pub tags: SentenceTags,
    pub provenance: SentenceProvenance,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SentenceEnrichment {
    pub id: SentenceId,
    pub target: Option<TargetText>,
    pub romanisation: Option<Romanisation>,
    pub english: Option<Gloss>,
    pub literal: Option<LiteralGloss>,
    pub register: Option<Register>,
    pub breakdown: Option<TokenBreakdown>,
}

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum SentenceError {
    #[error("unknown sentence status: {0:?}")]
    UnknownStatus(String),
    #[error("unknown register: {0:?}")]
    UnknownRegister(String),
    #[error("breakdown must contain at least one item")]
    EmptyBreakdown,
    #[error("enrichment belongs to another sentence")]
    WrongSentence,
    #[error("{0} is required before a sentence can be marked enriched")]
    MissingRequired(&'static str),
    #[error(transparent)]
    Authority(#[from] AuthorityError),
}

impl Sentence {
    pub fn from_record(input: SentenceRecordInput) -> Self {
        Self {
            id: input.id,
            collection: input.collection,
            batch_id: input.batch_id,
            title: input.title,
            section: input.section,
            order: input.order,
            target: input.target,
            romanisation: input.romanisation,
            english: input.english,
            literal: input.literal,
            register: input.register,
            authority: input.authority,
            status: input.status,
            enrich_run: input.enrich_run,
            provenance: input.provenance,
            tags: input.tags,
            breakdown: input.breakdown,
            audio: input.audio,
        }
    }

    pub fn draft(input: DraftSentenceInput) -> Result<Self, SentenceError> {
        Ok(Self {
            id: input.id,
            collection: input.collection,
            batch_id: input.batch_id,
            title: input.title,
            section: input.section,
            order: input.order,
            target: input.target,
            romanisation: input.romanisation,
            english: input.english,
            literal: None,
            register: None,
            authority: input.authority,
            status: SentenceStatus::Draft,
            enrich_run: None,
            provenance: input.provenance,
            tags: input.tags,
            breakdown: None,
            audio: None,
        })
    }

    pub fn apply_enrichment(&mut self, enrichment: SentenceEnrichment) -> Result<(), SentenceError> {
        if enrichment.id != self.id { return Err(SentenceError::WrongSentence); }
        self.authority.reject_human_field_changes(self, &enrichment)?;

        if self.authority.authority(SentenceField::Target) != Some(FieldAuthority::Human) {
            if let Some(value) = enrichment.target { self.target = value; self.authority.mark(SentenceField::Target, FieldAuthority::Ai); }
        }
        if self.authority.authority(SentenceField::Romanisation) != Some(FieldAuthority::Human) {
            if let Some(value) = enrichment.romanisation { self.romanisation = Some(value); self.authority.mark(SentenceField::Romanisation, FieldAuthority::Ai); }
        }
        if self.authority.authority(SentenceField::English) != Some(FieldAuthority::Human) {
            if let Some(value) = enrichment.english { self.english = Some(value); self.authority.mark(SentenceField::English, FieldAuthority::Ai); }
        }
        if self.authority.authority(SentenceField::Literal) != Some(FieldAuthority::Human) {
            if let Some(value) = enrichment.literal { self.literal = Some(value); self.authority.mark(SentenceField::Literal, FieldAuthority::Ai); }
        }
        if self.authority.authority(SentenceField::Register) != Some(FieldAuthority::Human) {
            if let Some(value) = enrichment.register { self.register = Some(value); self.authority.mark(SentenceField::Register, FieldAuthority::Ai); }
        }
        if self.authority.authority(SentenceField::Breakdown) != Some(FieldAuthority::Human) {
            if let Some(value) = enrichment.breakdown { self.breakdown = Some(value); self.authority.mark(SentenceField::Breakdown, FieldAuthority::Ai); }
        }

        if self.romanisation.is_none() { return Err(SentenceError::MissingRequired("romanisation")); }
        if self.english.is_none() { return Err(SentenceError::MissingRequired("english")); }
        if self.literal.is_none() { return Err(SentenceError::MissingRequired("literal")); }
        if self.breakdown.is_none() { return Err(SentenceError::MissingRequired("breakdown")); }
        self.status = SentenceStatus::Enriched;
        self.enrich_run = None;
        Ok(())
    }

    pub fn set_section(&mut self, section: Option<SectionName>) { self.section = section; }
    pub fn set_title(&mut self, title: Option<SectionName>) { self.title = title; }
    pub fn set_batch(&mut self, batch: Option<BatchId>) { self.batch_id = batch; }
    pub fn set_order(&mut self, order: SentenceOrder) { self.order = order; }
    pub fn mark_enriching(&mut self, run: RunId) { self.status = SentenceStatus::Enriching; self.enrich_run = Some(run); }
    pub fn reset_claim(&mut self) { self.status = SentenceStatus::Draft; self.enrich_run = None; }
    pub fn attach_audio(&mut self, audio: SentenceAudio) { self.audio = Some(audio); }

    pub fn id(&self) -> &SentenceId { &self.id }
    pub fn collection(&self) -> &CollectionId { &self.collection }
    pub fn batch_id(&self) -> Option<&BatchId> { self.batch_id.as_ref() }
    pub fn title(&self) -> Option<&SectionName> { self.title.as_ref() }
    pub fn section(&self) -> Option<&SectionName> { self.section.as_ref() }
    pub const fn order(&self) -> SentenceOrder { self.order }
    pub fn target(&self) -> &TargetText { &self.target }
    pub fn romanisation(&self) -> Option<&Romanisation> { self.romanisation.as_ref() }
    pub fn english(&self) -> Option<&Gloss> { self.english.as_ref() }
    pub fn literal(&self) -> Option<&LiteralGloss> { self.literal.as_ref() }
    pub fn register(&self) -> Option<Register> { self.register }
    pub fn authority(&self) -> &FieldAuthoritySet { &self.authority }
    pub const fn status(&self) -> SentenceStatus { self.status }
    pub fn enrich_run(&self) -> Option<&RunId> { self.enrich_run.as_ref() }
    pub fn provenance(&self) -> &SentenceProvenance { &self.provenance }
    pub fn tags(&self) -> &SentenceTags { &self.tags }
    pub fn breakdown(&self) -> Option<&TokenBreakdown> { self.breakdown.as_ref() }
    pub fn audio(&self) -> Option<&SentenceAudio> { self.audio.as_ref() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CollectionId, Gloss, LiteralGloss, SectionName, SentenceId, SentenceOrder, TargetText};

    fn draft_with_human_english() -> Sentence {
        let mut authority = FieldAuthoritySet::empty();
        authority.mark(SentenceField::English, FieldAuthority::Human);
        Sentence::draft(DraftSentenceInput {
            id: SentenceId::parse("sen-test").unwrap(),
            collection: CollectionId::parse("col-test").unwrap(),
            batch_id: None,
            title: None,
            section: Some(SectionName::parse("Chapter 01").unwrap()),
            order: SentenceOrder::new(1).unwrap(),
            target: TargetText::parse("अध्यापक जी").unwrap(),
            romanisation: None,
            english: Some(Gloss::parse("Teacher ji").unwrap()),
            authority,
            tags: SentenceTags::default(),
            provenance: SentenceProvenance::Manual,
        }).unwrap()
    }

    #[test]
    fn preserves_human_fields_during_enrichment() {
        let mut sentence = draft_with_human_english();
        let enrichment = SentenceEnrichment {
            id: sentence.id().clone(),
            target: None,
            romanisation: Some(Romanisation::parse("adhyapak jī").unwrap()),
            english: Some(Gloss::parse("Teacher").unwrap()),
            literal: Some(LiteralGloss::parse("teacher honorific").unwrap()),
            register: Some(Register::Standard),
            breakdown: Some(TokenBreakdown::try_new(vec![BreakdownItem::new(
                TargetText::parse("जी").unwrap(),
                Some(Romanisation::parse("jī").unwrap()),
                Gloss::parse("honorific (ji)").unwrap(),
                None,
            )]).unwrap()),
        };
        assert!(matches!(sentence.apply_enrichment(enrichment), Err(SentenceError::Authority(_))));
    }

    #[test]
    fn applies_ai_fields_when_human_text_is_unchanged() {
        let mut sentence = draft_with_human_english();
        sentence.apply_enrichment(SentenceEnrichment {
            id: sentence.id().clone(),
            target: None,
            romanisation: Some(Romanisation::parse("adhyapak jī").unwrap()),
            english: Some(Gloss::parse("Teacher ji").unwrap()),
            literal: Some(LiteralGloss::parse("teacher honorific").unwrap()),
            register: Some(Register::Standard),
            breakdown: Some(TokenBreakdown::try_new(vec![BreakdownItem::new(
                TargetText::parse("जी").unwrap(),
                Some(Romanisation::parse("jī").unwrap()),
                Gloss::parse("honorific (ji)").unwrap(),
                None,
            )]).unwrap()),
        }).unwrap();
        assert_eq!(sentence.status(), SentenceStatus::Enriched);
        assert_eq!(sentence.english().unwrap().as_str(), "Teacher ji");
    }
}
