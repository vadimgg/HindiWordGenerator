use crate::{CardId, ContentHash};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use std::fmt;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AudioBackendId {
    #[serde(rename = "gtts")]
    Gtts,
    #[serde(rename = "elevenlabs")]
    ElevenLabs,
}

impl AudioBackendId {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Gtts => "gtts",
            Self::ElevenLabs => "elevenlabs",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, AudioValueError> {
        match raw {
            "gtts" => Ok(Self::Gtts),
            "elevenlabs" => Ok(Self::ElevenLabs),
            other => Err(AudioValueError::UnknownBackend(other.to_string())),
        }
    }
}

impl fmt::Display for AudioBackendId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.wire_name())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    Mp3,
}

impl AudioFormat {
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Mp3 => "mp3",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioFailureClass {
    Retryable,
    Configuration,
    InvalidRequest,
}

impl AudioFailureClass {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Retryable => "retryable",
            Self::Configuration => "configuration",
            Self::InvalidRequest => "invalid_request",
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AudioValueError {
    #[error("unknown audio backend: {0:?}")]
    UnknownBackend(String),
    #[error("audio relative path is unsafe: {0:?}")]
    UnsafePath(String),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Serialize)]
#[serde(transparent)]
pub struct AudioRelativePath(String);

impl AudioRelativePath {
    pub fn parse(raw: impl Into<String>) -> Result<Self, AudioValueError> {
        let raw = raw.into().replace('\\', "/");
        let valid = !raw.is_empty()
            && !raw.starts_with('/')
            && !raw.contains(':')
            && raw.split('/').all(|part| {
                !part.is_empty() && part != "." && part != ".." && !part.contains('\0')
            });
        if !valid {
            return Err(AudioValueError::UnsafePath(raw));
        }
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AudioRelativePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for AudioRelativePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::parse(raw).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AudioRef {
    card_id: CardId,
    relative_path: AudioRelativePath,
    content_hash: ContentHash,
    backend: AudioBackendId,
    format: AudioFormat,
}

impl AudioRef {
    pub fn new(
        card_id: CardId,
        relative_path: AudioRelativePath,
        content_hash: ContentHash,
        backend: AudioBackendId,
        format: AudioFormat,
    ) -> Self {
        Self {
            card_id,
            relative_path,
            content_hash,
            backend,
            format,
        }
    }

    pub fn card_id(&self) -> &CardId {
        &self.card_id
    }

    pub fn relative_path(&self) -> &AudioRelativePath {
        &self.relative_path
    }

    pub fn content_hash(&self) -> &ContentHash {
        &self.content_hash
    }

    pub const fn backend(&self) -> AudioBackendId {
        self.backend
    }

    pub const fn format(&self) -> AudioFormat {
        self.format
    }
}

#[cfg(test)]
mod tests {
    use super::AudioRelativePath;

    #[test]
    fn rejects_escaping_paths() {
        assert!(AudioRelativePath::parse("audio/sentences/a.mp3").is_ok());
        assert!(AudioRelativePath::parse("../a.mp3").is_err());
        assert!(AudioRelativePath::parse("/tmp/a.mp3").is_err());
    }
}
