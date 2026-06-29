#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
//! Explicit audio-provider catalog.
//!
//! The production catalog synthesizes real MP3 audio with gTTS (run through
//! `uv`, so no Python environment management is needed). A deterministic offline
//! backend is available for tests and smoke flows that must not hit the network.

use lingo_application::ports::{AudioFailure, AudioRequest, AudioSynthesizer, SynthesizedAudio};
use lingo_domain::{AudioBackendId, AudioFailureClass, AudioFormat};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum AudioAdapterError {
    #[error("duplicate audio backend {0}")]
    DuplicateBackend(AudioBackendId),
}

/// An audio provider. Returns encoded MP3 bytes for a request.
trait Backend: Send + Sync {
    fn synthesize(&self, request: &AudioRequest) -> Result<Vec<u8>, AudioFailure>;
}

/// Real gTTS synthesis via `uv run --with gtts gtts-cli`.
struct GttsBackend {
    uv: PathBuf,
}

impl Backend for GttsBackend {
    fn synthesize(&self, request: &AudioRequest) -> Result<Vec<u8>, AudioFailure> {
        if request.text.trim().is_empty() {
            return Err(fail(AudioFailureClass::InvalidRequest, "target text is empty"));
        }
        let directory = tempfile::tempdir().map_err(|error| fail(AudioFailureClass::Retryable, error.to_string()))?;
        let target = directory.path().join("speech.mp3");
        let output = Command::new(&self.uv)
            .args(["run", "--with", "gtts", "gtts-cli"])
            .arg(&request.text)
            .arg("--lang")
            .arg(request.language.as_str())
            .arg("--output")
            .arg(&target)
            .output()
            .map_err(|error| {
                let class = if error.kind() == std::io::ErrorKind::NotFound {
                    AudioFailureClass::Configuration
                } else {
                    AudioFailureClass::Retryable
                };
                fail(class, format!("could not run gTTS via uv: {error}"))
            })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(fail(AudioFailureClass::Retryable, gtts_message(&stderr, output.status.code())));
        }
        let bytes = fs::read(&target).map_err(|error| fail(AudioFailureClass::Retryable, error.to_string()))?;
        if bytes.is_empty() {
            return Err(fail(AudioFailureClass::Retryable, "gTTS returned empty audio"));
        }
        Ok(bytes)
    }
}

/// Deterministic offline backend for tests/smoke flows (no network, no `uv`).
struct DeterministicBackend;

impl Backend for DeterministicBackend {
    fn synthesize(&self, request: &AudioRequest) -> Result<Vec<u8>, AudioFailure> {
        if request.text.trim().is_empty() {
            return Err(fail(AudioFailureClass::InvalidRequest, "target text is empty"));
        }
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"ID3");
        bytes.extend_from_slice(request.backend.wire_name().as_bytes());
        bytes.push(b'\n');
        bytes.extend_from_slice(request.text.as_bytes());
        Ok(bytes)
    }
}

#[derive(Default)]
pub struct AudioCatalogBuilder {
    backends: BTreeMap<AudioBackendId, Box<dyn Backend>>,
}

impl AudioCatalogBuilder {
    pub fn new() -> Self { Self::default() }

    /// Real gTTS, run through the given `uv` executable.
    pub fn add_gtts(self, uv: impl Into<PathBuf>) -> Result<Self, AudioAdapterError> {
        self.add(AudioBackendId::Gtts, Box::new(GttsBackend { uv: uv.into() }))
    }

    /// Deterministic offline backend registered under `id` (tests/smoke flows).
    pub fn add_deterministic(self, id: AudioBackendId) -> Result<Self, AudioAdapterError> {
        self.add(id, Box::new(DeterministicBackend))
    }

    fn add(mut self, id: AudioBackendId, backend: Box<dyn Backend>) -> Result<Self, AudioAdapterError> {
        if self.backends.insert(id, backend).is_some() {
            return Err(AudioAdapterError::DuplicateBackend(id));
        }
        Ok(self)
    }

    pub fn build(self) -> AudioCatalog { AudioCatalog { backends: self.backends } }
}

pub struct AudioCatalog {
    backends: BTreeMap<AudioBackendId, Box<dyn Backend>>,
}

impl AudioCatalog {
    pub fn builder() -> AudioCatalogBuilder { AudioCatalogBuilder::new() }

    /// The default production catalog: real gTTS via `uv` on `$PATH`.
    pub fn production() -> Self {
        Self::builder()
            .add_gtts(PathBuf::from("uv"))
            .map(AudioCatalogBuilder::build)
            .unwrap_or_else(|_| Self::builder().build())
    }

    /// Offline catalog for tests/smoke flows: deterministic gTTS + ElevenLabs.
    pub fn deterministic() -> Self {
        Self::builder()
            .add_deterministic(AudioBackendId::Gtts)
            .and_then(|builder| builder.add_deterministic(AudioBackendId::ElevenLabs))
            .map(AudioCatalogBuilder::build)
            .unwrap_or_else(|_| Self::builder().build())
    }
}

impl AudioSynthesizer for AudioCatalog {
    fn synthesize(&self, request: &AudioRequest) -> Result<SynthesizedAudio, AudioFailure> {
        let backend = self.backends.get(&request.backend).ok_or_else(|| {
            fail(AudioFailureClass::Configuration, format!("backend {} is not configured", request.backend))
        })?;
        let bytes = backend.synthesize(request)?;
        Ok(SynthesizedAudio {
            bytes,
            backend: request.backend,
            format: AudioFormat::Mp3,
            voice: request.voice.clone(),
            model: request.model.clone(),
        })
    }
}

fn fail(class: AudioFailureClass, message: impl Into<String>) -> AudioFailure {
    AudioFailure { class, message: message.into() }
}

fn gtts_message(stderr: &str, code: Option<i32>) -> String {
    let compact = stderr.lines().take(4).collect::<Vec<_>>().join(" ").chars().take(500).collect::<String>();
    if compact.is_empty() {
        format!("gTTS process exited with status {code:?}")
    } else {
        format!("gTTS process exited with status {code:?}: {compact}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lingo_application::ports::AudioSynthesizer;
    use lingo_domain::{LanguageCode, SentenceId};

    fn request() -> AudioRequest {
        AudioRequest {
            sentence: SentenceId::parse("sen-1").unwrap(),
            text: "नमस्ते".to_string(),
            language: LanguageCode::parse("hi").unwrap(),
            backend: AudioBackendId::Gtts,
            voice: None,
            model: None,
        }
    }

    #[test]
    fn duplicate_backends_are_rejected() {
        assert!(AudioCatalog::builder()
            .add_deterministic(AudioBackendId::Gtts)
            .unwrap()
            .add_deterministic(AudioBackendId::Gtts)
            .is_err());
    }

    #[test]
    fn deterministic_backend_returns_mp3_like_bytes() {
        let catalog = AudioCatalog::deterministic();
        let audio = catalog.synthesize(&request()).unwrap();
        assert!(audio.bytes.starts_with(b"ID3"));
    }

    #[test]
    fn unconfigured_backend_is_a_configuration_error() {
        let catalog = AudioCatalog::builder().build();
        let error = catalog.synthesize(&request()).unwrap_err();
        assert_eq!(error.class, AudioFailureClass::Configuration);
    }
}
