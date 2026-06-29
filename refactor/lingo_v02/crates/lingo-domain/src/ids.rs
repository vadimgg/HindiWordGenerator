use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum IdError {
    #[error("invalid {kind} identifier: {value:?}")]
    Invalid { kind: &'static str, value: String },
}

fn valid_id(raw: &str, max_len: usize) -> bool {
    !raw.is_empty()
        && raw.len() <= max_len
        && raw.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'-' | b'_')
        })
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn generated(prefix: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    let counter = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{millis:x}-{counter:x}")
}

macro_rules! id_type {
    ($name:ident, $kind:literal, $prefix:literal, $max:expr) => {
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn parse(raw: impl Into<String>) -> Result<Self, IdError> {
                let raw = raw.into();
                if !valid_id(&raw, $max) {
                    return Err(IdError::Invalid { kind: $kind, value: raw });
                }
                Ok(Self(raw))
            }

            pub fn generate() -> Self {
                Self(generated($prefix))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(self.as_str())
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

id_type!(CollectionId, "collection", "col", 96);
id_type!(SentenceId, "sentence", "sen", 96);
id_type!(WordId, "word", "word", 96);
id_type!(WordMeaningId, "word meaning", "mean", 96);
id_type!(RunId, "run", "run", 128);
id_type!(BatchId, "batch", "bat", 96);
id_type!(ProfileId, "profile", "profile", 64);

#[cfg(test)]
mod tests {
    use super::{CollectionId, SentenceId};

    #[test]
    fn parses_stable_identifiers() {
        assert!(CollectionId::parse("complete-hindi").is_ok());
        assert!(CollectionId::parse("Complete Hindi").is_err());
    }

    #[test]
    fn generated_ids_are_valid_and_unique() {
        let one = SentenceId::generate();
        let two = SentenceId::generate();
        assert_ne!(one, two);
        assert!(one.as_str().starts_with("sen-"));
    }
}
