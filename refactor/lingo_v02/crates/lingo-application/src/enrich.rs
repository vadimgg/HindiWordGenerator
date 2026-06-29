use crate::ports::{
    ApplyEnrichmentCommit, ClaimEnrichment, DeckContextProvider, EnrichPromptRequest, LibraryFailure,
    LibraryStore, PromptEngine, ResetEnrichmentClaim, SentenceSelection,
};
use crate::report::NextAction;
use lingo_domain::{
    BreakdownItem, Gloss, LiteralGloss, Register, Romanisation, RunId, SentenceEnrichment,
    SentenceId, TargetText, TokenBreakdown,
};
use std::collections::BTreeSet;
use thiserror::Error;

pub struct EnrichDeps<'a> {
    pub library: &'a dyn LibraryStore,
    pub context: &'a dyn DeckContextProvider,
    pub prompts: &'a dyn PromptEngine,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrepareEnrich {
    pub run_id: RunId,
    pub selection: SentenceSelection,
    pub limit: usize,
    pub force: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnrichPreparation {
    pub run_id: RunId,
    pub claimed: usize,
    pub packet: String,
    pub next: NextAction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyEnrich {
    pub run_id: RunId,
    pub reply: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnrichReport {
    pub updated: usize,
    pub next: NextAction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResetEnrich {
    pub run_id: Option<RunId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResetEnrichReport {
    pub reset: usize,
}

#[derive(Debug, Error)]
pub enum EnrichError {
    #[error(transparent)]
    Library(#[from] LibraryFailure),
    #[error(transparent)]
    Context(#[from] crate::ports::ContextFailure),
    #[error(transparent)]
    Prompt(#[from] crate::ports::PromptFailure),
    #[error("enrich reply is invalid: {0}")]
    InvalidDraft(String),
}

pub fn prepare_enrich(deps: &EnrichDeps<'_>, request: PrepareEnrich) -> Result<EnrichPreparation, EnrichError> {
    let context = deps.context.resolve()?;
    let run = deps.library.claim_for_enrichment(ClaimEnrichment {
        run_id: request.run_id.clone(),
        selection: request.selection,
        limit: request.limit,
        force: request.force,
    })?;
    let packet = deps.prompts.render_enrich(&EnrichPromptRequest { run: run.clone(), context })?;
    Ok(EnrichPreparation { run_id: request.run_id, claimed: run.sentences.len(), packet: packet.content, next: NextAction::Enrich })
}

pub fn apply_enrich(deps: &EnrichDeps<'_>, request: ApplyEnrich) -> Result<EnrichReport, EnrichError> {
    let draft = deps.prompts.parse_enrich_reply(&request.reply)?;
    if draft.sentences.is_empty() {
        return Err(EnrichError::InvalidDraft("reply contains no sentences".to_string()));
    }
    let mut seen = BTreeSet::new();
    let mut enrichments = Vec::with_capacity(draft.sentences.len());
    for raw in draft.sentences {
        let id = SentenceId::parse(raw.id).map_err(invalid)?;
        if !seen.insert(id.clone()) {
            return Err(EnrichError::InvalidDraft(format!("duplicate sentence id {id}")));
        }
        let breakdown = if raw.breakdown.is_empty() {
            None
        } else {
            let items = raw.breakdown.into_iter().map(|item| {
                Ok(BreakdownItem::new(
                    TargetText::parse(item.surface).map_err(invalid)?,
                    item.roman.map(Romanisation::parse).transpose().map_err(invalid)?,
                    Gloss::parse(item.gloss).map_err(invalid)?,
                    item.kind,
                ))
            }).collect::<Result<Vec<_>, EnrichError>>()?;
            Some(TokenBreakdown::try_new(items).map_err(invalid)?)
        };
        enrichments.push(SentenceEnrichment {
            id,
            target: None,
            romanisation: raw.romanisation.map(Romanisation::parse).transpose().map_err(invalid)?,
            english: raw.english.map(Gloss::parse).transpose().map_err(invalid)?,
            literal: raw.literal.map(LiteralGloss::parse).transpose().map_err(invalid)?,
            register: raw.register.as_deref().map(Register::parse).transpose().map_err(invalid)?,
            breakdown,
        });
    }
    let report = deps.library.apply_enrichment(ApplyEnrichmentCommit { run_id: request.run_id, enrichments })?;
    Ok(EnrichReport { updated: report.updated, next: NextAction::Audio })
}

pub fn reset_enrichment_claim(deps: &EnrichDeps<'_>, request: ResetEnrich) -> Result<ResetEnrichReport, EnrichError> {
    let report = deps.library.reset_enrichment_claim(ResetEnrichmentClaim { run_id: request.run_id })?;
    Ok(ResetEnrichReport { reset: report.reset })
}

fn invalid(error: impl std::fmt::Display) -> EnrichError {
    EnrichError::InvalidDraft(error.to_string())
}
