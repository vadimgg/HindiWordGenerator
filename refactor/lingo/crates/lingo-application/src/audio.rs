use crate::ports::{
    AudioFailure as PortAudioFailure, AudioRequest, AudioSynthesizer, ContextFailure,
    DeckContextProvider, StoredFile, WorkspaceFailure, WorkspaceStore,
};
use crate::report::{ChangeCounts, NextAction};
use lingo_domain::{AudioBackendId, BatchId, CardId};
use thiserror::Error;

pub struct AudioDeps<'a> {
    pub workspace: &'a dyn WorkspaceStore,
    pub context: &'a dyn DeckContextProvider,
    pub synthesizer: &'a dyn AudioSynthesizer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioMode {
    MissingOnly,
    ReplaceAll,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioCommand {
    pub batch: Option<BatchId>,
    pub mode: AudioMode,
    pub backend: Option<AudioBackendId>,
    pub elevenlabs_voice: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioCardResult {
    pub card: CardId,
    pub backend: Option<AudioBackendId>,
    pub skipped: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioBatchResult {
    pub batch: BatchId,
    pub cards: Vec<AudioCardResult>,
    pub stored: Option<StoredFile>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioReport {
    pub batches: Vec<AudioBatchResult>,
    pub counts: ChangeCounts,
    pub next: NextAction,
}

#[derive(Debug, Error)]
pub enum AudioError {
    #[error(transparent)]
    Workspace(#[from] WorkspaceFailure),
    #[error(transparent)]
    Context(#[from] ContextFailure),
    #[error(transparent)]
    Synthesis(#[from] PortAudioFailure),
    #[error("could not attach audio to card: {0}")]
    InvalidReference(String),
}

pub fn synthesize_audio(
    deps: &AudioDeps<'_>,
    request: AudioCommand,
) -> Result<AudioReport, AudioError> {
    let context = deps.context.resolve()?;
    let batches = match request.batch {
        Some(batch) => vec![batch],
        None => deps.workspace.list_card_batches()?,
    };
    let mut results = Vec::new();
    let mut counts = ChangeCounts::default();

    for batch in batches {
        let mut cards = deps.workspace.load_cards(&batch)?;
        let mut card_results = Vec::new();
        let mut changed = false;
        for card in cards.cards_mut() {
            if request.mode == AudioMode::MissingOnly && card.audio().is_some() {
                counts.skipped += 1;
                card_results.push(AudioCardResult {
                    card: card.id().clone(),
                    backend: card.audio().map(|audio| audio.backend()),
                    skipped: true,
                });
                continue;
            }
            let primary = request.backend.unwrap_or(context.audio.primary);
            let fallback = context
                .audio
                .fallback
                .filter(|fallback| *fallback != primary);
            let audio = deps.synthesizer.synthesize(&AudioRequest {
                card: card.id().clone(),
                text: card.target().as_str().to_string(),
                language: context.audio.language.clone(),
                primary,
                fallback,
                elevenlabs_voice: request
                    .elevenlabs_voice
                    .clone()
                    .or_else(|| context.audio.elevenlabs_voice.clone()),
                elevenlabs_model: context.audio.elevenlabs_model.clone(),
            })?;
            let audio_ref = deps.workspace.write_audio(card.id(), &audio)?;
            card.attach_audio(audio_ref)
                .map_err(|error| AudioError::InvalidReference(error.to_string()))?;
            counts.updated += 1;
            changed = true;
            card_results.push(AudioCardResult {
                card: card.id().clone(),
                backend: Some(audio.backend),
                skipped: false,
            });
        }
        let stored = if changed {
            Some(deps.workspace.replace_cards(&cards)?)
        } else {
            None
        };
        results.push(AudioBatchResult {
            batch,
            cards: card_results,
            stored,
        });
    }

    Ok(AudioReport {
        batches: results,
        counts,
        next: NextAction::Package,
    })
}
