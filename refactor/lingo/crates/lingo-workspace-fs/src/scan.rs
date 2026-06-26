use crate::store::FsWorkspace;
use lingo_application::ports::{
    BatchSnapshot, WorkspaceFailure, WorkspaceSnapshot, WorkspaceStore,
};
use lingo_domain::BatchId;
use std::collections::BTreeSet;

pub(crate) fn scan_workspace(
    workspace: &FsWorkspace,
) -> Result<WorkspaceSnapshot, WorkspaceFailure> {
    let raw = workspace.list_raw()?;
    let source_batches = workspace.list_source_batches()?;
    let card_batches = workspace.list_card_batches()?;
    let batches = source_batches
        .iter()
        .cloned()
        .chain(card_batches.iter().cloned())
        .collect::<BTreeSet<_>>();

    let mut snapshots = Vec::with_capacity(batches.len());
    for batch in batches {
        snapshots.push(scan_batch(
            workspace,
            &batch,
            &source_batches,
            &card_batches,
        ));
    }
    Ok(WorkspaceSnapshot {
        raw,
        batches: snapshots,
    })
}

fn scan_batch(
    workspace: &FsWorkspace,
    batch: &BatchId,
    source_batches: &[BatchId],
    card_batches: &[BatchId],
) -> BatchSnapshot {
    let source_present = source_batches.contains(batch);
    let cards_present = card_batches.contains(batch);
    let malformed_source = source_present
        .then(|| {
            workspace
                .load_source(batch)
                .err()
                .map(|error| error.to_string())
        })
        .flatten();
    let cards_result = cards_present.then(|| workspace.load_cards(batch));
    let malformed_cards = cards_result
        .as_ref()
        .and_then(|result| result.as_ref().err().map(ToString::to_string));
    let (card_count, audio_present) = cards_result
        .and_then(Result::ok)
        .map(|cards| {
            let total = cards.cards().len();
            let audio = cards
                .cards()
                .iter()
                .filter(|card| card.audio().is_some())
                .count();
            (total, audio)
        })
        .unwrap_or_default();

    BatchSnapshot {
        batch: batch.clone(),
        source_present,
        cards_present,
        card_count,
        audio_present,
        malformed_source,
        malformed_cards,
    }
}
