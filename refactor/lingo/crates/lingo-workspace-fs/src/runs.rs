use crate::atomic_file::{create_atomic, replace_atomic};
use crate::layout::WorkspaceLayout;
use lingo_application::ports::{
    PreparedRun, PromptPacket, PromptStage, RunFailure, RunJournal, RunRecord, StoredFile,
};
use lingo_domain::{BatchId, ProfileId, RunId, SourceSubtitle, SourceTitle};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct FsRunJournal {
    layout: WorkspaceLayout,
}

impl FsRunJournal {
    pub fn new(layout: WorkspaceLayout) -> Self {
        Self { layout }
    }

    fn find_run_dir(&self, run: &RunId) -> Result<(PromptStage, PathBuf), RunFailure> {
        for stage in [PromptStage::Import, PromptStage::Build] {
            let directory = self.layout.run_dir(stage, run);
            if directory.is_dir() {
                return Ok((stage, directory));
            }
        }
        Err(RunFailure::NotFound(run.clone()))
    }
}

impl RunJournal for FsRunJournal {
    fn record_prepared(&self, run: &PreparedRun) -> Result<RunRecord, RunFailure> {
        let directory = self.layout.run_dir(run.stage, &run.id);
        if directory.exists() {
            return Err(RunFailure::Invalid(format!(
                "run already exists: {}",
                run.id
            )));
        }
        fs::create_dir_all(&directory).map_err(run_io)?;
        let metadata = RunMetadataDto::from(run);
        let metadata_bytes = serde_json::to_vec_pretty(&metadata)
            .map_err(|error| RunFailure::Invalid(error.to_string()))?;
        let prompt_path = directory.join("prompt.md");
        let metadata_path = directory.join("run.json");
        create_atomic(&metadata_path, &with_newline(metadata_bytes)).map_err(run_atomic)?;
        if let Err(error) = create_atomic(&prompt_path, run.packet.content.as_bytes()) {
            let _ = fs::remove_dir_all(&directory);
            return Err(run_atomic(error));
        }
        Ok(RunRecord {
            id: run.id.clone(),
            directory: relative_to_root(self.layout.root().path(), &directory),
            prompt_path: relative_to_root(self.layout.root().path(), &prompt_path),
            reply_path: relative_to_root(
                self.layout.root().path(),
                &directory.join(reply_file_name(run.stage)),
            ),
        })
    }

    fn require_prepared(&self, run: &RunId) -> Result<PreparedRun, RunFailure> {
        let (stage, directory) = self.find_run_dir(run)?;
        let metadata_path = directory.join("run.json");
        let metadata: RunMetadataDto =
            serde_json::from_slice(&fs::read(&metadata_path).map_err(run_io)?)
                .map_err(|error| RunFailure::Invalid(error.to_string()))?;
        if metadata.id != run.as_str() || metadata.stage != stage.wire_name() {
            return Err(RunFailure::Invalid(format!(
                "run metadata identity mismatch at {}",
                metadata_path.display()
            )));
        }
        let prompt_path = directory.join("prompt.md");
        let content = fs::read_to_string(&prompt_path).map_err(run_io)?;
        metadata.into_prepared(content)
    }

    fn record_applied(
        &self,
        run: &RunId,
        reply: &str,
        stored: &StoredFile,
    ) -> Result<(), RunFailure> {
        let (stage, directory) = self.find_run_dir(run)?;
        let reply_path = directory.join(reply_file_name(stage));
        if reply_path.exists() {
            replace_atomic(&reply_path, reply.as_bytes()).map_err(run_atomic)?;
        } else {
            create_atomic(&reply_path, reply.as_bytes()).map_err(run_atomic)?;
        }
        let applied = AppliedRunDto {
            format: "lingo.run-applied/v1",
            stored: stored.relative_path().to_string_lossy().replace('\\', "/"),
        };
        let bytes = serde_json::to_vec_pretty(&applied)
            .map_err(|error| RunFailure::Invalid(error.to_string()))?;
        let path = directory.join("applied.json");
        if path.exists() {
            replace_atomic(&path, &with_newline(bytes)).map_err(run_atomic)
        } else {
            create_atomic(&path, &with_newline(bytes)).map_err(run_atomic)
        }
    }
}

#[derive(Serialize, Deserialize)]
struct RunMetadataDto {
    format: String,
    id: String,
    stage: String,
    batch: String,
    profile: String,
    title: String,
    subtitle: Option<String>,
}

impl From<&PreparedRun> for RunMetadataDto {
    fn from(run: &PreparedRun) -> Self {
        Self {
            format: "lingo.run/v1".to_string(),
            id: run.id.as_str().to_string(),
            stage: run.stage.wire_name().to_string(),
            batch: run.batch.as_str().to_string(),
            profile: run.profile.as_str().to_string(),
            title: run.title.as_str().to_string(),
            subtitle: run
                .subtitle
                .as_ref()
                .map(|value| value.as_str().to_string()),
        }
    }
}

impl RunMetadataDto {
    fn into_prepared(self, prompt: String) -> Result<PreparedRun, RunFailure> {
        if self.format != "lingo.run/v1" {
            return Err(RunFailure::Invalid(format!(
                "unsupported run format {:?}",
                self.format
            )));
        }
        let stage = match self.stage.as_str() {
            "import" => PromptStage::Import,
            "build" => PromptStage::Build,
            value => return Err(RunFailure::Invalid(format!("unknown run stage {value:?}"))),
        };
        Ok(PreparedRun {
            id: RunId::parse(self.id).map_err(run_invalid)?,
            stage,
            batch: BatchId::parse(self.batch).map_err(run_invalid)?,
            profile: ProfileId::parse(self.profile).map_err(run_invalid)?,
            title: SourceTitle::parse(self.title).map_err(run_invalid)?,
            subtitle: self
                .subtitle
                .map(SourceSubtitle::parse)
                .transpose()
                .map_err(run_invalid)?,
            packet: PromptPacket {
                stage,
                content: prompt,
            },
        })
    }
}

#[derive(Serialize)]
struct AppliedRunDto {
    format: &'static str,
    stored: String,
}

fn reply_file_name(stage: PromptStage) -> &'static str {
    match stage {
        PromptStage::Import => "reply.yaml",
        PromptStage::Build => "reply.json",
    }
}

fn relative_to_root(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

fn with_newline(mut bytes: Vec<u8>) -> Vec<u8> {
    bytes.push(b'\n');
    bytes
}

fn run_io(error: std::io::Error) -> RunFailure {
    RunFailure::Io(error.to_string())
}

fn run_atomic(error: crate::atomic_file::AtomicFileError) -> RunFailure {
    RunFailure::Io(error.to_string())
}

fn run_invalid(error: impl std::fmt::Display) -> RunFailure {
    RunFailure::Invalid(error.to_string())
}
