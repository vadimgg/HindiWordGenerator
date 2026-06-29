use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceLayout {
    root: PathBuf,
}

impl WorkspaceLayout {
    pub fn new(root: impl Into<PathBuf>) -> Self { Self { root: root.into() } }
    pub fn root(&self) -> &Path { &self.root }
    pub fn config_file(&self) -> PathBuf { self.root.join("config.toml") }
    pub fn library_db(&self) -> PathBuf { self.root.join("library.db") }
    pub fn audio_dir(&self) -> PathBuf { self.root.join("audio") }
    pub fn prompts_dir(&self) -> PathBuf { self.root.join("prompts") }
    pub fn raw_dir(&self) -> PathBuf { self.root.join("raw") }
    pub fn runs_dir(&self) -> PathBuf { self.root.join("runs") }
    pub fn packages_dir(&self) -> PathBuf { self.root.join("packages") }
    pub fn exports_dir(&self) -> PathBuf { self.root.join("exports") }
}
