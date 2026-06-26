use crate::atomic_file::{AtomicFileError, create_atomic, replace_atomic};
use crate::codecs::{decode_cards, decode_source, encode_cards, encode_source, map_codec_path};
use crate::config::resolve_context;
use crate::layout::WorkspaceLayout;
use crate::profiles::FsProfileCatalog;
use crate::root::{RootError, WorkspaceRoot};
use crate::scan::scan_workspace;
use lingo_application::ports::{
    BootstrapChange, BootstrapChangeKind, BootstrapChanges, DeckContext, DeckContextProvider,
    ProfileDefinition, RawDocument, RawDocumentSummary, StoredFile, SynthesizedAudio,
    WorkspaceBootstrap, WorkspaceFailure, WorkspaceSnapshot, WorkspaceStore,
};
use lingo_domain::{
    AudioRef, AudioRelativePath, BatchId, CardBatch, CardId, ContentHash, RawDocumentId,
    SourceBatch, content_hash,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct FsWorkspace {
    root: WorkspaceRoot,
    layout: WorkspaceLayout,
    global_config_dir: PathBuf,
}

impl FsWorkspace {
    pub fn discover(from: &Path, global_config_dir: PathBuf) -> Result<Self, WorkspaceFailure> {
        let root = WorkspaceRoot::discover(from).map_err(map_root)?;
        Ok(Self::new(root, global_config_dir))
    }

    pub fn open(root: WorkspaceRoot, global_config_dir: PathBuf) -> Self {
        Self::new(root, global_config_dir)
    }

    fn new(root: WorkspaceRoot, global_config_dir: PathBuf) -> Self {
        let layout = WorkspaceLayout::new(root.clone());
        Self {
            root,
            layout,
            global_config_dir,
        }
    }

    pub fn root(&self) -> &WorkspaceRoot {
        &self.root
    }

    pub fn layout(&self) -> &WorkspaceLayout {
        &self.layout
    }

    pub fn profile_catalog(&self) -> FsProfileCatalog {
        FsProfileCatalog::for_workspace(self.global_config_dir.clone(), self.root.clone())
    }

    pub(crate) fn stored_file(&self, path: &Path) -> StoredFile {
        StoredFile::new(
            path.strip_prefix(self.root.path())
                .unwrap_or(path)
                .to_path_buf(),
        )
    }
}

impl DeckContextProvider for FsWorkspace {
    fn resolve(&self) -> Result<DeckContext, lingo_application::ports::ContextFailure> {
        resolve_context(&self.layout, &self.global_config_dir)
    }
}

impl WorkspaceStore for FsWorkspace {
    fn load_raw(&self, raw: &RawDocumentId) -> Result<RawDocument, WorkspaceFailure> {
        let summaries = self.list_raw()?;
        let summary = summaries
            .into_iter()
            .find(|summary| &summary.id == raw)
            .ok_or_else(|| WorkspaceFailure::NotFound(format!("raw document {raw}")))?;
        let path = self.root.join(&summary.relative_path);
        let text = fs::read_to_string(&path).map_err(|error| io_failure(&path, error))?;
        Ok(RawDocument::new(summary.id, summary.relative_path, text))
    }

    fn list_raw(&self) -> Result<Vec<RawDocumentSummary>, WorkspaceFailure> {
        let mut by_id = BTreeMap::<RawDocumentId, RawDocumentSummary>::new();
        for entry in read_dir(&self.layout.raw_dir())? {
            let path = entry.path();
            if !path.is_file() || !is_raw_extension(&path) {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|value| value.to_str())
                .ok_or_else(|| {
                    WorkspaceFailure::InvalidData(format!(
                        "raw filename is not UTF-8: {}",
                        path.display()
                    ))
                })?;
            let slug = slug(stem);
            let id = RawDocumentId::parse(slug.clone())
                .map_err(|error| WorkspaceFailure::InvalidData(error.to_string()))?;
            let batch_hint = BatchId::parse(slug).ok();
            let relative_path = path
                .strip_prefix(self.root.path())
                .unwrap_or(&path)
                .to_path_buf();
            if by_id
                .insert(
                    id.clone(),
                    RawDocumentSummary {
                        id: id.clone(),
                        relative_path,
                        batch_hint,
                    },
                )
                .is_some()
            {
                return Err(WorkspaceFailure::InvalidData(format!(
                    "multiple raw files normalize to id {id}"
                )));
            }
        }
        Ok(by_id.into_values().collect())
    }

    fn load_source(&self, batch: &BatchId) -> Result<SourceBatch, WorkspaceFailure> {
        let path = self.layout.source_file(batch);
        let bytes = fs::read(&path).map_err(|error| io_or_not_found(&path, error))?;
        decode_source(&bytes).map_err(|error| map_codec_path(path, error))
    }

    fn list_source_batches(&self) -> Result<Vec<BatchId>, WorkspaceFailure> {
        list_batches(&self.layout.source_dir(), "yaml")
    }

    fn create_source(&self, source: &SourceBatch) -> Result<StoredFile, WorkspaceFailure> {
        let path = self.layout.source_file(source.batch_id());
        let bytes = encode_source(source).map_err(|error| map_codec_path(path.clone(), error))?;
        create_atomic(&path, &bytes).map_err(map_atomic)?;
        Ok(self.stored_file(&path))
    }

    fn replace_source(&self, source: &SourceBatch) -> Result<StoredFile, WorkspaceFailure> {
        let path = self.layout.source_file(source.batch_id());
        let bytes = encode_source(source).map_err(|error| map_codec_path(path.clone(), error))?;
        replace_atomic(&path, &bytes).map_err(map_atomic)?;
        Ok(self.stored_file(&path))
    }

    fn load_cards(&self, batch: &BatchId) -> Result<CardBatch, WorkspaceFailure> {
        let path = self.layout.card_file(batch);
        let bytes = fs::read(&path).map_err(|error| io_or_not_found(&path, error))?;
        decode_cards(&bytes).map_err(|error| map_codec_path(path, error))
    }

    fn list_card_batches(&self) -> Result<Vec<BatchId>, WorkspaceFailure> {
        list_batches(&self.layout.card_dir(), "json")
    }

    fn create_cards(&self, cards: &CardBatch) -> Result<StoredFile, WorkspaceFailure> {
        let path = self.layout.card_file(cards.batch_id());
        let bytes = encode_cards(cards).map_err(|error| map_codec_path(path.clone(), error))?;
        create_atomic(&path, &bytes).map_err(map_atomic)?;
        Ok(self.stored_file(&path))
    }

    fn replace_cards(&self, cards: &CardBatch) -> Result<StoredFile, WorkspaceFailure> {
        let path = self.layout.card_file(cards.batch_id());
        let bytes = encode_cards(cards).map_err(|error| map_codec_path(path.clone(), error))?;
        replace_atomic(&path, &bytes).map_err(map_atomic)?;
        Ok(self.stored_file(&path))
    }

    fn write_audio(
        &self,
        card: &CardId,
        audio: &SynthesizedAudio,
    ) -> Result<AudioRef, WorkspaceFailure> {
        let path = self.layout.audio_file(card);
        let Some(parent) = path.parent() else {
            return Err(WorkspaceFailure::Io("audio path has no parent".into()));
        };
        fs::create_dir_all(parent).map_err(|error| io_failure(parent, error))?;
        if path.exists() {
            replace_atomic(&path, &audio.bytes).map_err(map_atomic)?;
        } else {
            create_atomic(&path, &audio.bytes).map_err(map_atomic)?;
        }
        let relative = path
            .strip_prefix(self.root.path())
            .map_err(|_| WorkspaceFailure::InvalidData("audio escaped workspace root".into()))?
            .to_string_lossy()
            .replace('\\', "/");
        let relative_path = AudioRelativePath::parse(relative)
            .map_err(|error| WorkspaceFailure::InvalidData(error.to_string()))?;
        let hash: ContentHash = content_hash(&audio.bytes);
        Ok(AudioRef::new(
            card.clone(),
            relative_path,
            hash,
            audio.backend,
            audio.format,
        ))
    }

    fn read_audio(&self, audio: &AudioRef) -> Result<Vec<u8>, WorkspaceFailure> {
        let path = self.root.join(audio.relative_path().as_str());
        let bytes = fs::read(&path).map_err(|error| io_or_not_found(&path, error))?;
        if content_hash(&bytes) != *audio.content_hash() {
            return Err(WorkspaceFailure::InvalidData(format!(
                "audio content hash mismatch at {}",
                path.display()
            )));
        }
        Ok(bytes)
    }

    fn scan(&self) -> Result<WorkspaceSnapshot, WorkspaceFailure> {
        scan_workspace(self)
    }
}

#[derive(Clone, Debug)]
pub struct FsWorkspaceBootstrap {
    global_config_dir: PathBuf,
}

impl FsWorkspaceBootstrap {
    pub fn new(global_config_dir: PathBuf) -> Self {
        Self { global_config_dir }
    }
}

impl WorkspaceBootstrap for FsWorkspaceBootstrap {
    fn create_missing(
        &self,
        target: &Path,
        profile: &ProfileDefinition,
    ) -> Result<BootstrapChanges, WorkspaceFailure> {
        let root = WorkspaceRoot::new_target(target).map_err(map_root)?;
        let layout = WorkspaceLayout::new(root.clone());
        let mut entries = Vec::new();
        for directory in layout.all_directories() {
            let relative = directory
                .strip_prefix(root.path())
                .unwrap_or(&directory)
                .to_path_buf();
            let kind = if directory.is_dir() {
                BootstrapChangeKind::Kept
            } else {
                fs::create_dir_all(&directory).map_err(|error| io_failure(&directory, error))?;
                BootstrapChangeKind::Created
            };
            entries.push(BootstrapChange {
                relative_path: relative,
                kind,
            });
        }
        let config = layout.config_file();
        let config_kind = if config.is_file() {
            BootstrapChangeKind::Kept
        } else {
            let content = starter_config(profile);
            create_atomic(&config, content.as_bytes()).map_err(map_atomic)?;
            BootstrapChangeKind::Created
        };
        entries.insert(
            0,
            BootstrapChange {
                relative_path: PathBuf::from("config.toml"),
                kind: config_kind,
            },
        );
        let _ = &self.global_config_dir;
        Ok(BootstrapChanges {
            root: root.path().to_path_buf(),
            entries,
        })
    }
}

fn starter_config(profile: &ProfileDefinition) -> String {
    let lead = if profile.profile.romanisation().is_required() {
        "romanisation"
    } else {
        "target"
    };
    format!(
        "[target]\nprofile = \"{}\"\n\n[learner]\nnative_languages = [\"English\"]\ngoal = \"practical fluency\"\n\n[display]\nlead = \"{lead}\"\nshow_secondary = true\n\n[audio]\nbackend = \"gtts\"\n\n[audio.gtts]\nlang = \"{}\"\n\n[audio.elevenlabs]\nvoice = \"{}\"\nmodel = \"{}\"\napi_key = \"env:ELEVENLABS_API_KEY\"\n\n[package]\ndestination = \"packages/sentences\"\n\n[export]\ndeck = \"{}::Sentences\"\n",
        profile.profile.id().as_str(),
        profile.profile.code().as_str(),
        profile
            .default_elevenlabs_voice
            .as_deref()
            .unwrap_or("9BWtsMINqrJLrRacOk9x"),
        profile
            .default_elevenlabs_model
            .as_deref()
            .unwrap_or("eleven_multilingual_v2"),
        profile.profile.language().as_str(),
    )
}

fn list_batches(directory: &Path, extension: &str) -> Result<Vec<BatchId>, WorkspaceFailure> {
    let mut batches = Vec::new();
    for entry in read_dir(directory)? {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some(extension) {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or_else(|| {
                WorkspaceFailure::InvalidData(format!(
                    "batch filename is not UTF-8: {}",
                    path.display()
                ))
            })?;
        batches.push(
            BatchId::parse(stem)
                .map_err(|error| WorkspaceFailure::InvalidData(error.to_string()))?,
        );
    }
    batches.sort();
    Ok(batches)
}

fn read_dir(path: &Path) -> Result<Vec<fs::DirEntry>, WorkspaceFailure> {
    let mut entries = fs::read_dir(path)
        .map_err(|error| io_failure(path, error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| io_failure(path, error))?;
    entries.sort_by_key(fs::DirEntry::file_name);
    Ok(entries)
}

fn is_raw_extension(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|value| value.to_str()),
        Some("txt" | "md" | "text")
    )
}

fn slug(value: &str) -> String {
    let mut slug = String::new();
    let mut separator = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            separator = false;
        } else if !separator && !slug.is_empty() {
            slug.push('-');
            separator = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "raw".to_string()
    } else {
        slug
    }
}

fn map_root(error: RootError) -> WorkspaceFailure {
    WorkspaceFailure::Io(error.to_string())
}

fn map_atomic(error: AtomicFileError) -> WorkspaceFailure {
    match error {
        AtomicFileError::Collision(path) => WorkspaceFailure::Collision(path.display().to_string()),
        other => WorkspaceFailure::Io(other.to_string()),
    }
}

fn io_failure(path: &Path, error: std::io::Error) -> WorkspaceFailure {
    WorkspaceFailure::Io(format!("{}: {error}", path.display()))
}

fn io_or_not_found(path: &Path, error: std::io::Error) -> WorkspaceFailure {
    if error.kind() == std::io::ErrorKind::NotFound {
        WorkspaceFailure::NotFound(path.display().to_string())
    } else {
        io_failure(path, error)
    }
}
