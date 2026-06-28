use lingo_application::ports::{
    CapabilityCheck, CapabilityKind, DeckContextProvider, EnvironmentFailure, EnvironmentProbe,
    RequiredCapabilities, WorkspaceFailure,
};
use lingo_artifacts::{ApkgExporter, PortablePackagePublisher};
use lingo_audio::{AudioAdapterError, AudioCatalog, AudioCatalogBuilder};
use lingo_prompt::HandlebarsPromptEngine;
use lingo_workspace_fs::{FsProfileCatalog, FsRunJournal, FsWorkspace, default_global_config_dir};
use secrecy::SecretString;
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;

pub struct Composition {
    pub workspace: FsWorkspace,
    pub profiles: FsProfileCatalog,
    pub runs: FsRunJournal,
    pub prompts: HandlebarsPromptEngine,
    pub audio: AudioCatalog,
    pub package: PortablePackagePublisher,
    pub anki: ApkgExporter,
    pub environment: ProcessEnvironmentProbe,
}

#[derive(Debug, Error)]
pub enum CompositionError {
    #[error(transparent)]
    Workspace(#[from] WorkspaceFailure),
    #[error("HTTP client could not be configured: {0}")]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Audio(#[from] AudioAdapterError),
    #[error("deck context could not be resolved: {0}")]
    Context(String),
}

impl Composition {
    pub fn discover(from: &Path) -> Result<Self, CompositionError> {
        let global = default_global_config_dir();
        let workspace = FsWorkspace::discover(from, global)?;
        let profiles = workspace.profile_catalog();
        let runs = FsRunJournal::new(workspace.layout().clone());
        let context = workspace
            .resolve()
            .map_err(|error| CompositionError::Context(error.to_string()))?;
        let uv = which::which("uv").unwrap_or_else(|_| PathBuf::from("uv"));
        let mut audio_builder = AudioCatalogBuilder::new().add_gtts(uv)?;
        if let Some(name) = context.audio.elevenlabs_key_env.as_deref() {
            if let Some(value) = crate::secrets::resolve(workspace.layout().root().path(), name) {
                let client = reqwest::blocking::Client::builder()
                    .timeout(Duration::from_secs(90))
                    .build()?;
                audio_builder = audio_builder.add_elevenlabs(client, SecretString::from(value))?;
            }
        }
        Ok(Self {
            workspace,
            profiles,
            runs,
            prompts: HandlebarsPromptEngine::strict(),
            audio: audio_builder.build(),
            package: PortablePackagePublisher,
            anki: ApkgExporter,
            environment: ProcessEnvironmentProbe,
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ProcessEnvironmentProbe;

impl EnvironmentProbe for ProcessEnvironmentProbe {
    fn probe(
        &self,
        required: &RequiredCapabilities,
    ) -> Result<Vec<CapabilityCheck>, EnvironmentFailure> {
        Ok(required
            .kinds
            .iter()
            .copied()
            .map(|kind| probe_one(kind, required.elevenlabs_key_env.as_deref()))
            .collect())
    }
}

fn probe_one(kind: CapabilityKind, key_env: Option<&str>) -> CapabilityCheck {
    match kind {
        CapabilityKind::Workspace => available(kind, true, Some("config resolved".to_string())),
        CapabilityKind::Editor => {
            let editor = std::env::var("VISUAL")
                .ok()
                .or_else(|| std::env::var("EDITOR").ok());
            CapabilityCheck {
                kind,
                required: true,
                available: editor.is_some(),
                detail: editor,
                recovery: Some("set $VISUAL or $EDITOR".to_string()),
            }
        }
        CapabilityKind::Clipboard => {
            let command = clipboard_command().map(|path| path.display().to_string());
            CapabilityCheck {
                kind,
                required: true,
                available: command.is_some(),
                detail: command,
                recovery: Some("install pbcopy, wl-copy, or xclip".to_string()),
            }
        }
        CapabilityKind::Gtts => {
            let uv = which::which("uv").ok();
            CapabilityCheck {
                kind,
                required: true,
                available: uv.is_some(),
                detail: uv.map(|path| path.display().to_string()),
                recovery: Some("install uv to use the gTTS backend".to_string()),
            }
        }
        CapabilityKind::ElevenLabsKey => {
            let available = key_env.is_some_and(|name| std::env::var_os(name).is_some());
            CapabilityCheck {
                kind,
                required: true,
                available,
                detail: key_env.map(ToString::to_string),
                recovery: key_env.map(|name| format!("set ${name}")),
            }
        }
        CapabilityKind::Node => {
            let node = which::which("node").ok();
            CapabilityCheck {
                kind,
                required: false,
                available: node.is_some(),
                detail: node.map(|path| path.display().to_string()),
                recovery: Some("install Node.js to build the Astro viewer".to_string()),
            }
        }
    }
}

fn available(kind: CapabilityKind, required: bool, detail: Option<String>) -> CapabilityCheck {
    CapabilityCheck {
        kind,
        required,
        available: true,
        detail,
        recovery: None,
    }
}

pub fn clipboard_command() -> Option<PathBuf> {
    ["pbcopy", "wl-copy", "xclip"]
        .into_iter()
        .find_map(|command| which::which(command).ok())
}
