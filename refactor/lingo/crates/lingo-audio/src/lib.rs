#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
//! Explicit audio-provider catalog with one bounded retryable fallback.

mod backend;
mod catalog;
mod elevenlabs;
mod error;
mod fallback;
mod gtts;
mod model;

pub use catalog::{AudioCatalog, AudioCatalogBuilder};
pub use error::AudioAdapterError;
