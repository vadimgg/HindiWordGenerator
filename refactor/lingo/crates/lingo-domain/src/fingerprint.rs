use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use sha2::{Digest, Sha256};
use std::fmt;
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FingerprintError {
    #[error("invalid SHA-256 fingerprint: {0:?}")]
    Invalid(String),
}

fn is_sha256(raw: &str) -> bool {
    raw.strip_prefix("sha256:")
        .is_some_and(|hex| hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

macro_rules! hash_value {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn parse(raw: impl Into<String>) -> Result<Self, FingerprintError> {
                let raw = raw.into().to_ascii_lowercase();
                if !is_sha256(&raw) {
                    return Err(FingerprintError::Invalid(raw));
                }
                Ok(Self(raw))
            }

            fn from_digest(digest: impl AsRef<[u8]>) -> Self {
                Self(format!("sha256:{}", hex(digest.as_ref())))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn hex(&self) -> &str {
                self.0.strip_prefix("sha256:").unwrap_or(&self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let raw = String::deserialize(deserializer)?;
                Self::parse(raw).map_err(D::Error::custom)
            }
        }
    };
}

hash_value!(SourceFingerprint);
hash_value!(ContentHash);

pub struct SourceTextParts<'a> {
    pub target: &'a str,
    pub romanisation: Option<&'a str>,
    pub english: &'a str,
}

pub fn source_fingerprint(parts: &SourceTextParts<'_>) -> SourceFingerprint {
    let canonical = [
        normalize_for_fingerprint(parts.target),
        parts
            .romanisation
            .map(normalize_for_fingerprint)
            .unwrap_or_default(),
        normalize_for_fingerprint(parts.english),
    ]
    .join("\n");
    SourceFingerprint::from_digest(Sha256::digest(canonical.as_bytes()))
}

pub fn content_hash(bytes: &[u8]) -> ContentHash {
    ContentHash::from_digest(Sha256::digest(bytes))
}

pub fn normalize_for_fingerprint(raw: &str) -> String {
    raw.nfc()
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{SourceTextParts, content_hash, source_fingerprint};

    #[test]
    fn equivalent_whitespace_has_same_fingerprint() {
        let one = source_fingerprint(&SourceTextParts {
            target: "यह  किताब",
            romanisation: Some("yah kitāb"),
            english: "this book",
        });
        let two = source_fingerprint(&SourceTextParts {
            target: " यह किताब ",
            romanisation: Some("yah   kitāb"),
            english: "this   book",
        });
        assert_eq!(one, two);
    }

    #[test]
    fn content_hash_is_prefixed() {
        assert!(content_hash(b"hello").as_str().starts_with("sha256:"));
    }
}
