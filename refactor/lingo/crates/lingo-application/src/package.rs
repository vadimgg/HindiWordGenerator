use crate::ports::{
    ArtifactFailure, AudioAsset, ContextFailure, DeckContextProvider, PackagePublisher,
    PublishBatch, PublishMaterial, PublishedArtifact, WorkspaceFailure, WorkspaceStore,
};
use crate::report::NextAction;
use lingo_domain::{BatchId, CardId};
use std::path::PathBuf;
use thiserror::Error;

pub struct PackageDeps<'a> {
    pub workspace: &'a dyn WorkspaceStore,
    pub context: &'a dyn DeckContextProvider,
    pub publisher: &'a dyn PackagePublisher,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PackageRequest {
    pub batch: Option<BatchId>,
    pub destination: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageReport {
    pub artifact: PublishedArtifact,
    pub batches: usize,
    pub cards: usize,
    pub next: NextAction,
}

#[derive(Debug, Error)]
pub enum PackageError {
    #[error(transparent)]
    Workspace(#[from] WorkspaceFailure),
    #[error(transparent)]
    Context(#[from] ContextFailure),
    #[error(transparent)]
    Artifact(#[from] ArtifactFailure),
    #[error("card {0} has no audio; run `lingo audio` first")]
    MissingAudio(CardId),
    #[error("no card batches were selected")]
    EmptySelection,
}

pub fn package(
    deps: &PackageDeps<'_>,
    request: PackageRequest,
) -> Result<PackageReport, PackageError> {
    let context = deps.context.resolve()?;
    let batches = match request.batch {
        Some(batch) => vec![batch],
        None => deps.workspace.list_card_batches()?,
    };
    if batches.is_empty() {
        return Err(PackageError::EmptySelection);
    }
    let mut publish_batches = Vec::new();
    let mut card_count = 0usize;
    for batch in batches {
        let cards = deps.workspace.load_cards(&batch)?;
        card_count += cards.cards().len();
        let mut audio = Vec::new();
        for card in cards.cards() {
            let Some(reference) = card.audio() else {
                return Err(PackageError::MissingAudio(card.id().clone()));
            };
            audio.push(AudioAsset {
                reference: reference.clone(),
                bytes: deps.workspace.read_audio(reference)?,
            });
        }
        publish_batches.push(PublishBatch { cards, audio });
    }
    let material = PublishMaterial {
        profile: context.profile,
        display: context.display,
        batches: publish_batches,
    };
    let destination = request.destination.unwrap_or(context.package_destination);
    let artifact = deps.publisher.publish(&destination, &material)?;
    Ok(PackageReport {
        artifact,
        batches: material.batches.len(),
        cards: card_count,
        next: NextAction::Export,
    })
}
