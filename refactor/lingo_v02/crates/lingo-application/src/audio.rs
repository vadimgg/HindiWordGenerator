use crate::ports::{
    AudioFailure, AudioFileFailure, AudioFileStore, AudioRequest, AudioSynthesizer,
    DeckContextProvider, LibraryFailure, LibraryStore, PageRequest, SentenceQuery, SentenceSelection,
    SentenceSort,
};
use crate::report::NextAction;
use lingo_domain::{AudioBackendId, SentenceId};
use thiserror::Error;

pub struct AudioDeps<'a> {
    pub library: &'a dyn LibraryStore,
    pub context: &'a dyn DeckContextProvider,
    pub synthesizer: &'a dyn AudioSynthesizer,
    pub files: &'a dyn AudioFileStore,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioMode { MissingOnly, ReplaceAll }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioCommand {
    pub selection: SentenceSelection,
    pub mode: AudioMode,
    pub backend: Option<AudioBackendId>,
    pub voice: Option<String>,
    pub model: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioSentenceResult {
    pub sentence: SentenceId,
    pub backend: Option<AudioBackendId>,
    pub skipped: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioReport {
    pub results: Vec<AudioSentenceResult>,
    pub updated: usize,
    pub skipped: usize,
    pub next: NextAction,
}

#[derive(Debug, Error)]
pub enum AudioError {
    #[error(transparent)] Library(#[from] LibraryFailure),
    #[error(transparent)] Context(#[from] crate::ports::ContextFailure),
    #[error(transparent)] Synthesis(#[from] AudioFailure),
    #[error(transparent)] Files(#[from] AudioFileFailure),
}

pub fn synthesize_audio(deps: &AudioDeps<'_>, request: AudioCommand) -> Result<AudioReport, AudioError> {
    let context = deps.context.resolve()?;
    let page = deps.library.list_sentences(&SentenceQuery {
        selection: request.selection,
        sort: SentenceSort::LibraryOrder,
        page: PageRequest { limit: 10_000, offset: 0 },
    })?;
    let mut results = Vec::new();
    let mut updated = 0usize;
    let mut skipped = 0usize;
    for sentence in page.sentences {
        if request.mode == AudioMode::MissingOnly && sentence.audio().is_some() {
            skipped += 1;
            results.push(AudioSentenceResult { sentence: sentence.id().clone(), backend: sentence.audio().map(|audio| audio.backend()), skipped: true });
            continue;
        }
        let backend = request.backend.unwrap_or(context.audio_backend);
        let audio = deps.synthesizer.synthesize(&AudioRequest {
            sentence: sentence.id().clone(),
            text: sentence.target().as_str().to_string(),
            language: context.profile.code().clone(),
            backend,
            voice: request.voice.clone(),
            model: request.model.clone(),
        })?;
        let attachment = deps.files.write_sentence_audio(sentence.id(), &audio)?;
        deps.library.set_audio(sentence.id(), attachment)?;
        updated += 1;
        results.push(AudioSentenceResult { sentence: sentence.id().clone(), backend: Some(audio.backend), skipped: false });
    }
    Ok(AudioReport { results, updated, skipped, next: NextAction::Package })
}
