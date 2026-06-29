use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use sha2::{Digest, Sha256};
use std::fmt;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
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
    pub const fn wire_name(self) -> &'static str {
        match self { Self::Mp3 => "mp3" }
    }
    pub const fn extension(self) -> &'static str { self.wire_name() }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioFailureClass {
    Retryable,
    Configuration,
    InvalidRequest,
}

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum AudioValueError {
    #[error("unknown audio backend: {0:?}")]
    UnknownBackend(String),
    #[error("audio path is unsafe: {0:?}")]
    UnsafePath(String),
    #[error("invalid content hash: {0:?}")]
    InvalidHash(String),
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
        if !valid { return Err(AudioValueError::UnsafePath(raw)); }
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str { &self.0 }
}

impl fmt::Display for AudioRelativePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for AudioRelativePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(raw).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Serialize)]
#[serde(transparent)]
pub struct ContentHash(String);

impl ContentHash {
    pub fn parse(raw: impl Into<String>) -> Result<Self, AudioValueError> {
        let raw = raw.into().to_ascii_lowercase();
        let valid = raw
            .strip_prefix("sha256:")
            .is_some_and(|hex| hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()));
        if !valid { return Err(AudioValueError::InvalidHash(raw)); }
        Ok(Self(raw))
    }

    pub fn sha256(bytes: &[u8]) -> Self {
        let digest = Sha256::digest(bytes);
        let mut out = String::from("sha256:");
        for byte in digest {
            use std::fmt::Write as _;
            let _ = write!(&mut out, "{byte:02x}");
        }
        Self(out)
    }

    pub fn as_str(&self) -> &str { &self.0 }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ContentHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(raw).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SentenceAudio {
    path: AudioRelativePath,
    hash: ContentHash,
    backend: AudioBackendId,
    format: AudioFormat,
    voice: Option<String>,
    model: Option<String>,
}

impl SentenceAudio {
    pub fn new(
        path: AudioRelativePath,
        hash: ContentHash,
        backend: AudioBackendId,
        format: AudioFormat,
        voice: Option<String>,
        model: Option<String>,
    ) -> Self {
        Self { path, hash, backend, format, voice, model }
    }

    pub fn path(&self) -> &AudioRelativePath { &self.path }
    pub fn hash(&self) -> &ContentHash { &self.hash }
    pub const fn backend(&self) -> AudioBackendId { self.backend }
    pub const fn format(&self) -> AudioFormat { self.format }
    pub fn voice(&self) -> Option<&str> { self.voice.as_deref() }
    pub fn model(&self) -> Option<&str> { self.model.as_deref() }
}

#[cfg(test)]
mod tests {
    use super::{AudioRelativePath, ContentHash};

    #[test]
    fn rejects_path_traversal() {
        assert!(AudioRelativePath::parse("audio/sen-1.mp3").is_ok());
        assert!(AudioRelativePath::parse("../secret.mp3").is_err());
        assert!(AudioRelativePath::parse("/tmp/a.mp3").is_err());
    }

    #[test]
    fn hashes_are_stable() {
        assert_eq!(ContentHash::sha256(b"abc").as_str(), "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }
}
