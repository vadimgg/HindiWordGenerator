#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
//! Prompt packet rendering and strict untrusted-reply parsing.

mod build_reply;
mod error;
mod import_reply;
mod packet;
mod render;

pub use error::PromptAdapterError;
pub use render::HandlebarsPromptEngine;
