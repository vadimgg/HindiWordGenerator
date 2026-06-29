use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use std::collections::BTreeSet;
use std::fmt;
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum TextError {
    #[error("{kind} cannot be empty")]
    Empty { kind: &'static str },
    #[error("{kind} is too long")]
    TooLong { kind: &'static str },
    #[error("invalid language code: {0:?}")]
    InvalidLanguageCode(String),
    #[error("invalid tag: {0:?}")]
    InvalidTag(String),
    #[error("sentence order must be positive")]
    InvalidOrder,
}

pub fn normalize_text(raw: impl AsRef<str>) -> String {
    raw.as_ref()
        .nfc()
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

macro_rules! text_type {
    ($name:ident, $kind:literal, $max:expr) => {
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn parse(raw: impl Into<String>) -> Result<Self, TextError> {
                let value = normalize_text(raw.into());
                if value.is_empty() {
                    return Err(TextError::Empty { kind: $kind });
                }
                if value.chars().count() > $max {
                    return Err(TextError::TooLong { kind: $kind });
                }
                Ok(Self(value))
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

text_type!(TargetText, "target text", 2_000);
text_type!(Romanisation, "romanisation", 2_000);
text_type!(Gloss, "gloss", 4_000);
text_type!(LiteralGloss, "literal gloss", 4_000);
text_type!(CollectionTitle, "collection title", 240);
text_type!(SectionName, "section name", 240);
text_type!(BatchName, "batch name", 240);
text_type!(LanguageName, "language name", 120);
text_type!(ScriptName, "script name", 120);

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Serialize)]
#[serde(transparent)]
pub struct LanguageCode(String);

impl LanguageCode {
    pub fn parse(raw: impl Into<String>) -> Result<Self, TextError> {
        let raw = raw.into().to_ascii_lowercase();
        let valid = (2..=16).contains(&raw.len())
            && raw
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        if !valid {
            return Err(TextError::InvalidLanguageCode(raw));
        }
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LanguageCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for LanguageCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::parse(raw).map_err(D::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextDirection {
    Ltr,
    Rtl,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RomanisationConvention {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "iast-tilde")]
    IastTilde,
    #[serde(rename = "hepburn")]
    Hepburn,
}

impl RomanisationConvention {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::IastTilde => "iast-tilde",
            Self::Hepburn => "hepburn",
        }
    }

    pub const fn is_required(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LanguageProfile {
    id: crate::ProfileId,
    language: LanguageName,
    code: LanguageCode,
    script: ScriptName,
    direction: TextDirection,
    romanisation: RomanisationConvention,
}

impl LanguageProfile {
    pub fn new(
        id: crate::ProfileId,
        language: LanguageName,
        code: LanguageCode,
        script: ScriptName,
        direction: TextDirection,
        romanisation: RomanisationConvention,
    ) -> Self {
        Self { id, language, code, script, direction, romanisation }
    }

    pub fn id(&self) -> &crate::ProfileId { &self.id }
    pub fn language(&self) -> &LanguageName { &self.language }
    pub fn code(&self) -> &LanguageCode { &self.code }
    pub fn script(&self) -> &ScriptName { &self.script }
    pub const fn direction(&self) -> TextDirection { self.direction }
    pub const fn romanisation(&self) -> RomanisationConvention { self.romanisation }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DisplayLead {
    Romanisation,
    Target,
}

impl DisplayLead {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Romanisation => "romanisation",
            Self::Target => "target",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DisplayPolicy {
    lead: DisplayLead,
    show_secondary: bool,
}

impl DisplayPolicy {
    pub const fn new(lead: DisplayLead, show_secondary: bool) -> Self {
        Self { lead, show_secondary }
    }
    pub const fn lead(self) -> DisplayLead { self.lead }
    pub const fn show_secondary(self) -> bool { self.show_secondary }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SentenceOrder(i64);

impl SentenceOrder {
    pub fn new(value: i64) -> Result<Self, TextError> {
        if value <= 0 { return Err(TextError::InvalidOrder); }
        Ok(Self(value))
    }
    pub const fn get(self) -> i64 { self.0 }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SentenceTags(Vec<String>);

impl SentenceTags {
    pub fn try_from_values(values: impl IntoIterator<Item = String>) -> Result<Self, TextError> {
        let mut tags = BTreeSet::new();
        for value in values {
            let tag = value.trim().to_ascii_lowercase();
            let valid = !tag.is_empty()
                && tag.len() <= 80
                && tag.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'-' | b'_')
                });
            if !valid { return Err(TextError::InvalidTag(value)); }
            tags.insert(tag);
        }
        Ok(Self(tags.into_iter().collect()))
    }

    pub fn values(&self) -> &[String] { &self.0 }
    pub fn into_values(self) -> Vec<String> { self.0 }
}

#[cfg(test)]
mod tests {
    use super::{SentenceOrder, SentenceTags, TargetText};

    #[test]
    fn normalizes_text() {
        let text = TargetText::parse("  नमस्ते   जी  ").unwrap();
        assert_eq!(text.as_str(), "नमस्ते जी");
    }

    #[test]
    fn validates_order_and_tags() {
        assert!(SentenceOrder::new(1).is_ok());
        assert!(SentenceOrder::new(0).is_err());
        let tags = SentenceTags::try_from_values(["Classroom".to_string(), "classroom".to_string()]).unwrap();
        assert_eq!(tags.values(), &["classroom".to_string()]);
    }
}
