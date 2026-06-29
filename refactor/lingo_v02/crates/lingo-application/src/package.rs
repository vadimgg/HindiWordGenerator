use crate::ports::{
    AnkiExporter, ArtifactFailure, DeckContextProvider, LibraryFailure, LibraryStore,
    PackagePublisher, PageRequest, PublishMaterial, PublishedArtifact, SentenceQuery,
    SentenceSelection, SentenceSort,
};
use crate::report::NextAction;
use std::path::PathBuf;
use thiserror::Error;

pub struct PackageDeps<'a> {
    pub library: &'a dyn LibraryStore,
    pub context: &'a dyn DeckContextProvider,
    pub publisher: &'a dyn PackagePublisher,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackageFormat { Json, Db }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageRequest {
    pub selection: SentenceSelection,
    pub destination: Option<PathBuf>,
    pub format: PackageFormat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageReport {
    pub artifact: PublishedArtifact,
    pub sentences: usize,
    pub next: NextAction,
}

#[derive(Debug, Error)]
pub enum PackageError {
    #[error(transparent)] Library(#[from] LibraryFailure),
    #[error(transparent)] Context(#[from] crate::ports::ContextFailure),
    #[error(transparent)] Artifact(#[from] ArtifactFailure),
    #[error("no sentences were selected")]
    EmptySelection,
}

pub fn package(deps: &PackageDeps<'_>, request: PackageRequest) -> Result<PackageReport, PackageError> {
    let context = deps.context.resolve()?;
    let page = deps.library.list_sentences(&SentenceQuery {
        selection: request.selection,
        sort: SentenceSort::LibraryOrder,
        page: PageRequest { limit: 100_000, offset: 0 },
    })?;
    if page.sentences.is_empty() { return Err(PackageError::EmptySelection); }
    let destination = request.destination.unwrap_or(context.package_destination.clone());
    let material = PublishMaterial { profile: context.profile, display: context.display, sentences: page.sentences, source_root: context.root.clone() };
    let artifact = match request.format {
        PackageFormat::Json => deps.publisher.publish_json(&destination, &material)?,
        PackageFormat::Db => deps.publisher.publish_db_copy(&destination, &material)?,
    };
    Ok(PackageReport { artifact, sentences: material.sentences.len(), next: NextAction::Export })
}

pub struct ExportDeps<'a> {
    pub library: &'a dyn LibraryStore,
    pub context: &'a dyn DeckContextProvider,
    pub exporter: &'a dyn AnkiExporter,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportRequest {
    pub selection: SentenceSelection,
    pub deck: String,
    pub destination: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportReport {
    pub artifact: PublishedArtifact,
    pub sentences: usize,
    pub next: NextAction,
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error(transparent)] Library(#[from] LibraryFailure),
    #[error(transparent)] Context(#[from] crate::ports::ContextFailure),
    #[error(transparent)] Artifact(#[from] ArtifactFailure),
    #[error("no sentences were selected")]
    EmptySelection,
}

pub fn export_anki(deps: &ExportDeps<'_>, request: ExportRequest) -> Result<ExportReport, ExportError> {
    let context = deps.context.resolve()?;
    let page = deps.library.list_sentences(&SentenceQuery {
        selection: request.selection,
        sort: SentenceSort::LibraryOrder,
        page: PageRequest { limit: 100_000, offset: 0 },
    })?;
    if page.sentences.is_empty() { return Err(ExportError::EmptySelection); }
    let material = PublishMaterial { profile: context.profile, display: context.display, sentences: page.sentences, source_root: context.root.clone() };
    let artifact = deps.exporter.export_apkg(&request.destination, &request.deck, &material)?;
    Ok(ExportReport { artifact, sentences: material.sentences.len(), next: NextAction::None })
}
