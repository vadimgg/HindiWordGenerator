use lingo_application::ports::AudioFailure;
use lingo_domain::{AudioBackendId, AudioFailureClass};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioAdapterError {
    #[error("audio catalog contains duplicate backend {0}")]
    DuplicateBackend(AudioBackendId),
    #[error("audio backend {0} is not configured")]
    BackendUnavailable(AudioBackendId),
    #[error("{backend} synthesis failed ({class:?}): {message}")]
    Backend {
        backend: AudioBackendId,
        class: AudioFailureClass,
        message: String,
    },
}

impl AudioAdapterError {
    pub(crate) fn backend(
        backend: AudioBackendId,
        class: AudioFailureClass,
        message: impl Into<String>,
    ) -> Self {
        Self::Backend {
            backend,
            class,
            message: message.into(),
        }
    }

    pub(crate) const fn class(&self) -> AudioFailureClass {
        match self {
            Self::DuplicateBackend(_) | Self::BackendUnavailable(_) => {
                AudioFailureClass::Configuration
            }
            Self::Backend { class, .. } => *class,
        }
    }
}

impl From<AudioAdapterError> for AudioFailure {
    fn from(error: AudioAdapterError) -> Self {
        Self {
            class: error.class(),
            message: error.to_string(),
        }
    }
}
