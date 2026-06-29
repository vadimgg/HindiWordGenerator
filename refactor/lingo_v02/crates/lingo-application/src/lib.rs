#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
//! Workflow policy and ports for Lingo's SQLite sentence library.

pub mod ports;
mod audio;
mod enrich;
mod extract;
mod import;
mod library;
mod package;
mod report;
mod status;

pub use audio::{AudioCommand, AudioDeps, AudioError, AudioMode, AudioReport, AudioSentenceResult, synthesize_audio};
pub use import::{ImportDeps, ImportError, ImportPackage, ImportReport, ImportedSentence, import_package};
pub use enrich::{ApplyEnrich, EnrichDeps, EnrichError, EnrichPreparation, EnrichReport, PrepareEnrich, ResetEnrich, ResetEnrichReport, apply_enrich, prepare_enrich, reset_enrichment_claim};
pub use extract::{ApplyExtract, BatchRef, ExtractDeps, ExtractError, ExtractPreparation, ExtractReport, PrepareExtract, apply_extract, prepare_extract};
pub use library::{LibraryError, default_query, get_sentence, list_library, list_words, reorder_sentences, update_sentence};
pub use package::{ExportDeps, ExportError, ExportReport, ExportRequest, PackageDeps, PackageError, PackageFormat, PackageReport, PackageRequest, export_anki, package};
pub use report::{CommandHint, NextAction};
pub use status::{StatusDeps, StatusError, StatusReport, status};
