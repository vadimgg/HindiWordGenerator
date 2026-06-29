use crate::ports::{
    AppendBatchRawText, CreateBatch, DeckContextProvider, DraftSentence, ExtractPromptRequest,
    InsertDrafts, LibraryFailure, LibraryStore, PromptEngine,
};
use crate::report::NextAction;
use lingo_domain::{
    BatchId, BatchName, CollectionTitle, FieldAuthority, FieldAuthoritySet, Gloss, Romanisation,
    RunId, SectionName, SentenceField, SentenceProvenance, SentenceTags, TargetText,
};
use thiserror::Error;

/// Whether an extraction creates a new batch or appends to an existing one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BatchRef {
    Existing(BatchId),
    New {
        name: BatchName,
        default_title: Option<SectionName>,
        default_subtitle: Option<SectionName>,
    },
}

pub struct ExtractDeps<'a> {
    pub library: &'a dyn LibraryStore,
    pub context: &'a dyn DeckContextProvider,
    pub prompts: &'a dyn PromptEngine,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrepareExtract {
    pub run_id: RunId,
    pub raw: String,
    pub collection: Option<CollectionTitle>,
    pub section: Option<SectionName>,
    /// Create a new batch (persisting the raw text) or append to an existing one.
    pub batch: Option<BatchRef>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtractPreparation {
    pub run_id: RunId,
    pub packet: String,
    /// The batch the drafts will belong to (resolved/created at prepare time).
    pub batch_id: Option<BatchId>,
    pub next: NextAction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyExtract {
    pub run_id: RunId,
    pub reply: String,
    pub collection: Option<CollectionTitle>,
    pub section: Option<SectionName>,
    pub batch_id: Option<BatchId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtractReport {
    pub created: usize,
    pub next: NextAction,
}

#[derive(Debug, Error)]
pub enum ExtractError {
    #[error(transparent)]
    Library(#[from] LibraryFailure),
    #[error(transparent)]
    Context(#[from] crate::ports::ContextFailure),
    #[error(transparent)]
    Prompt(#[from] crate::ports::PromptFailure),
    #[error("extract reply is invalid: {0}")]
    InvalidDraft(String),
}

pub fn prepare_extract(deps: &ExtractDeps<'_>, request: PrepareExtract) -> Result<ExtractPreparation, ExtractError> {
    let context = deps.context.resolve()?;
    // Resolve/create the batch and persist the raw text now, so the (side-effect
    // free) apply step only needs the batch id.
    let batch_id = match request.batch {
        Some(BatchRef::New { name, default_title, default_subtitle }) => {
            let report = deps.library.create_batch(CreateBatch {
                collection: context.default_collection.clone(),
                title: request.collection.clone().unwrap_or_else(|| context.default_title.clone()),
                language: context.profile.code().clone(),
                name,
                default_title,
                default_subtitle,
                raw_text: request.raw.clone(),
            })?;
            Some(report.batch.id().clone())
        }
        Some(BatchRef::Existing(id)) => {
            deps.library.append_batch_raw_text(AppendBatchRawText { batch_id: id.clone(), raw_text: request.raw.clone() })?;
            Some(id)
        }
        None => None,
    };
    let packet = deps.prompts.render_extract(&ExtractPromptRequest {
        run_id: request.run_id.clone(),
        raw: request.raw,
        collection_title: request.collection.unwrap_or_else(|| context.default_title.clone()),
        section: request.section,
        context,
    })?;
    Ok(ExtractPreparation { run_id: request.run_id, packet: packet.content, batch_id, next: NextAction::Enrich })
}

pub fn apply_extract(deps: &ExtractDeps<'_>, request: ApplyExtract) -> Result<ExtractReport, ExtractError> {
    let context = deps.context.resolve()?;
    // Resolve the batch (if any) for its default subtitle fallback.
    let batch = request.batch_id.as_ref().map(|id| deps.library.get_batch(id)).transpose()?;
    let draft = deps.prompts.parse_extract_reply(&request.reply)?;
    if draft.sentences.is_empty() {
        return Err(ExtractError::InvalidDraft("reply contains no sentences".to_string()));
    }
    let mut accepted = Vec::with_capacity(draft.sentences.len());
    for raw in draft.sentences {
        let target = TargetText::parse(raw.target).map_err(invalid)?;
        let romanisation = raw.romanisation.map(Romanisation::parse).transpose().map_err(invalid)?;
        let english = raw.english.map(Gloss::parse).transpose().map_err(invalid)?;
        let mut authority = FieldAuthoritySet::empty();
        for (field, source) in raw.authority {
            let Some(field) = SentenceField::parse(&field) else {
                return Err(ExtractError::InvalidDraft(format!("unknown authority field {field:?}")));
            };
            let Some(source) = FieldAuthority::parse(&source) else {
                return Err(ExtractError::InvalidDraft(format!("unknown field authority {source:?}")));
            };
            authority.mark(field, source);
        }
        if romanisation.is_some() && authority.authority(SentenceField::Romanisation).is_none() {
            authority.mark(SentenceField::Romanisation, FieldAuthority::Human);
        }
        if english.is_some() && authority.authority(SentenceField::English).is_none() {
            authority.mark(SentenceField::English, FieldAuthority::Human);
        }
        let tags = SentenceTags::try_from_values(raw.tags).map_err(invalid)?;
        accepted.push(DraftSentence {
            target,
            romanisation,
            english,
            authority,
            tags,
            provenance: SentenceProvenance::Generated { run: request.run_id.clone() },
        });
    }
    // Subtitle falls back to the batch default; per-sentence title stays None so
    // changing the batch default re-flows to inherited sentences.
    let section = request.section.or_else(|| batch.as_ref().and_then(|b| b.default_subtitle().cloned()));
    let report = deps.library.insert_drafts(InsertDrafts {
        collection: context.default_collection,
        title: request.collection.unwrap_or(context.default_title),
        language: context.profile.code().clone(),
        batch_id: request.batch_id.clone(),
        sentence_title: None,
        section,
        drafts: accepted,
    })?;
    Ok(ExtractReport { created: report.created, next: NextAction::Enrich })
}

fn invalid(error: impl std::fmt::Display) -> ExtractError {
    ExtractError::InvalidDraft(error.to_string())
}
