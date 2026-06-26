use crate::ports::{ContextFailure, DeckContextProvider, WorkspaceFailure, WorkspaceStore};
use lingo_domain::{BatchId, ValidationReport, check_card_batch};
use thiserror::Error;

pub struct CheckDeps<'a> {
    pub workspace: &'a dyn WorkspaceStore,
    pub context: &'a dyn DeckContextProvider,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CheckRequest {
    pub batch: Option<BatchId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedBatch {
    pub batch: BatchId,
    pub report: ValidationReport,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CheckReport {
    pub batches: Vec<CheckedBatch>,
}

impl CheckReport {
    pub fn is_clean(&self) -> bool {
        self.batches.iter().all(|batch| batch.report.is_clean())
    }

    pub fn error_count(&self) -> usize {
        self.batches
            .iter()
            .map(|batch| batch.report.error_count())
            .sum()
    }

    pub fn warning_count(&self) -> usize {
        self.batches
            .iter()
            .map(|batch| batch.report.warning_count())
            .sum()
    }
}

#[derive(Debug, Error)]
pub enum CheckError {
    #[error(transparent)]
    Workspace(#[from] WorkspaceFailure),
    #[error(transparent)]
    Context(#[from] ContextFailure),
}

pub fn check(deps: &CheckDeps<'_>, request: CheckRequest) -> Result<CheckReport, CheckError> {
    let context = deps.context.resolve()?;
    let batches = match request.batch {
        Some(batch) => vec![batch],
        None => deps.workspace.list_card_batches()?,
    };
    let mut report = CheckReport::default();
    for batch in batches {
        let cards = deps.workspace.load_cards(&batch)?;
        let source = deps.workspace.load_source(&batch)?;
        report.batches.push(CheckedBatch {
            batch,
            report: check_card_batch(&cards, &source, &context.profile),
        });
    }
    Ok(report)
}
