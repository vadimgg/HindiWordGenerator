use crate::ports::{
    LibraryFailure, LibraryStore, PageRequest, ReorderReport, ReorderSentences, SentencePage,
    SentenceQuery, SentenceReport, SentenceSelection, SentenceSort, UpdateSentence, WordPage,
    WordQuery,
};
use lingo_domain::{Sentence, SentenceId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LibraryError {
    #[error(transparent)]
    Store(#[from] LibraryFailure),
}

pub fn list_library(store: &dyn LibraryStore, query: SentenceQuery) -> Result<SentencePage, LibraryError> {
    Ok(store.list_sentences(&query)?)
}

pub fn get_sentence(store: &dyn LibraryStore, id: &SentenceId) -> Result<Sentence, LibraryError> {
    Ok(store.get_sentence(id)?)
}

pub fn reorder_sentences(store: &dyn LibraryStore, request: ReorderSentences) -> Result<ReorderReport, LibraryError> {
    Ok(store.reorder(request)?)
}

pub fn update_sentence(store: &dyn LibraryStore, request: UpdateSentence) -> Result<SentenceReport, LibraryError> {
    Ok(store.update_sentence(request)?)
}

pub fn list_words(store: &dyn LibraryStore, query: WordQuery) -> Result<WordPage, LibraryError> {
    Ok(store.list_words(&query)?)
}

pub fn default_query(selection: SentenceSelection) -> SentenceQuery {
    SentenceQuery { selection, sort: SentenceSort::LibraryOrder, page: PageRequest::default() }
}
