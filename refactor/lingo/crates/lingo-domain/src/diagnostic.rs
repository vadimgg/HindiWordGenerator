use crate::{BatchId, CardId, SourceItemId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    Warning,
    Error,
}

impl Severity {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticCode {
    BatchMismatch,
    MissingSourceItem,
    DuplicateSourceItem,
    FingerprintDrift,
    SourceContentDrift,
    MissingRomanisation,
    ForbiddenRomanisation,
    TokenMissingWord,
    UnusedWord,
    TokenTextMismatch,
    RomanisationReconstructionMismatch,
    MissingAudio,
    InvalidAudioReference,
}

impl DiagnosticCode {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::BatchMismatch => "batch_mismatch",
            Self::MissingSourceItem => "missing_source_item",
            Self::DuplicateSourceItem => "duplicate_source_item",
            Self::FingerprintDrift => "fingerprint_drift",
            Self::SourceContentDrift => "source_content_drift",
            Self::MissingRomanisation => "missing_romanisation",
            Self::ForbiddenRomanisation => "forbidden_romanisation",
            Self::TokenMissingWord => "token_missing_word",
            Self::UnusedWord => "unused_word",
            Self::TokenTextMismatch => "token_text_mismatch",
            Self::RomanisationReconstructionMismatch => "romanisation_reconstruction_mismatch",
            Self::MissingAudio => "missing_audio",
            Self::InvalidAudioReference => "invalid_audio_reference",
        }
    }

    pub const fn default_severity(self) -> Severity {
        match self {
            Self::MissingAudio => Severity::Warning,
            _ => Severity::Error,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiagnosticLocation {
    batch: Option<BatchId>,
    card: Option<CardId>,
    source_item: Option<SourceItemId>,
    field: Option<String>,
    index: Option<usize>,
}

impl DiagnosticLocation {
    pub fn batch(batch: BatchId) -> Self {
        Self {
            batch: Some(batch),
            ..Self::default()
        }
    }

    pub fn with_card(mut self, card: CardId) -> Self {
        self.card = Some(card);
        self
    }

    pub fn with_source_item(mut self, source_item: SourceItemId) -> Self {
        self.source_item = Some(source_item);
        self
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    pub fn with_index(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }

    pub fn batch_id(&self) -> Option<&BatchId> {
        self.batch.as_ref()
    }

    pub fn card_id(&self) -> Option<&CardId> {
        self.card.as_ref()
    }

    pub fn source_item_id(&self) -> Option<&SourceItemId> {
        self.source_item.as_ref()
    }

    pub fn field(&self) -> Option<&str> {
        self.field.as_deref()
    }

    pub const fn index(&self) -> Option<usize> {
        self.index
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    code: DiagnosticCode,
    severity: Severity,
    location: DiagnosticLocation,
    message: String,
}

impl Diagnostic {
    pub fn new(
        code: DiagnosticCode,
        location: DiagnosticLocation,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity: code.default_severity(),
            location,
            message: message.into(),
        }
    }

    pub const fn code(&self) -> DiagnosticCode {
        self.code
    }

    pub const fn severity(&self) -> Severity {
        self.severity
    }

    pub fn location(&self) -> &DiagnosticLocation {
        &self.location
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ValidationReport {
    diagnostics: Vec<Diagnostic>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn extend(&mut self, diagnostics: impl IntoIterator<Item = Diagnostic>) {
        self.diagnostics.extend(diagnostics);
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn is_clean(&self) -> bool {
        self.diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity() != Severity::Error)
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity() == Severity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics.len().saturating_sub(self.error_count())
    }
}
