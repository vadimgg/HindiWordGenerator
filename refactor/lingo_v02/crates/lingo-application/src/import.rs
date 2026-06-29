use crate::ports::{
    AudioFileFailure, AudioFileStore, ContextFailure, DeckContextProvider, ImportSentenceInput,
    ImportSentences, LibraryFailure, LibraryStore, SynthesizedAudio,
};
use crate::report::NextAction;
use lingo_domain::{
    AudioFormat, FieldAuthoritySet, Gloss, LiteralGloss, Register, Romanisation, SectionName,
    SentenceProvenance, SentenceStatus, SentenceTags, TargetText, TokenBreakdown,
};
use thiserror::Error;

pub struct ImportDeps<'a> {
    pub library: &'a dyn LibraryStore,
    pub context: &'a dyn DeckContextProvider,
    pub audio_files: &'a dyn AudioFileStore,
}

/// One sentence read from a package, ready to merge into the library.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedSentence {
    pub section: Option<SectionName>,
    pub target: TargetText,
    pub romanisation: Option<Romanisation>,
    pub english: Option<Gloss>,
    pub literal: Option<LiteralGloss>,
    pub register: Option<Register>,
    pub authority: FieldAuthoritySet,
    pub tags: SentenceTags,
    pub breakdown: Option<TokenBreakdown>,
    pub status: SentenceStatus,
    /// Original id in the source package (kept as provenance, not identity).
    pub source_item: Option<String>,
    /// Raw audio bytes, if the package shipped a clip for this sentence.
    pub audio: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportPackage {
    pub package_name: String,
    pub sentences: Vec<ImportedSentence>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportReport {
    pub imported: usize,
    pub audio: usize,
    pub next: NextAction,
}

#[derive(Debug, Error)]
pub enum ImportError {
    #[error(transparent)] Library(#[from] LibraryFailure),
    #[error(transparent)] Context(#[from] ContextFailure),
    #[error(transparent)] Audio(#[from] AudioFileFailure),
    #[error("package contains no sentences")]
    Empty,
}

pub fn import_package(deps: &ImportDeps<'_>, package: ImportPackage) -> Result<ImportReport, ImportError> {
    if package.sentences.is_empty() { return Err(ImportError::Empty); }
    let context = deps.context.resolve()?;

    let mut audio_bytes = Vec::with_capacity(package.sentences.len());
    let mut inputs = Vec::with_capacity(package.sentences.len());
    for sentence in package.sentences {
        audio_bytes.push(sentence.audio);
        inputs.push(ImportSentenceInput {
            batch_id: None,
            title: None,
            section: sentence.section,
            target: sentence.target,
            romanisation: sentence.romanisation,
            english: sentence.english,
            literal: sentence.literal,
            register: sentence.register,
            authority: sentence.authority,
            tags: sentence.tags,
            breakdown: sentence.breakdown,
            status: sentence.status,
            provenance: SentenceProvenance::Imported {
                package: package.package_name.clone(),
                source_batch: None,
                source_item: sentence.source_item,
            },
        });
    }

    let report = deps.library.import_sentences(ImportSentences {
        collection: context.default_collection,
        title: context.default_title,
        language: context.profile.code().clone(),
        sentences: inputs,
    })?;

    let mut audio_written = 0usize;
    for (id, bytes) in report.ids.iter().zip(audio_bytes) {
        if let Some(bytes) = bytes {
            let synthesized = SynthesizedAudio {
                bytes,
                backend: context.audio_backend,
                format: AudioFormat::Mp3,
                voice: None,
                model: None,
            };
            let audio = deps.audio_files.write_sentence_audio(id, &synthesized)?;
            deps.library.set_audio(id, audio)?;
            audio_written += 1;
        }
    }

    Ok(ImportReport { imported: report.created, audio: audio_written, next: NextAction::Package })
}
