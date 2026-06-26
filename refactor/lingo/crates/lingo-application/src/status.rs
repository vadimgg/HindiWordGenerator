use crate::ports::{ContextFailure, DeckContextProvider, WorkspaceFailure, WorkspaceStore};
use crate::report::NextAction;
use lingo_domain::{
    AudioCoverage, BatchId, CheckState, RawDocumentId, ValidationReport, check_card_batch,
};
use std::collections::BTreeSet;
use thiserror::Error;

pub struct StatusDeps<'a> {
    pub workspace: &'a dyn WorkspaceStore,
    pub context: &'a dyn DeckContextProvider,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StatusRequest {
    pub batch: Option<BatchId>,
    pub problems_only: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusRow {
    pub batch: BatchId,
    pub source_present: bool,
    pub cards_present: bool,
    pub check: CheckState,
    pub audio: AudioCoverage,
    pub problem: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingRaw {
    pub id: RawDocumentId,
    pub batch_hint: Option<BatchId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusReport {
    pub rows: Vec<StatusRow>,
    pub pending_raw: Vec<PendingRaw>,
    pub total_batches: usize,
    pub next: NextAction,
}

#[derive(Debug, Error)]
pub enum StatusError {
    #[error(transparent)]
    Workspace(#[from] WorkspaceFailure),
    #[error(transparent)]
    Context(#[from] ContextFailure),
}

pub fn status(deps: &StatusDeps<'_>, request: StatusRequest) -> Result<StatusReport, StatusError> {
    let snapshot = deps.workspace.scan()?;
    let context = deps.context.resolve()?;
    let source_batches = deps
        .workspace
        .list_source_batches()?
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut rows = Vec::new();
    for batch in snapshot.batches {
        if request
            .batch
            .as_ref()
            .is_some_and(|selected| selected != &batch.batch)
        {
            continue;
        }
        let mut problem = batch
            .malformed_source
            .clone()
            .or(batch.malformed_cards.clone());
        let check = if batch.cards_present && batch.source_present && problem.is_none() {
            let cards = deps.workspace.load_cards(&batch.batch)?;
            let source = deps.workspace.load_source(&batch.batch)?;
            let validation: ValidationReport = check_card_batch(&cards, &source, &context.profile);
            if validation.is_clean() {
                CheckState::Clean
            } else {
                problem = Some(format!(
                    "{} error(s), {} warning(s)",
                    validation.error_count(),
                    validation.warning_count()
                ));
                CheckState::Problems {
                    errors: validation.error_count(),
                    warnings: validation.warning_count(),
                }
            }
        } else {
            CheckState::NotRun
        };
        let row = StatusRow {
            batch: batch.batch,
            source_present: batch.source_present,
            cards_present: batch.cards_present,
            check,
            audio: AudioCoverage::new(batch.audio_present, batch.card_count),
            problem,
        };
        let actionable_problem =
            row.problem.is_some() || !row.cards_present || !row.audio.is_complete();
        if !request.problems_only || actionable_problem {
            rows.push(row);
        }
    }
    let pending_raw = snapshot
        .raw
        .into_iter()
        .filter(|raw| {
            raw.batch_hint
                .as_ref()
                .is_none_or(|batch| !source_batches.contains(batch))
        })
        .map(|raw| PendingRaw {
            id: raw.id,
            batch_hint: raw.batch_hint,
        })
        .collect::<Vec<_>>();
    let next = choose_next(&pending_raw, &rows);
    let total_batches = rows.len();
    Ok(StatusReport {
        rows,
        pending_raw,
        total_batches,
        next,
    })
}

fn choose_next(pending_raw: &[PendingRaw], rows: &[StatusRow]) -> NextAction {
    if let Some(raw) = pending_raw.first() {
        return NextAction::Import {
            raw: Some(raw.id.clone()),
        };
    }
    if let Some(row) = rows
        .iter()
        .find(|row| row.source_present && !row.cards_present)
    {
        return NextAction::Build {
            batch: row.batch.clone(),
        };
    }
    if let Some(row) = rows
        .iter()
        .find(|row| matches!(row.check, CheckState::Problems { errors, .. } if errors > 0))
    {
        return NextAction::Check {
            batch: Some(row.batch.clone()),
        };
    }
    if let Some(row) = rows
        .iter()
        .find(|row| row.cards_present && !row.audio.is_complete())
    {
        return NextAction::Audio {
            batch: Some(row.batch.clone()),
        };
    }
    if rows.is_empty() {
        NextAction::Import { raw: None }
    } else {
        NextAction::Package
    }
}
