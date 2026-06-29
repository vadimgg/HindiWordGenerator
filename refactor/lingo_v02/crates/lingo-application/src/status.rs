use crate::ports::{DeckContextProvider, LibraryFailure, LibraryStore, LibrarySummary};
use crate::report::NextAction;
use thiserror::Error;

pub struct StatusDeps<'a> {
    pub library: &'a dyn LibraryStore,
    pub context: &'a dyn DeckContextProvider,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusReport {
    pub summary: LibrarySummary,
    pub next: NextAction,
}

#[derive(Debug, Error)]
pub enum StatusError {
    #[error(transparent)] Library(#[from] LibraryFailure),
    #[error(transparent)] Context(#[from] crate::ports::ContextFailure),
}

pub fn status(deps: &StatusDeps<'_>) -> Result<StatusReport, StatusError> {
    let _context = deps.context.resolve()?;
    let summary = deps.library.summary()?;
    let next = if summary.sentences == 0 {
        NextAction::Extract
    } else if summary.draft > 0 || summary.enriching > 0 {
        NextAction::Enrich
    } else if summary.audio < summary.sentences {
        NextAction::Audio
    } else {
        NextAction::Package
    };
    Ok(StatusReport { summary, next })
}
