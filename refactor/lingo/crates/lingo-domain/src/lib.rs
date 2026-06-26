#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
//! Language-neutral learning-card domain.
//!
//! Canonical aggregates are valid by construction. Boundary crates parse
//! untrusted YAML/JSON into DTOs and then call these constructors.

mod audio;
mod card;
mod diagnostic;
mod fingerprint;
mod ids;
mod language;
mod pipeline;
mod source;
mod validation;

pub use audio::{
    AudioBackendId, AudioFailureClass, AudioFormat, AudioRef, AudioRelativePath, AudioValueError,
};
pub use card::{
    Card, CardBatch, CardError, CardFormat, CardTags, CardToken, GrammarTag, Register, SourceRef,
    Word, WordKind,
};
pub use diagnostic::{Diagnostic, DiagnosticCode, DiagnosticLocation, Severity, ValidationReport};
pub use fingerprint::{
    ContentHash, FingerprintError, SourceFingerprint, SourceTextParts, content_hash,
    normalize_for_fingerprint, source_fingerprint,
};
pub use ids::{
    ArtifactId, BatchId, CardId, IdError, ProfileId, RawDocumentId, RunId, SourceItemId, WordId,
};
pub use language::{
    DisplayLead, DisplayPolicy, Gloss, LanguageCode, LanguageError, LanguageName, LanguageProfile,
    LearnerNotes, Romanisation, RomanisationConvention, ScriptName, SourceSubtitle, SourceTitle,
    TargetText, TextDirection,
};
pub use pipeline::{AudioCoverage, BatchProgress, CheckState, PipelineStage};
pub use source::{SourceBatch, SourceError, SourceFormat, SourceItem, SourceTags};
pub use validation::check_card_batch;
