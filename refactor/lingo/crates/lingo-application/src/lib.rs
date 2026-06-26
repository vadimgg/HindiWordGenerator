#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
//! Use cases and ports for the lingo workflow.
//!
//! This crate owns policy and typed reports. Concrete filesystem, prompt,
//! provider, artifact, and presentation mechanics live in outward adapters.

pub mod ports;

mod audio;
mod build;
mod check;
mod doctor;
mod export;
mod import;
mod init;
mod lang;
mod package;
mod report;
mod status;
mod viewer;

pub use audio::{
    AudioBatchResult, AudioCardResult, AudioCommand, AudioDeps, AudioError, AudioMode, AudioReport,
    synthesize_audio,
};
pub use build::{
    ApplyBuild, BuildDeps, BuildError, BuildPreparation, BuildReport, PrepareBuild, apply_build,
    prepare_build,
};
pub use check::{CheckDeps, CheckError, CheckReport, CheckRequest, CheckedBatch, check};
pub use doctor::{DoctorDeps, DoctorError, DoctorReport, doctor};
pub use export::{ExportDeps, ExportError, ExportReport, ExportRequest, export};
pub use import::{
    ApplyImport, ImportDeps, ImportError, ImportPreparation, ImportReport, PrepareImport,
    apply_import, prepare_import,
};
pub use init::{InitError, InitReport, InitRequest, init};
pub use lang::{
    EditPromptReport, EditPromptRequest, LangError, LanguageListReport, LanguageShowReport,
    PromptOriginReport, create_prompt_override, list_languages, show_language, which_prompts,
};
pub use package::{PackageDeps, PackageError, PackageReport, PackageRequest, package};
pub use report::{ChangeCounts, CommandHint, NextAction};
pub use status::{
    PendingRaw, StatusDeps, StatusError, StatusReport, StatusRequest, StatusRow, status,
};
pub use viewer::{ViewerCard, ViewerDeps, ViewerError, ViewerPlan, ViewerRequest, prepare_viewer};
