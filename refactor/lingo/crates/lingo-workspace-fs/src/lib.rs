#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
//! File-backed adapters for lingo workspaces, profiles, config, and run journals.

mod atomic_file;
mod codecs;
mod config;
mod error;
mod layout;
mod profiles;
mod root;
mod runs;
mod scan;
mod store;

pub use atomic_file::{AtomicFileError, create_atomic, replace_atomic};
pub use error::FsAdapterError;
pub use layout::WorkspaceLayout;
pub use profiles::FsProfileCatalog;
pub use root::{RootError, WorkspaceRoot};
pub use runs::FsRunJournal;
pub use store::{FsWorkspace, FsWorkspaceBootstrap};

use std::path::PathBuf;

pub fn default_global_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("lingo")
}
