use lingo_domain::{
    AudioBackendId, AudioFailureClass, AudioFormat, AudioRef, BatchId, CardBatch, CardId,
    DisplayPolicy, LanguageCode, LanguageProfile, PipelineStage, ProfileId, RawDocumentId, RunId,
    SourceBatch, SourceSubtitle, SourceTitle,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredFile {
    relative_path: PathBuf,
}

impl StoredFile {
    pub fn new(relative_path: PathBuf) -> Self {
        Self { relative_path }
    }

    pub fn relative_path(&self) -> &Path {
        &self.relative_path
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawDocument {
    id: RawDocumentId,
    relative_path: PathBuf,
    text: String,
}

impl RawDocument {
    pub fn new(id: RawDocumentId, relative_path: PathBuf, text: String) -> Self {
        Self {
            id,
            relative_path,
            text,
        }
    }

    pub fn id(&self) -> &RawDocumentId {
        &self.id
    }

    pub fn relative_path(&self) -> &Path {
        &self.relative_path
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawDocumentSummary {
    pub id: RawDocumentId,
    pub relative_path: PathBuf,
    pub batch_hint: Option<BatchId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchSnapshot {
    pub batch: BatchId,
    pub source_present: bool,
    pub cards_present: bool,
    pub card_count: usize,
    pub audio_present: usize,
    pub malformed_source: Option<String>,
    pub malformed_cards: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkspaceSnapshot {
    pub raw: Vec<RawDocumentSummary>,
    pub batches: Vec<BatchSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BootstrapChangeKind {
    Created,
    Kept,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootstrapChange {
    pub relative_path: PathBuf,
    pub kind: BootstrapChangeKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootstrapChanges {
    pub root: PathBuf,
    pub entries: Vec<BootstrapChange>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LearnerContext {
    pub native_languages: Vec<String>,
    pub location: Option<String>,
    pub goal: Option<String>,
    pub notes: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigOrigin {
    BuiltIn,
    Global,
    Profile,
    Deck,
    DeckProfile,
    Command,
}

impl ConfigOrigin {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::BuiltIn => "built-in",
            Self::Global => "global",
            Self::Profile => "profile",
            Self::Deck => "deck",
            Self::DeckProfile => "deck-profile",
            Self::Command => "command",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedPrompt {
    pub content: String,
    pub origin: ConfigOrigin,
    pub origin_path: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioSettings {
    pub primary: AudioBackendId,
    pub fallback: Option<AudioBackendId>,
    pub language: LanguageCode,
    pub elevenlabs_voice: Option<String>,
    pub elevenlabs_model: Option<String>,
    pub elevenlabs_key_env: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeckContext {
    pub profile: LanguageProfile,
    pub learner: LearnerContext,
    pub display: DisplayPolicy,
    pub audio: AudioSettings,
    pub package_destination: PathBuf,
    pub exports_directory: PathBuf,
    pub export_deck: Option<String>,
    pub import_prompt: ResolvedPrompt,
    pub build_prompt: ResolvedPrompt,
    pub field_origins: BTreeMap<String, ConfigOrigin>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileSummary {
    pub id: ProfileId,
    pub language: String,
    pub code: String,
    pub romanisation: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileDefinition {
    pub profile: LanguageProfile,
    pub import_prompt: ResolvedPrompt,
    pub build_prompt: ResolvedPrompt,
    pub default_elevenlabs_voice: Option<String>,
    pub default_elevenlabs_model: Option<String>,
    pub field_origins: BTreeMap<String, ConfigOrigin>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WorkspaceFailure {
    #[error("workspace item was not found: {0}")]
    NotFound(String),
    #[error("workspace item already exists: {0}")]
    Collision(String),
    #[error("workspace data is invalid: {0}")]
    InvalidData(String),
    #[error("workspace I/O failed: {0}")]
    Io(String),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ContextFailure {
    #[error("deck context could not be resolved: {0}")]
    Invalid(String),
    #[error("deck context I/O failed: {0}")]
    Io(String),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProfileFailure {
    #[error("unknown language profile: {0}")]
    Unknown(ProfileId),
    #[error("profile catalog is invalid: {0}")]
    Invalid(String),
    #[error("profile I/O failed: {0}")]
    Io(String),
}

pub trait WorkspaceBootstrap {
    fn create_missing(
        &self,
        target: &Path,
        profile: &ProfileDefinition,
    ) -> Result<BootstrapChanges, WorkspaceFailure>;
}

pub trait ProfileCatalog {
    fn list(&self) -> Result<Vec<ProfileSummary>, ProfileFailure>;
    fn require(&self, profile: &ProfileId) -> Result<ProfileDefinition, ProfileFailure>;
}

pub trait DeckContextProvider {
    fn resolve(&self) -> Result<DeckContext, ContextFailure>;
}

pub trait WorkspaceStore {
    fn load_raw(&self, raw: &RawDocumentId) -> Result<RawDocument, WorkspaceFailure>;
    fn list_raw(&self) -> Result<Vec<RawDocumentSummary>, WorkspaceFailure>;
    fn load_source(&self, batch: &BatchId) -> Result<SourceBatch, WorkspaceFailure>;
    fn list_source_batches(&self) -> Result<Vec<BatchId>, WorkspaceFailure>;
    fn create_source(&self, source: &SourceBatch) -> Result<StoredFile, WorkspaceFailure>;
    fn replace_source(&self, source: &SourceBatch) -> Result<StoredFile, WorkspaceFailure>;
    fn load_cards(&self, batch: &BatchId) -> Result<CardBatch, WorkspaceFailure>;
    fn list_card_batches(&self) -> Result<Vec<BatchId>, WorkspaceFailure>;
    fn create_cards(&self, cards: &CardBatch) -> Result<StoredFile, WorkspaceFailure>;
    fn replace_cards(&self, cards: &CardBatch) -> Result<StoredFile, WorkspaceFailure>;
    fn write_audio(
        &self,
        card: &CardId,
        audio: &SynthesizedAudio,
    ) -> Result<AudioRef, WorkspaceFailure>;
    fn read_audio(&self, audio: &AudioRef) -> Result<Vec<u8>, WorkspaceFailure>;
    fn scan(&self) -> Result<WorkspaceSnapshot, WorkspaceFailure>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptStage {
    Import,
    Build,
}

impl PromptStage {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Import => "import",
            Self::Build => "build",
        }
    }

    pub const fn pipeline_stage(self) -> PipelineStage {
        match self {
            Self::Import => PipelineStage::Source,
            Self::Build => PipelineStage::Cards,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptPacket {
    pub stage: PromptStage,
    pub content: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportPromptRequest {
    pub batch: BatchId,
    pub title: SourceTitle,
    pub subtitle: Option<SourceSubtitle>,
    pub raw: RawDocument,
    pub context: DeckContext,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildPromptRequest {
    pub source: SourceBatch,
    pub context: DeckContext,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportDraftItem {
    pub target: String,
    pub romanisation: Option<String>,
    pub english: String,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceBatchDraft {
    pub items: Vec<ImportDraftItem>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenDraft {
    pub target: String,
    pub romanisation: Option<String>,
    pub word_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WordDraft {
    pub id: String,
    pub target: String,
    pub romanisation: Option<String>,
    pub meaning: String,
    pub kind: String,
    pub grammar: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CardDraft {
    pub source_item: String,
    pub literal: String,
    pub register: String,
    pub tokens: Vec<TokenDraft>,
    pub words: Vec<WordDraft>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CardBatchDraft {
    pub cards: Vec<CardDraft>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PromptFailure {
    #[error("prompt rendering failed: {0}")]
    Render(String),
    #[error("model reply is invalid: {0}")]
    InvalidReply(String),
}

pub trait PromptEngine {
    fn render_import(&self, request: &ImportPromptRequest) -> Result<PromptPacket, PromptFailure>;
    fn parse_import_reply(&self, reply: &str) -> Result<SourceBatchDraft, PromptFailure>;
    fn render_build(&self, request: &BuildPromptRequest) -> Result<PromptPacket, PromptFailure>;
    fn parse_build_reply(&self, reply: &str) -> Result<CardBatchDraft, PromptFailure>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedRun {
    pub id: RunId,
    pub stage: PromptStage,
    pub batch: BatchId,
    pub profile: ProfileId,
    pub title: SourceTitle,
    pub subtitle: Option<SourceSubtitle>,
    pub packet: PromptPacket,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunRecord {
    pub id: RunId,
    pub directory: PathBuf,
    pub prompt_path: PathBuf,
    pub reply_path: PathBuf,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RunFailure {
    #[error("prepared run was not found: {0}")]
    NotFound(RunId),
    #[error("run journal failed: {0}")]
    Io(String),
    #[error("run metadata is invalid: {0}")]
    Invalid(String),
}

pub trait RunJournal {
    fn record_prepared(&self, run: &PreparedRun) -> Result<RunRecord, RunFailure>;
    fn require_prepared(&self, run: &RunId) -> Result<PreparedRun, RunFailure>;
    fn record_applied(
        &self,
        run: &RunId,
        reply: &str,
        stored: &StoredFile,
    ) -> Result<(), RunFailure>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioRequest {
    pub card: CardId,
    pub text: String,
    pub language: LanguageCode,
    pub primary: AudioBackendId,
    pub fallback: Option<AudioBackendId>,
    pub elevenlabs_voice: Option<String>,
    pub elevenlabs_model: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SynthesizedAudio {
    pub bytes: Vec<u8>,
    pub backend: AudioBackendId,
    pub format: AudioFormat,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("audio synthesis failed ({class:?}): {message}")]
pub struct AudioFailure {
    pub class: AudioFailureClass,
    pub message: String,
}

pub trait AudioSynthesizer {
    fn synthesize(&self, request: &AudioRequest) -> Result<SynthesizedAudio, AudioFailure>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioAsset {
    pub reference: AudioRef,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishBatch {
    pub cards: CardBatch,
    pub audio: Vec<AudioAsset>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishMaterial {
    pub profile: LanguageProfile,
    pub display: DisplayPolicy,
    pub batches: Vec<PublishBatch>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishedArtifact {
    pub path: PathBuf,
    pub files: usize,
    pub bytes: u64,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("artifact publication failed: {0}")]
pub struct ArtifactFailure(pub String);

pub trait PackagePublisher {
    fn publish(
        &self,
        destination: &Path,
        material: &PublishMaterial,
    ) -> Result<PublishedArtifact, ArtifactFailure>;
}

pub trait AnkiExporter {
    fn export(
        &self,
        destination: &Path,
        deck: &str,
        material: &PublishMaterial,
    ) -> Result<PublishedArtifact, ArtifactFailure>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityKind {
    Workspace,
    Editor,
    Clipboard,
    Gtts,
    ElevenLabsKey,
    Node,
}

impl CapabilityKind {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Workspace => "workspace",
            Self::Editor => "editor",
            Self::Clipboard => "clipboard",
            Self::Gtts => "gtts",
            Self::ElevenLabsKey => "elevenlabs_key",
            Self::Node => "node",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequiredCapabilities {
    pub kinds: Vec<CapabilityKind>,
    pub elevenlabs_key_env: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityCheck {
    pub kind: CapabilityKind,
    pub required: bool,
    pub available: bool,
    pub detail: Option<String>,
    pub recovery: Option<String>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("environment probe failed: {0}")]
pub struct EnvironmentFailure(pub String);

pub trait EnvironmentProbe {
    fn probe(
        &self,
        required: &RequiredCapabilities,
    ) -> Result<Vec<CapabilityCheck>, EnvironmentFailure>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverrideScope {
    Global,
    Deck,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptOverrideTarget {
    pub path: PathBuf,
    pub created: bool,
}

pub trait ProfileOverrideStore {
    fn create_prompt_override(
        &self,
        profile: &ProfileId,
        stage: PromptStage,
        scope: OverrideScope,
    ) -> Result<PromptOverrideTarget, ProfileFailure>;
}
