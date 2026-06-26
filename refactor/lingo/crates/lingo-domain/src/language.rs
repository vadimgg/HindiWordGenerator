use crate::ProfileId;
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use std::fmt;
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum LanguageError {
    #[error("{kind} cannot be empty")]
    EmptyText { kind: &'static str },
    #[error("{kind} is too long")]
    TextTooLong { kind: &'static str },
    #[error("invalid language code: {0:?}")]
    InvalidLanguageCode(String),
    #[error("unknown romanisation convention: {0:?}")]
    UnknownRomanisation(String),
}

fn normalize_text(raw: String) -> String {
    raw.nfc()
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

macro_rules! text_value {
    ($name:ident, $kind:literal, $max_len:expr) => {
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn parse(raw: impl Into<String>) -> Result<Self, LanguageError> {
                let value = normalize_text(raw.into());
                if value.is_empty() {
                    return Err(LanguageError::EmptyText { kind: $kind });
                }
                if value.chars().count() > $max_len {
                    return Err(LanguageError::TextTooLong { kind: $kind });
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

text_value!(TargetText, "target text", 2_000);
text_value!(Romanisation, "romanisation", 2_000);
text_value!(Gloss, "gloss", 4_000);
text_value!(SourceTitle, "source title", 240);
text_value!(SourceSubtitle, "source subtitle", 240);
text_value!(LanguageName, "language name", 120);
text_value!(ScriptName, "script name", 120);
text_value!(LearnerNotes, "learner notes", 4_000);

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Serialize)]
#[serde(transparent)]
pub struct LanguageCode(String);

impl LanguageCode {
    pub fn parse(raw: impl Into<String>) -> Result<Self, LanguageError> {
        let raw = raw.into().to_ascii_lowercase();
        let valid = (2..=16).contains(&raw.len())
            && raw
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        if !valid {
            return Err(LanguageError::InvalidLanguageCode(raw));
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

impl TextDirection {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }
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

    pub fn parse(raw: &str) -> Result<Self, LanguageError> {
        match raw {
            "none" => Ok(Self::None),
            "iast-tilde" => Ok(Self::IastTilde),
            "hepburn" => Ok(Self::Hepburn),
            other => Err(LanguageError::UnknownRomanisation(other.to_string())),
        }
    }

    pub const fn is_required(self) -> bool {
        !matches!(self, Self::None)
    }
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
        Self {
            lead,
            show_secondary,
        }
    }

    pub const fn lead(self) -> DisplayLead {
        self.lead
    }

    pub const fn show_secondary(self) -> bool {
        self.show_secondary
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LanguageProfile {
    id: ProfileId,
    language: LanguageName,
    code: LanguageCode,
    script: ScriptName,
    direction: TextDirection,
    romanisation: RomanisationConvention,
}

impl LanguageProfile {
    pub fn new(
        id: ProfileId,
        language: LanguageName,
        code: LanguageCode,
        script: ScriptName,
        direction: TextDirection,
        romanisation: RomanisationConvention,
    ) -> Self {
        Self {
            id,
            language,
            code,
            script,
            direction,
            romanisation,
        }
    }

    pub fn id(&self) -> &ProfileId {
        &self.id
    }

    pub fn language(&self) -> &LanguageName {
        &self.language
    }

    pub fn code(&self) -> &LanguageCode {
        &self.code
    }

    pub fn script(&self) -> &ScriptName {
        &self.script
    }

    pub const fn direction(&self) -> TextDirection {
        self.direction
    }

    pub const fn romanisation(&self) -> RomanisationConvention {
        self.romanisation
    }
}

#[cfg(test)]
mod tests {
    use super::{DisplayLead, DisplayPolicy, RomanisationConvention, TargetText};

    #[test]
    fn normalizes_text_values() {
        let text = TargetText::parse("  यह   किताब  ").unwrap();
        assert_eq!(text.as_str(), "यह किताब");
    }

    #[test]
    fn closed_set_names_are_stable() {
        assert_eq!(RomanisationConvention::IastTilde.wire_name(), "iast-tilde");
        assert_eq!(
            DisplayPolicy::new(DisplayLead::Target, true).lead(),
            DisplayLead::Target
        );
    }
}
