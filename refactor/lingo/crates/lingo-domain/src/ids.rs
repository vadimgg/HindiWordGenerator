use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum IdError {
    #[error("invalid {kind} identifier: {value:?}")]
    Invalid { kind: &'static str, value: String },
    #[error("invalid card identifier: {0:?}")]
    InvalidCard(String),
}

fn valid_segment(raw: &str, max_len: usize) -> bool {
    !raw.is_empty()
        && raw.len() <= max_len
        && raw.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

macro_rules! string_id {
    ($name:ident, $kind:literal, $max_len:expr) => {
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn parse(raw: impl Into<String>) -> Result<Self, IdError> {
                let raw = raw.into();
                if !valid_segment(&raw, $max_len) {
                    return Err(IdError::Invalid {
                        kind: $kind,
                        value: raw,
                    });
                }
                Ok(Self(raw))
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

string_id!(ProfileId, "profile", 48);
string_id!(BatchId, "batch", 80);
string_id!(SourceItemId, "source item", 48);
string_id!(WordId, "word", 32);
string_id!(RunId, "run", 96);
string_id!(ArtifactId, "artifact", 96);
string_id!(RawDocumentId, "raw document", 96);

impl SourceItemId {
    pub fn from_fingerprint_prefix(
        prefix: &str,
        duplicate_ordinal: usize,
    ) -> Result<Self, IdError> {
        let prefix = prefix
            .chars()
            .filter(|character| character.is_ascii_hexdigit())
            .take(16)
            .collect::<String>()
            .to_ascii_lowercase();
        Self::parse(format!("s-{prefix}-{duplicate_ordinal:02}"))
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct CardId {
    batch: BatchId,
    source_item: SourceItemId,
}

impl CardId {
    pub fn new(batch: BatchId, source_item: SourceItemId) -> Self {
        Self { batch, source_item }
    }

    pub fn parse(raw: &str) -> Result<Self, IdError> {
        let Some((batch, source_item)) = raw.split_once(':') else {
            return Err(IdError::InvalidCard(raw.to_string()));
        };
        if source_item.contains(':') {
            return Err(IdError::InvalidCard(raw.to_string()));
        }
        Ok(Self::new(
            BatchId::parse(batch).map_err(|_| IdError::InvalidCard(raw.to_string()))?,
            SourceItemId::parse(source_item).map_err(|_| IdError::InvalidCard(raw.to_string()))?,
        ))
    }

    pub fn batch(&self) -> &BatchId {
        &self.batch
    }

    pub fn source_item(&self) -> &SourceItemId {
        &self.source_item
    }
}

impl fmt::Display for CardId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.batch, self.source_item)
    }
}

impl Serialize for CardId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for CardId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::{BatchId, CardId, ProfileId, SourceItemId};

    #[test]
    fn validates_segments() {
        assert!(BatchId::parse("chapter-01").is_ok());
        assert!(BatchId::parse("Chapter 01").is_err());
        assert!(ProfileId::parse("hindi").is_ok());
    }

    #[test]
    fn card_id_round_trips() {
        let id = CardId::new(
            BatchId::parse("chapter-01").unwrap(),
            SourceItemId::parse("s-1234567890abcdef-01").unwrap(),
        );
        assert_eq!(CardId::parse(&id.to_string()).unwrap(), id);
    }
}
