use crate::ports::{
    AnkiExporter, ArtifactFailure, AudioAsset, ContextFailure, DeckContextProvider, PublishBatch,
    PublishMaterial, PublishedArtifact, WorkspaceFailure, WorkspaceStore,
};
use crate::report::NextAction;
use lingo_domain::{BatchId, CardId};
use std::path::PathBuf;
use thiserror::Error;

pub struct ExportDeps<'a> {
    pub workspace: &'a dyn WorkspaceStore,
    pub context: &'a dyn DeckContextProvider,
    pub exporter: &'a dyn AnkiExporter,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExportRequest {
    pub batches: Vec<BatchId>,
    pub all: bool,
    pub deck: Option<String>,
    pub destination: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportReport {
    pub artifact: PublishedArtifact,
    pub deck: String,
    pub batches: usize,
    pub cards: usize,
    pub next: NextAction,
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error(transparent)]
    Workspace(#[from] WorkspaceFailure),
    #[error(transparent)]
    Context(#[from] ContextFailure),
    #[error(transparent)]
    Artifact(#[from] ArtifactFailure),
    #[error("Anki deck name is missing; set [export].deck or pass --deck")]
    MissingDeck,
    #[error("card {0} has no audio; run `lingo audio` first")]
    MissingAudio(CardId),
    #[error("no card batches were selected")]
    EmptySelection,
}

pub fn export(deps: &ExportDeps<'_>, request: ExportRequest) -> Result<ExportReport, ExportError> {
    let context = deps.context.resolve()?;
    let selected = if request.all || request.batches.is_empty() {
        deps.workspace.list_card_batches()?
    } else {
        request.batches
    };
    if selected.is_empty() {
        return Err(ExportError::EmptySelection);
    }
    let deck = request
        .deck
        .or(context.export_deck)
        .ok_or(ExportError::MissingDeck)?;
    let mut publish_batches = Vec::new();
    let mut card_count = 0usize;
    for batch in selected {
        let cards = deps.workspace.load_cards(&batch)?;
        card_count += cards.cards().len();
        let mut audio = Vec::new();
        for card in cards.cards() {
            let Some(reference) = card.audio() else {
                return Err(ExportError::MissingAudio(card.id().clone()));
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
    let destination = request
        .destination
        .unwrap_or_else(|| context.exports_directory.join("lingo-sentences.apkg"));
    let artifact = deps.exporter.export(&destination, &deck, &material)?;
    Ok(ExportReport {
        artifact,
        deck,
        batches: material.batches.len(),
        cards: card_count,
        next: NextAction::None,
    })
}
