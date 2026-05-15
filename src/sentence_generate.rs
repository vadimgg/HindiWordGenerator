use crate::accepted_writer::write_sentence_batch;
use crate::config::{load_config, ConfigError};
use crate::ollama::{HttpOllamaClient, ModelReadiness, SentenceModelClient};
use crate::project::{ProjectRoot, ProjectRootError};
use crate::run_report::{
    unix_now, write_sentence_run_report, SentenceRunReport, ValidationSummary, WriteSummary,
};
use crate::sentence_enrichment::{build_prompt, merge_enrichment};
use crate::sentence_plan::{generation_plan, PlannedSentenceBatch, SentencePlanError};
use crate::sentence_validate::{validate_sentence_batch, ExpectedSource};
use crate::source_identity::content_fingerprint;
use std::fs;
use std::io;
use std::path::PathBuf;

const PROMPT_PATH: &str = "generation_prompt_sentences_enrichment.txt";

#[derive(Debug)]
pub enum SentenceGenerateError {
    Project(ProjectRootError),
    Config(ConfigError),
    Plan(SentencePlanError),
    Io { path: PathBuf, source: io::Error },
    Report(String),
}

impl std::fmt::Display for SentenceGenerateError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SentenceGenerateError::Project(error) => write!(formatter, "{error}"),
            SentenceGenerateError::Config(error) => write!(formatter, "{error}"),
            SentenceGenerateError::Plan(error) => write!(formatter, "{error}"),
            SentenceGenerateError::Io { path, source } => {
                write!(formatter, "Could not read {}\n\n{source}", path.display())
            }
            SentenceGenerateError::Report(error) => write!(formatter, "{error}"),
        }
    }
}

impl From<ProjectRootError> for SentenceGenerateError {
    fn from(error: ProjectRootError) -> Self {
        SentenceGenerateError::Project(error)
    }
}

impl From<ConfigError> for SentenceGenerateError {
    fn from(error: ConfigError) -> Self {
        SentenceGenerateError::Config(error)
    }
}

impl From<SentencePlanError> for SentenceGenerateError {
    fn from(error: SentencePlanError) -> Self {
        SentenceGenerateError::Plan(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentenceGenerateOutcome {
    pub success: bool,
    model: String,
    planned_batches: usize,
    accepted: Vec<PathBuf>,
    run_reports: Vec<PathBuf>,
    message: Option<String>,
    recovery: Option<String>,
}

impl SentenceGenerateOutcome {
    pub fn render(&self) -> String {
        let mut output = String::from("Generate Sentences\n\n");
        output.push_str(&format!("  model             {}\n", self.model));
        output.push_str(&format!("  planned batches   {}\n", self.planned_batches));
        output.push_str(&format!("  accepted batches  {}\n", self.accepted.len()));

        if let Some(message) = &self.message {
            output.push_str("\nProblem\n");
            output.push_str(&format!("  {message}\n"));
        }
        if !self.accepted.is_empty() {
            output.push_str("\nAccepted Output\n");
            for path in &self.accepted {
                output.push_str(&format!("  {}\n", path.display()));
            }
        }
        if !self.run_reports.is_empty() {
            output.push_str("\nRun Reports\n");
            for path in &self.run_reports {
                output.push_str(&format!("  {}\n", path.display()));
            }
        }
        if let Some(recovery) = &self.recovery {
            output.push_str("\nRun\n");
            output.push_str(&format!("  {recovery}\n"));
        } else if self.success {
            output.push_str("\nNext\n  hindi sentences audio\n");
        }
        output
    }
}

pub fn generate_from_current_dir(
    max_batches: usize,
) -> Result<SentenceGenerateOutcome, SentenceGenerateError> {
    let root = ProjectRoot::discover_from(current_dir()?)?;
    generate_from(&root, max_batches, &HttpOllamaClient)
}

pub fn generate_from<C: SentenceModelClient>(
    root: &ProjectRoot,
    max_batches: usize,
    client: &C,
) -> Result<SentenceGenerateOutcome, SentenceGenerateError> {
    let config = load_config(root)?;
    let model = config.sentence_generation_model;
    let (plan_summary, generation_plan) = generation_plan(root, max_batches)?;
    if plan_summary.has_errors() {
        return Ok(SentenceGenerateOutcome {
            success: false,
            model: model.original,
            planned_batches: generation_plan.batches.len(),
            accepted: Vec::new(),
            run_reports: Vec::new(),
            message: Some("Planner found source/output problems. Run `hindi sentences plan --max-batches 1` for details.".to_string()),
            recovery: None,
        });
    }
    if generation_plan.batches.is_empty() {
        return Ok(SentenceGenerateOutcome {
            success: true,
            model: model.original,
            planned_batches: 0,
            accepted: Vec::new(),
            run_reports: Vec::new(),
            message: Some("No pending sentence batches.".to_string()),
            recovery: None,
        });
    }
    let planned_batches = generation_plan.batches.len();

    let readiness = client.check_model(&model);
    if !readiness.ready {
        return Ok(not_ready(model.original, planned_batches, readiness));
    }

    let prompt_path = root.join(PROMPT_PATH);
    let prompt_template =
        fs::read_to_string(&prompt_path).map_err(|source| SentenceGenerateError::Io {
            path: prompt_path.clone(),
            source,
        })?;
    let prompt_fingerprint = content_fingerprint(prompt_template.as_bytes());
    let mut accepted = Vec::new();
    let mut run_reports = Vec::new();

    for batch in generation_plan.batches {
        let started_at = unix_now();
        let prompt = build_prompt(&prompt_template, &batch.rows);
        let target = root.join(&batch.target_path);
        let attempt = client
            .generate(&model, &prompt)
            .map_err(|error| error.to_string())
            .and_then(|output| {
                merge_enrichment(&batch, &output.text).map_err(|error| error.to_string())
            })
            .and_then(|candidate| {
                let expected = expected_sources(&batch);
                let validation = validate_sentence_batch(&candidate, &expected);
                if !validation.is_valid() {
                    return Err(validation.errors().join("\n"));
                }
                write_sentence_batch(&target, &candidate).map_err(|error| error.to_string())
            });

        match attempt {
            Ok(write) => {
                let report = report_for(
                    &batch,
                    &model.original,
                    readiness.model_digest.clone(),
                    &prompt_fingerprint,
                    started_at,
                    true,
                    Vec::new(),
                    vec![write
                        .path
                        .strip_prefix(root.path())
                        .unwrap_or(&write.path)
                        .to_path_buf()],
                    Vec::new(),
                );
                let report_path = write_sentence_run_report(root, &report)
                    .map_err(|error| SentenceGenerateError::Report(error.to_string()))?;
                accepted.extend(report.writes.accepted.iter().map(PathBuf::from));
                run_reports.push(report_path);
            }
            Err(error) => {
                let report = report_for(
                    &batch,
                    &model.original,
                    readiness.model_digest.clone(),
                    &prompt_fingerprint,
                    started_at,
                    false,
                    vec![error.clone()],
                    Vec::new(),
                    vec![batch.target_path.clone()],
                );
                let report_path = write_sentence_run_report(root, &report)
                    .map_err(|error| SentenceGenerateError::Report(error.to_string()))?;
                run_reports.push(report_path);
                return Ok(SentenceGenerateOutcome {
                    success: false,
                    model: model.original,
                    planned_batches,
                    accepted,
                    run_reports,
                    message: Some(error),
                    recovery: Some(
                        "Inspect the run report, fix prompt/model/source issues, then rerun `hindi sentences generate --max-batches 1`.".to_string(),
                    ),
                });
            }
        }
    }

    Ok(SentenceGenerateOutcome {
        success: true,
        model: model.original,
        planned_batches,
        accepted,
        run_reports,
        message: None,
        recovery: None,
    })
}

fn current_dir() -> Result<PathBuf, SentenceGenerateError> {
    std::env::current_dir().map_err(|source| SentenceGenerateError::Io {
        path: PathBuf::from("."),
        source,
    })
}

fn not_ready(
    model: String,
    planned_batches: usize,
    readiness: ModelReadiness,
) -> SentenceGenerateOutcome {
    SentenceGenerateOutcome {
        success: false,
        model,
        planned_batches,
        accepted: Vec::new(),
        run_reports: Vec::new(),
        message: Some(readiness.message),
        recovery: readiness.recovery,
    }
}

fn expected_sources(batch: &PlannedSentenceBatch) -> Vec<ExpectedSource> {
    batch
        .rows
        .iter()
        .map(|row| {
            ExpectedSource::new(
                batch.source_file.to_string_lossy().to_string(),
                row.id.clone(),
                row.fingerprint.clone(),
            )
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn report_for(
    batch: &PlannedSentenceBatch,
    model: &str,
    model_digest: Option<String>,
    prompt_fingerprint: &str,
    started_at: u64,
    valid: bool,
    errors: Vec<String>,
    accepted: Vec<PathBuf>,
    skipped: Vec<PathBuf>,
) -> SentenceRunReport {
    SentenceRunReport {
        command: "hindi sentences generate".to_string(),
        status: if valid { "accepted" } else { "failed" }.to_string(),
        source_files: vec![batch.source_file.to_string_lossy().to_string()],
        targets: vec![batch.target_path.to_string_lossy().to_string()],
        model: model.to_string(),
        model_digest,
        prompt_path: PROMPT_PATH.to_string(),
        prompt_fingerprint: prompt_fingerprint.to_string(),
        started_at_unix: started_at,
        finished_at_unix: unix_now(),
        validation: ValidationSummary { valid, errors },
        writes: WriteSummary {
            accepted: accepted
                .iter()
                .map(|path| path.to_string_lossy().to_string())
                .collect(),
            skipped: skipped
                .iter()
                .map(|path| path.to_string_lossy().to_string())
                .collect(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::generate_from;
    use crate::config::ModelSpec;
    use crate::ollama::{ModelOutput, ModelReadiness, SentenceModelClient};
    use crate::project::ProjectRoot;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct FakeClient {
        ready: bool,
        output: String,
    }

    impl SentenceModelClient for FakeClient {
        fn check_model(&self, model: &ModelSpec) -> ModelReadiness {
            ModelReadiness {
                ready: self.ready,
                model_digest: Some("sha256:test".to_string()),
                message: "fake readiness".to_string(),
                recovery: (!self.ready).then(|| model.ollama_run_command()),
            }
        }

        fn generate(
            &self,
            _model: &ModelSpec,
            _prompt: &str,
        ) -> Result<ModelOutput, crate::ollama::ModelClientError> {
            Ok(ModelOutput {
                text: self.output.clone(),
            })
        }
    }

    #[test]
    fn generates_valid_sentence_batch_from_enrichment() {
        let root = fixture_root();
        let project = ProjectRoot::discover_from(&root).unwrap();
        let client = FakeClient {
            ready: true,
            output: r#"{"items":[{"id":"0001","literal":"here","register":"standard","tokens":[{"hindi":"यहाँ","roman":"yahā̃","kind":"word","word_id":"w1"}],"words":[{"id":"w1","hindi":"यहाँ","roman":"yahā̃","meaning":"here"}]}]}"#.to_string(),
        };

        let outcome = generate_from(&project, 1, &client).unwrap();

        assert!(outcome.success);
        assert_eq!(outcome.accepted.len(), 1);
        assert!(root
            .join("output/sentences/example_batch_01.json")
            .is_file());
        assert!(root.join("runs/sentences").is_dir());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn model_not_ready_writes_no_output() {
        let root = fixture_root();
        let project = ProjectRoot::discover_from(&root).unwrap();
        let client = FakeClient {
            ready: false,
            output: String::new(),
        };

        let outcome = generate_from(&project, 1, &client).unwrap();

        assert!(!outcome.success);
        assert!(outcome.recovery.unwrap().contains("ollama run"));
        assert!(!root.join("output/sentences/example_batch_01.json").exists());
        fs::remove_dir_all(root).unwrap();
    }

    fn fixture_root() -> PathBuf {
        let root = temp_path("hindi-generate");
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::create_dir_all(root.join("input/sentences")).unwrap();
        fs::create_dir_all(root.join("output/sentences")).unwrap();
        fs::create_dir_all(root.join("audio")).unwrap();
        fs::write(root.join("docs/DESIGN.md"), "").unwrap();
        fs::write(root.join("docs/ROADMAP.md"), "").unwrap();
        fs::write(
            root.join("generation_prompt_sentences_enrichment.txt"),
            "Prompt",
        )
        .unwrap();
        fs::write(
            root.join("input/sentences/example.yaml"),
            "title: Test\nsubtitle: Unit\nitems:\n  - id: \"0001\"\n    hindi: \"यहाँ\"\n    romanisation: \"yahā̃\"\n    english: \"Here.\"\n",
        )
        .unwrap();
        root
    }

    fn temp_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{label}-{}-{nanos}", std::process::id()))
    }
}
