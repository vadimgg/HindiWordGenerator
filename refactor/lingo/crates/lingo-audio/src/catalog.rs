use crate::backend::AudioBackend;
use crate::elevenlabs::ElevenLabsBackend;
use crate::error::AudioAdapterError;
use crate::fallback::synthesize_with_fallback;
use crate::gtts::GttsBackend;
use crate::model::BackendRequest;
use lingo_application::ports::{AudioFailure, AudioRequest, AudioSynthesizer, SynthesizedAudio};
use lingo_domain::AudioBackendId;
use reqwest::blocking::Client;
use secrecy::SecretString;
use std::path::PathBuf;

pub struct AudioCatalogBuilder {
    backends: Vec<Box<dyn AudioBackend>>,
}

impl AudioCatalogBuilder {
    pub fn new() -> Self {
        Self {
            backends: Vec::new(),
        }
    }

    pub fn add_gtts(mut self, uv: PathBuf) -> Result<Self, AudioAdapterError> {
        self.add(Box::new(GttsBackend::new(uv)))?;
        Ok(self)
    }

    pub fn add_elevenlabs(
        mut self,
        client: Client,
        key: SecretString,
    ) -> Result<Self, AudioAdapterError> {
        self.add(Box::new(ElevenLabsBackend::new(client, key)))?;
        Ok(self)
    }

    pub fn build(self) -> AudioCatalog {
        AudioCatalog {
            backends: self.backends,
        }
    }

    fn add(&mut self, backend: Box<dyn AudioBackend>) -> Result<(), AudioAdapterError> {
        if self.backends.iter().any(|known| known.id() == backend.id()) {
            return Err(AudioAdapterError::DuplicateBackend(backend.id()));
        }
        self.backends.push(backend);
        Ok(())
    }
}

impl Default for AudioCatalogBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AudioCatalog {
    backends: Vec<Box<dyn AudioBackend>>,
}

impl AudioCatalog {
    pub fn builder() -> AudioCatalogBuilder {
        AudioCatalogBuilder::new()
    }

    fn require(&self, id: AudioBackendId) -> Result<&dyn AudioBackend, AudioAdapterError> {
        self.backends
            .iter()
            .find(|backend| backend.id() == id)
            .map(Box::as_ref)
            .ok_or(AudioAdapterError::BackendUnavailable(id))
    }
}

impl AudioSynthesizer for AudioCatalog {
    fn synthesize(&self, request: &AudioRequest) -> Result<SynthesizedAudio, AudioFailure> {
        let primary = self.require(request.primary)?;
        let fallback = request.fallback.map(|id| self.require(id)).transpose()?;
        let primary_request = BackendRequest::for_backend(request, request.primary);
        let result = match fallback {
            Some(fallback) if fallback.id() != request.primary => {
                match primary.synthesize(&primary_request) {
                    Ok(audio) => Ok((primary.id(), audio)),
                    Err(error) if error.class() == lingo_domain::AudioFailureClass::Retryable => {
                        let fallback_request = BackendRequest::for_backend(request, fallback.id());
                        synthesize_with_fallback(fallback, None, &fallback_request)
                    }
                    Err(error) => Err(error),
                }
            }
            _ => synthesize_with_fallback(primary, None, &primary_request),
        }?;
        Ok(SynthesizedAudio {
            bytes: result.1.bytes,
            backend: result.0,
            format: result.1.format,
        })
    }
}
