use crate::root::WorkspaceRoot;
use lingo_application::ports::PromptStage;
use lingo_domain::{BatchId, CardId, RunId};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct WorkspaceLayout {
    root: WorkspaceRoot,
}

impl WorkspaceLayout {
    pub fn new(root: WorkspaceRoot) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &WorkspaceRoot {
        &self.root
    }

    pub fn config_file(&self) -> PathBuf {
        self.root.join("config.toml")
    }

    pub fn deck_profile_file(&self) -> PathBuf {
        self.root.join("profile.toml")
    }

    pub fn deck_prompts_dir(&self) -> PathBuf {
        self.root.join("prompts")
    }

    pub fn raw_dir(&self) -> PathBuf {
        self.root.join("raw")
    }

    pub fn source_dir(&self) -> PathBuf {
        self.root.join("input/sentences")
    }

    pub fn source_file(&self, batch: &BatchId) -> PathBuf {
        self.source_dir().join(format!("{}.yaml", batch.as_str()))
    }

    pub fn card_dir(&self) -> PathBuf {
        self.root.join("output/sentences")
    }

    pub fn card_file(&self, batch: &BatchId) -> PathBuf {
        self.card_dir().join(format!("{}.json", batch.as_str()))
    }

    pub fn audio_dir(&self) -> PathBuf {
        self.root.join("audio/sentences")
    }

    pub fn audio_file(&self, card: &CardId) -> PathBuf {
        self.audio_dir()
            .join(card.batch().as_str())
            .join(format!("{}.mp3", card.source_item().as_str()))
    }

    /// Canonical per-sentence layer: one file per sentence, the unit the
    /// Organize tab reorders/retitles and the package export mirrors.
    pub fn sentences_dir(&self) -> PathBuf {
        self.root.join("sentences")
    }

    pub fn run_dir(&self, stage: PromptStage, run: &RunId) -> PathBuf {
        self.root
            .join("runs")
            .join(stage.wire_name())
            .join(run.as_str())
    }

    pub fn packages_dir(&self) -> PathBuf {
        self.root.join("packages")
    }

    pub fn exports_dir(&self) -> PathBuf {
        self.root.join("exports")
    }

    pub fn all_directories(&self) -> Vec<PathBuf> {
        vec![
            self.raw_dir(),
            self.source_dir(),
            self.card_dir(),
            self.audio_dir(),
            self.root.join("runs"),
            self.packages_dir(),
            self.exports_dir(),
        ]
    }
}
