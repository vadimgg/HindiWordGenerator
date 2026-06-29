#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
//! Pure sentence-library domain model.
//!
//! The domain owns value objects and invariants. SQLite, filesystem layout,
//! prompts, audio providers, CLI rendering, and package formats are adapter
//! concerns.

mod audio;
mod diagnostic;
mod ids;
mod library;
mod sentence;
mod text;
mod word;

pub use audio::{
    AudioBackendId, AudioFailureClass, AudioFormat, AudioRelativePath, AudioValueError,
    ContentHash, SentenceAudio,
};
pub use diagnostic::{Diagnostic, DiagnosticCode, Severity, ValidationReport};
pub use ids::{BatchId, CollectionId, IdError, ProfileId, RunId, SentenceId, WordId, WordMeaningId};
pub use library::{Batch, Collection};
pub use sentence::{
    AuthorityError, BreakdownItem, DraftSentenceInput, FieldAuthority, FieldAuthoritySet,
    Register, Sentence, SentenceEnrichment, SentenceError, SentenceField, SentenceProvenance, SentenceRecordInput,
    SentenceStatus, TokenBreakdown,
};
pub use text::{
    BatchName, CollectionTitle, DisplayLead, DisplayPolicy, Gloss, LanguageCode, LanguageName,
    LanguageProfile, LiteralGloss, Romanisation, RomanisationConvention, ScriptName, SectionName,
    SentenceOrder, SentenceTags, TargetText, TextDirection, TextError, normalize_text,
};
pub use word::{WordEntry, WordKey, WordMeaning, WordOccurrence};
