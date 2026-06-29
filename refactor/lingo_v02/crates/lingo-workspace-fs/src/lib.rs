#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
//! Filesystem adapters for local Lingo workspaces.

mod audio_files;
mod config;
mod layout;
pub mod library;

pub use audio_files::FsAudioFileStore;
pub use config::{DeckConfig, get_config_value, read_config, set_config_value, write_default};
pub use layout::WorkspaceLayout;
pub use library::SqliteLibraryStore;

use lingo_application::ports::{ContextFailure, DeckContext, DeckContextProvider, LibraryFailure, LibraryStore};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct FsWorkspace {
    layout: WorkspaceLayout,
    library: SqliteLibraryStore,
}

impl FsWorkspace {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, LibraryFailure> {
        let layout = WorkspaceLayout::new(root.into());
        let library = SqliteLibraryStore::open(layout.library_db())?;
        Ok(Self { layout, library })
    }

    pub fn init(root: impl Into<PathBuf>, profile: &str) -> Result<Self, ContextFailure> {
        let root = root.into();
        let layout = WorkspaceLayout::new(root);
        for directory in [layout.raw_dir(), layout.runs_dir(), layout.audio_dir(), layout.packages_dir(), layout.exports_dir()] {
            std::fs::create_dir_all(directory).map_err(|error| ContextFailure::Io(error.to_string()))?;
        }
        if !layout.config_file().is_file() {
            config::write_default(&layout.config_file(), profile)?;
        }
        SqliteLibraryStore::open(layout.library_db()).map_err(|error| ContextFailure::Io(error.to_string()))?;
        Self::open(layout.root()).map_err(|error| ContextFailure::Io(error.to_string()))
    }

    pub fn discover(from: impl AsRef<Path>) -> Result<Self, LibraryFailure> {
        let mut current = from.as_ref().to_path_buf();
        loop {
            if current.join("config.toml").is_file() {
                return Self::open(current);
            }
            if !current.pop() { break; }
        }
        Self::open(from.as_ref())
    }

    pub fn layout(&self) -> &WorkspaceLayout { &self.layout }
    pub fn library(&self) -> &SqliteLibraryStore { &self.library }
    pub fn audio_files(&self) -> FsAudioFileStore { FsAudioFileStore::new(self.layout.audio_dir()) }
}

impl DeckContextProvider for FsWorkspace {
    fn resolve(&self) -> Result<DeckContext, ContextFailure> {
        let config = config::read_config(&self.layout.config_file())?;
        config::context_from_config(self.layout.root(), config)
    }
}

impl LibraryStore for FsWorkspace {
    fn summary(&self) -> Result<lingo_application::ports::LibrarySummary, LibraryFailure> { self.library.summary() }
    fn list_sentences(&self, query: &lingo_application::ports::SentenceQuery) -> Result<lingo_application::ports::SentencePage, LibraryFailure> { self.library.list_sentences(query) }
    fn get_sentence(&self, id: &lingo_domain::SentenceId) -> Result<lingo_domain::Sentence, LibraryFailure> { self.library.get_sentence(id) }
    fn insert_drafts(&self, request: lingo_application::ports::InsertDrafts) -> Result<lingo_application::ports::CommitReport, LibraryFailure> { self.library.insert_drafts(request) }
    fn import_sentences(&self, request: lingo_application::ports::ImportSentences) -> Result<lingo_application::ports::CommitReport, LibraryFailure> { self.library.import_sentences(request) }
    fn claim_for_enrichment(&self, request: lingo_application::ports::ClaimEnrichment) -> Result<lingo_application::ports::EnrichmentRun, LibraryFailure> { self.library.claim_for_enrichment(request) }
    fn apply_enrichment(&self, request: lingo_application::ports::ApplyEnrichmentCommit) -> Result<lingo_application::ports::CommitReport, LibraryFailure> { self.library.apply_enrichment(request) }
    fn reset_enrichment_claim(&self, request: lingo_application::ports::ResetEnrichmentClaim) -> Result<lingo_application::ports::ResetReport, LibraryFailure> { self.library.reset_enrichment_claim(request) }
    fn reorder(&self, request: lingo_application::ports::ReorderSentences) -> Result<lingo_application::ports::ReorderReport, LibraryFailure> { self.library.reorder(request) }
    fn update_sentence(&self, request: lingo_application::ports::UpdateSentence) -> Result<lingo_application::ports::SentenceReport, LibraryFailure> { self.library.update_sentence(request) }
    fn create_batch(&self, request: lingo_application::ports::CreateBatch) -> Result<lingo_application::ports::BatchReport, LibraryFailure> { self.library.create_batch(request) }
    fn append_batch_raw_text(&self, request: lingo_application::ports::AppendBatchRawText) -> Result<lingo_application::ports::BatchReport, LibraryFailure> { self.library.append_batch_raw_text(request) }
    fn list_batches(&self, collection: Option<&lingo_domain::CollectionId>) -> Result<lingo_application::ports::BatchPage, LibraryFailure> { self.library.list_batches(collection) }
    fn get_batch(&self, id: &lingo_domain::BatchId) -> Result<lingo_domain::Batch, LibraryFailure> { self.library.get_batch(id) }
    fn set_audio(&self, sentence: &lingo_domain::SentenceId, audio: lingo_domain::SentenceAudio) -> Result<(), LibraryFailure> { self.library.set_audio(sentence, audio) }
    fn list_words(&self, query: &lingo_application::ports::WordQuery) -> Result<lingo_application::ports::WordPage, LibraryFailure> { self.library.list_words(query) }
}
