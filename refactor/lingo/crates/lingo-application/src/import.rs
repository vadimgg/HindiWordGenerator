use crate::ports::{
    ContextFailure, DeckContextProvider, ImportDraftItem, ImportPromptRequest, PreparedRun,
    PromptEngine, PromptFailure, PromptStage, RunFailure, RunJournal, RunRecord, SourceBatchDraft,
    StoredFile, WorkspaceFailure, WorkspaceStore,
};
use crate::report::NextAction;
use lingo_domain::{
    BatchId, Gloss, LanguageProfile, ProfileId, RawDocumentId, Romanisation, RunId, SourceBatch,
    SourceError, SourceFingerprint, SourceItem, SourceItemId, SourceSubtitle, SourceTags,
    SourceTextParts, SourceTitle, TargetText, source_fingerprint,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use thiserror::Error;

pub struct ImportDeps<'a> {
    pub workspace: &'a dyn WorkspaceStore,
    pub context: &'a dyn DeckContextProvider,
    pub prompts: &'a dyn PromptEngine,
    pub runs: &'a dyn RunJournal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrepareImport {
    pub run_id: RunId,
    pub raw: RawDocumentId,
    pub batch: BatchId,
    pub title: SourceTitle,
    pub subtitle: Option<SourceSubtitle>,
    pub journal: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportPreparation {
    pub run: Option<RunRecord>,
    pub packet: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyImport {
    pub run_id: RunId,
    pub reply: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportReport {
    pub batch: BatchId,
    pub items: usize,
    pub stored: StoredFile,
    pub next: NextAction,
}

#[derive(Debug, Error)]
pub enum ImportError {
    #[error(transparent)]
    Workspace(#[from] WorkspaceFailure),
    #[error(transparent)]
    Context(#[from] ContextFailure),
    #[error(transparent)]
    Prompt(#[from] PromptFailure),
    #[error(transparent)]
    Run(#[from] RunFailure),
    #[error("prepared run is for {actual:?}, not import")]
    WrongStage { actual: PromptStage },
    #[error("prepared run profile {prepared} does not match current profile {current}")]
    ProfileChanged {
        prepared: ProfileId,
        current: ProfileId,
    },
    #[error("import reply is invalid: {0}")]
    InvalidDraft(String),
}

pub fn prepare_import(
    deps: &ImportDeps<'_>,
    request: PrepareImport,
) -> Result<ImportPreparation, ImportError> {
    let raw = deps.workspace.load_raw(&request.raw)?;
    let context = deps.context.resolve()?;
    let packet = deps.prompts.render_import(&ImportPromptRequest {
        batch: request.batch.clone(),
        title: request.title.clone(),
        subtitle: request.subtitle.clone(),
        raw,
        context: context.clone(),
    })?;
    let prepared = PreparedRun {
        id: request.run_id,
        stage: PromptStage::Import,
        batch: request.batch,
        profile: context.profile.id().clone(),
        title: request.title,
        subtitle: request.subtitle,
        packet: packet.clone(),
    };
    let run = request
        .journal
        .then(|| deps.runs.record_prepared(&prepared))
        .transpose()?;
    Ok(ImportPreparation {
        run,
        packet: packet.content,
    })
}

pub fn apply_import(
    deps: &ImportDeps<'_>,
    request: ApplyImport,
) -> Result<ImportReport, ImportError> {
    let prepared = deps.runs.require_prepared(&request.run_id)?;
    if prepared.stage != PromptStage::Import {
        return Err(ImportError::WrongStage {
            actual: prepared.stage,
        });
    }
    let context = deps.context.resolve()?;
    if prepared.profile != context.profile.id().clone() {
        return Err(ImportError::ProfileChanged {
            prepared: prepared.profile,
            current: context.profile.id().clone(),
        });
    }
    let prior = match deps.workspace.load_source(&prepared.batch) {
        Ok(source) => Some(source),
        Err(WorkspaceFailure::NotFound(_)) => None,
        Err(WorkspaceFailure::InvalidData(_)) => None,
        Err(error) => return Err(error.into()),
    };
    let draft = deps.prompts.parse_import_reply(&request.reply)?;
    let source = accept_draft(
        draft,
        prepared.batch.clone(),
        prepared.title,
        prepared.subtitle,
        &context.profile,
        prior.as_ref(),
    )?;
    let stored = if prior.is_some() {
        deps.workspace.replace_source(&source)?
    } else {
        match deps.workspace.create_source(&source) {
            Ok(stored) => stored,
            Err(WorkspaceFailure::Collision(_)) => deps.workspace.replace_source(&source)?,
            Err(error) => return Err(error.into()),
        }
    };
    deps.runs
        .record_applied(&request.run_id, &request.reply, &stored)?;
    Ok(ImportReport {
        batch: source.batch_id().clone(),
        items: source.items().len(),
        stored,
        next: NextAction::Build {
            batch: source.batch_id().clone(),
        },
    })
}

fn accept_draft(
    draft: SourceBatchDraft,
    batch: BatchId,
    title: SourceTitle,
    subtitle: Option<SourceSubtitle>,
    profile: &LanguageProfile,
    prior: Option<&SourceBatch>,
) -> Result<SourceBatch, ImportError> {
    if draft.items.is_empty() {
        return Err(ImportError::InvalidDraft(
            "reply contains no source items".to_string(),
        ));
    }
    let mut prior_ids = prior_ids_by_fingerprint(prior);
    let mut duplicate_ordinals = BTreeMap::<SourceFingerprint, usize>::new();
    let mut used_ids = BTreeSet::<SourceItemId>::new();
    let mut items = Vec::with_capacity(draft.items.len());
    for item in draft.items {
        let accepted = accept_item(
            item,
            profile,
            &mut duplicate_ordinals,
            &mut prior_ids,
            &mut used_ids,
        )?;
        items.push(accepted);
    }
    SourceBatch::try_new(batch, title, subtitle, items).map_err(map_source_error)
}

fn prior_ids_by_fingerprint(
    prior: Option<&SourceBatch>,
) -> BTreeMap<SourceFingerprint, VecDeque<SourceItemId>> {
    let mut ids = BTreeMap::<SourceFingerprint, VecDeque<SourceItemId>>::new();
    if let Some(prior) = prior {
        for item in prior.items() {
            ids.entry(item.fingerprint().clone())
                .or_default()
                .push_back(item.id().clone());
        }
    }
    ids
}

fn accept_item(
    item: ImportDraftItem,
    profile: &LanguageProfile,
    ordinals: &mut BTreeMap<SourceFingerprint, usize>,
    prior_ids: &mut BTreeMap<SourceFingerprint, VecDeque<SourceItemId>>,
    used_ids: &mut BTreeSet<SourceItemId>,
) -> Result<SourceItem, ImportError> {
    let target = TargetText::parse(item.target).map_err(invalid)?;
    let romanisation = item
        .romanisation
        .map(Romanisation::parse)
        .transpose()
        .map_err(invalid)?;
    if profile.romanisation().is_required() && romanisation.is_none() {
        return Err(ImportError::InvalidDraft(
            "profile requires romanisation for every item".to_string(),
        ));
    }
    let english = Gloss::parse(item.english).map_err(invalid)?;
    let tags = SourceTags::try_from_values(item.tags).map_err(invalid)?;
    let fingerprint = source_fingerprint(&SourceTextParts {
        target: target.as_str(),
        romanisation: romanisation.as_ref().map(Romanisation::as_str),
        english: english.as_str(),
    });
    let ordinal = ordinals.entry(fingerprint.clone()).or_default();
    *ordinal += 1;
    let id = if let Some(prior_id) = prior_ids
        .get_mut(&fingerprint)
        .and_then(VecDeque::pop_front)
        .filter(|id| !used_ids.contains(id))
    {
        prior_id
    } else {
        let mut candidate_ordinal = *ordinal;
        loop {
            let candidate =
                SourceItemId::from_fingerprint_prefix(fingerprint.hex(), candidate_ordinal)
                    .map_err(invalid)?;
            if !used_ids.contains(&candidate) {
                break candidate;
            }
            candidate_ordinal += 1;
        }
    };
    used_ids.insert(id.clone());
    Ok(SourceItem::new(
        id,
        target,
        romanisation,
        english,
        tags,
        fingerprint,
    ))
}

fn invalid(error: impl std::fmt::Display) -> ImportError {
    ImportError::InvalidDraft(error.to_string())
}

fn map_source_error(error: SourceError) -> ImportError {
    invalid(error)
}
