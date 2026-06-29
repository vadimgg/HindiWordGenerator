use lingo_domain::{
    AudioBackendId, AudioFailureClass, AudioFormat, Batch, BatchId, BatchName, CollectionId,
    CollectionTitle, DisplayPolicy, FieldAuthoritySet, Gloss, LanguageCode, LanguageProfile,
    LiteralGloss, Register, Romanisation, RunId, SectionName, Sentence, SentenceAudio, SentenceId,
    SentenceProvenance, SentenceStatus, SentenceTags, TargetText, TokenBreakdown, WordEntry,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum LibraryFailure {
    #[error("library item was not found: {0}")]
    NotFound(String),
    #[error("library data is invalid: {0}")]
    Invalid(String),
    #[error("library I/O failed: {0}")]
    Io(String),
    #[error("library SQL failed: {0}")]
    Sql(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibrarySummary {
    pub collections: usize,
    pub sentences: usize,
    pub draft: usize,
    pub enriching: usize,
    pub enriched: usize,
    pub words: usize,
    pub audio: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SentenceSelection {
    All,
    Collection(CollectionId),
    Section { collection: CollectionId, section: SectionName },
    Status(SentenceStatus),
    MissingAudio,
    ReadyToPublish,
    Ids(Vec<SentenceId>),
    Batch(BatchId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SentenceSort {
    LibraryOrder,
    UpdatedDesc,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageRequest {
    pub limit: usize,
    pub offset: usize,
}

impl Default for PageRequest {
    fn default() -> Self { Self { limit: 500, offset: 0 } }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SentenceQuery {
    pub selection: SentenceSelection,
    pub sort: SentenceSort,
    pub page: PageRequest,
}

impl Default for SentenceQuery {
    fn default() -> Self {
        Self { selection: SentenceSelection::All, sort: SentenceSort::LibraryOrder, page: PageRequest::default() }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SentencePage {
    pub sentences: Vec<Sentence>,
    pub total: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftSentence {
    pub target: TargetText,
    pub romanisation: Option<Romanisation>,
    pub english: Option<Gloss>,
    pub authority: FieldAuthoritySet,
    pub tags: SentenceTags,
    pub provenance: SentenceProvenance,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsertDrafts {
    pub collection: CollectionId,
    pub title: CollectionTitle,
    pub language: LanguageCode,
    /// Batch these drafts belong to (None = legacy/no batch).
    pub batch_id: Option<BatchId>,
    /// Per-sentence title override applied to every draft (usually None — sentences
    /// inherit the batch's default title).
    pub sentence_title: Option<SectionName>,
    /// Per-sentence subtitle override applied to every draft.
    pub section: Option<SectionName>,
    pub drafts: Vec<DraftSentence>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitReport {
    pub created: usize,
    pub updated: usize,
    pub ids: Vec<SentenceId>,
}

/// A fully-formed sentence brought in from a package (already enriched, with its
/// own field authority and breakdown). The store assigns a fresh id and order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportSentenceInput {
    pub batch_id: Option<BatchId>,
    pub title: Option<SectionName>,
    pub section: Option<SectionName>,
    pub target: TargetText,
    pub romanisation: Option<Romanisation>,
    pub english: Option<Gloss>,
    pub literal: Option<LiteralGloss>,
    pub register: Option<Register>,
    pub authority: FieldAuthoritySet,
    pub tags: SentenceTags,
    pub breakdown: Option<TokenBreakdown>,
    pub status: SentenceStatus,
    pub provenance: SentenceProvenance,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportSentences {
    pub collection: CollectionId,
    pub title: CollectionTitle,
    pub language: LanguageCode,
    pub sentences: Vec<ImportSentenceInput>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimEnrichment {
    pub run_id: RunId,
    pub selection: SentenceSelection,
    pub limit: usize,
    pub force: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnrichmentRun {
    pub run_id: RunId,
    pub sentences: Vec<Sentence>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyEnrichmentCommit {
    pub run_id: RunId,
    pub enrichments: Vec<lingo_domain::SentenceEnrichment>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResetEnrichmentClaim {
    pub run_id: Option<RunId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResetReport {
    pub reset: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReorderSentences {
    pub collection: CollectionId,
    pub ordered_ids: Vec<SentenceId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReorderReport {
    pub moved: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateSentence {
    pub id: SentenceId,
    pub section: Option<Option<SectionName>>,
    pub target: Option<TargetText>,
    pub romanisation: Option<Option<Romanisation>>,
    pub english: Option<Option<Gloss>>,
    pub literal: Option<Option<LiteralGloss>>,
    pub register: Option<Option<Register>>,
    pub tags: Option<SentenceTags>,
    /// Lifecycle transition (e.g. confirm `enriched` → `active`).
    pub status: Option<SentenceStatus>,
    /// Move to a batch (outer Some = change it, inner None = detach).
    pub batch_id: Option<Option<BatchId>>,
    /// Per-sentence title override (outer Some = change it, inner None = clear).
    pub title: Option<Option<SectionName>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SentenceReport {
    pub sentence: Sentence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateBatch {
    pub collection: CollectionId,
    pub title: CollectionTitle,
    pub language: LanguageCode,
    pub name: BatchName,
    pub default_title: Option<SectionName>,
    pub default_subtitle: Option<SectionName>,
    pub raw_text: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppendBatchRawText {
    pub batch_id: BatchId,
    pub raw_text: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchReport {
    pub batch: Batch,
}

/// A batch plus a count of its sentences by lifecycle status (for the Generate
/// page and view_state).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchSummary {
    pub batch: Batch,
    pub draft: usize,
    pub enriching: usize,
    pub enriched: usize,
    pub active: usize,
    pub total: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchPage {
    pub batches: Vec<BatchSummary>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WordQuery {
    pub collection: Option<CollectionId>,
    pub min_count: Option<usize>,
    pub page: PageRequest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WordRow {
    pub word: WordEntry,
    pub sentence_count: usize,
    pub meanings: Vec<Gloss>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WordPage {
    pub words: Vec<WordRow>,
    pub total: usize,
}

pub trait LibraryStore {
    fn summary(&self) -> Result<LibrarySummary, LibraryFailure>;
    fn list_sentences(&self, query: &SentenceQuery) -> Result<SentencePage, LibraryFailure>;
    fn get_sentence(&self, id: &SentenceId) -> Result<Sentence, LibraryFailure>;
    fn insert_drafts(&self, request: InsertDrafts) -> Result<CommitReport, LibraryFailure>;
    fn import_sentences(&self, request: ImportSentences) -> Result<CommitReport, LibraryFailure>;
    fn claim_for_enrichment(&self, request: ClaimEnrichment) -> Result<EnrichmentRun, LibraryFailure>;
    fn apply_enrichment(&self, request: ApplyEnrichmentCommit) -> Result<CommitReport, LibraryFailure>;
    fn reset_enrichment_claim(&self, request: ResetEnrichmentClaim) -> Result<ResetReport, LibraryFailure>;
    fn reorder(&self, request: ReorderSentences) -> Result<ReorderReport, LibraryFailure>;
    fn update_sentence(&self, request: UpdateSentence) -> Result<SentenceReport, LibraryFailure>;
    fn create_batch(&self, request: CreateBatch) -> Result<BatchReport, LibraryFailure>;
    fn append_batch_raw_text(&self, request: AppendBatchRawText) -> Result<BatchReport, LibraryFailure>;
    fn list_batches(&self, collection: Option<&CollectionId>) -> Result<BatchPage, LibraryFailure>;
    fn get_batch(&self, id: &BatchId) -> Result<Batch, LibraryFailure>;
    fn set_audio(&self, sentence: &SentenceId, audio: SentenceAudio) -> Result<(), LibraryFailure>;
    fn list_words(&self, query: &WordQuery) -> Result<WordPage, LibraryFailure>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeckContext {
    pub profile: LanguageProfile,
    pub display: DisplayPolicy,
    pub default_collection: CollectionId,
    pub default_title: CollectionTitle,
    pub package_destination: PathBuf,
    pub exports_directory: PathBuf,
    pub audio_backend: AudioBackendId,
    /// Workspace root; lets publishers resolve audio files and the source db.
    pub root: PathBuf,
}

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum ContextFailure {
    #[error("deck context is invalid: {0}")]
    Invalid(String),
    #[error("deck context I/O failed: {0}")]
    Io(String),
}

pub trait DeckContextProvider {
    fn resolve(&self) -> Result<DeckContext, ContextFailure>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptPacket {
    pub run_id: RunId,
    pub content: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtractPromptRequest {
    pub run_id: RunId,
    pub raw: String,
    pub collection_title: CollectionTitle,
    pub section: Option<SectionName>,
    pub context: DeckContext,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExtractSentenceDraft {
    pub target: String,
    pub romanisation: Option<String>,
    pub english: Option<String>,
    #[serde(default)]
    pub authority: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtractDraft {
    pub sentences: Vec<ExtractSentenceDraft>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnrichPromptRequest {
    pub run: EnrichmentRun,
    pub context: DeckContext,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EnrichSentenceDraft {
    pub id: String,
    pub romanisation: Option<String>,
    pub english: Option<String>,
    pub literal: Option<String>,
    pub register: Option<String>,
    #[serde(default)]
    pub breakdown: Vec<EnrichBreakdownDraft>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EnrichBreakdownDraft {
    pub surface: String,
    pub roman: Option<String>,
    pub gloss: String,
    pub kind: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnrichDraft {
    pub sentences: Vec<EnrichSentenceDraft>,
}

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum PromptFailure {
    #[error("prompt rendering failed: {0}")]
    Render(String),
    #[error("model reply is invalid: {0}")]
    InvalidReply(String),
}

pub trait PromptEngine {
    fn render_extract(&self, request: &ExtractPromptRequest) -> Result<PromptPacket, PromptFailure>;
    fn parse_extract_reply(&self, reply: &str) -> Result<ExtractDraft, PromptFailure>;
    fn render_enrich(&self, request: &EnrichPromptRequest) -> Result<PromptPacket, PromptFailure>;
    fn parse_enrich_reply(&self, reply: &str) -> Result<EnrichDraft, PromptFailure>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioRequest {
    pub sentence: SentenceId,
    pub text: String,
    pub language: LanguageCode,
    pub backend: AudioBackendId,
    pub voice: Option<String>,
    pub model: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SynthesizedAudio {
    pub bytes: Vec<u8>,
    pub backend: AudioBackendId,
    pub format: AudioFormat,
    pub voice: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Error, Clone, Eq, PartialEq)]
#[error("audio synthesis failed ({class:?}): {message}")]
pub struct AudioFailure {
    pub class: AudioFailureClass,
    pub message: String,
}

pub trait AudioSynthesizer {
    fn synthesize(&self, request: &AudioRequest) -> Result<SynthesizedAudio, AudioFailure>;
}

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum AudioFileFailure {
    #[error("audio file I/O failed: {0}")]
    Io(String),
    #[error("audio bytes failed verification")]
    Verification,
}

pub trait AudioFileStore {
    fn write_sentence_audio(&self, sentence: &SentenceId, audio: &SynthesizedAudio) -> Result<SentenceAudio, AudioFileFailure>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishMaterial {
    pub profile: LanguageProfile,
    pub display: DisplayPolicy,
    pub sentences: Vec<Sentence>,
    /// Workspace root the sentences came from; audio files and the source
    /// `library.db` are resolved relative to it.
    pub source_root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishedArtifact {
    pub path: PathBuf,
    pub files: usize,
    pub bytes: u64,
}

#[derive(Debug, Error, Clone, Eq, PartialEq)]
#[error("artifact publication failed: {0}")]
pub struct ArtifactFailure(pub String);

pub trait PackagePublisher {
    fn publish_json(&self, destination: &Path, material: &PublishMaterial) -> Result<PublishedArtifact, ArtifactFailure>;
    fn publish_db_copy(&self, destination: &Path, material: &PublishMaterial) -> Result<PublishedArtifact, ArtifactFailure>;
}

pub trait AnkiExporter {
    fn export_apkg(&self, destination: &Path, deck: &str, material: &PublishMaterial) -> Result<PublishedArtifact, ArtifactFailure>;
}
