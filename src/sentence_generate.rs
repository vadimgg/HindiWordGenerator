use crate::accepted_writer::write_sentence_batch;
use crate::config::{load_config, ConfigError, ModelSpec};
use crate::ollama::{HttpOllamaClient, ModelReadiness, SentenceModelClient};
use crate::project::{ProjectRoot, ProjectRootError};
use crate::run_report::{
    unix_now, write_sentence_run_report, SentenceRunReport, SentenceStageReport, ValidationSummary,
    WriteSummary,
};
use crate::sentence_enrichment::{
    merge_staged_enrichment, parse_literal_stage, parse_register_stage, parse_word_breakdown_stage,
    render_stage_prompt, StagedEnrichment, LITERAL_STAGE_ID, REGISTER_STAGE_ID,
    WORD_BREAKDOWN_FROM_TRANSLATION_STAGE_ID,
};
use crate::sentence_plan::{
    generation_plan, PlannedSentenceBatch, PlannedSentenceRow, SentencePlanError,
};
use crate::sentence_validate::{validate_sentence_batch, ExpectedSource};
use crate::source_identity::content_fingerprint;
use std::io;
use std::path::PathBuf;
use std::time::Instant;

const STAGED_PROMPT_PATH: &str = "staged-sentence-generation";

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
    let command_started = Instant::now();
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
    progress(&format!(
        "planned {planned_batches} batch(es) in {}",
        format_elapsed(command_started.elapsed())
    ));

    let readiness_started = Instant::now();
    progress(&format!("checking model {}", model.original));
    let readiness = client.check_model(&model);
    progress(&format!(
        "model check finished in {}",
        format_elapsed(readiness_started.elapsed())
    ));
    if !readiness.ready {
        return Ok(not_ready(model.original, planned_batches, readiness));
    }

    let mut accepted = Vec::new();
    let mut run_reports = Vec::new();

    for (index, batch) in generation_plan.batches.into_iter().enumerate() {
        let batch_number = index + 1;
        let started_at = unix_now();
        let target = root.join(&batch.target_path);
        let mut stages = Vec::new();

        let staged = match run_staged_enrichment_per_sentence(
            client,
            &model,
            readiness.model_digest.as_deref(),
            batch_number,
            planned_batches,
            &batch,
            &mut stages,
        ) {
            Ok(staged) => staged,
            Err(error) => {
                return failed_batch_outcome(
                    root,
                    &batch,
                    &model.original,
                    readiness.model_digest.clone(),
                    started_at,
                    planned_batches,
                    accepted,
                    run_reports,
                    stages,
                    error,
                );
            }
        };

        let validate_started = Instant::now();
        progress(&format!(
            "batch {batch_number}/{planned_batches}: validating merged response"
        ));
        let attempt = merge_staged_enrichment(&batch, staged)
            .map_err(|error| error.to_string())
            .and_then(|candidate| {
                let expected = expected_sources(&batch);
                let validation = validate_sentence_batch(&candidate, &expected);
                if !validation.is_valid() {
                    return Err(validation.errors().join("\n"));
                }
                progress(&format!(
                    "batch {batch_number}/{planned_batches}: validation passed in {}",
                    format_elapsed(validate_started.elapsed())
                ));
                let write_started = Instant::now();
                let write =
                    write_sentence_batch(&target, &candidate).map_err(|error| error.to_string());
                if write.is_ok() {
                    progress(&format!(
                        "batch {batch_number}/{planned_batches}: accepted output written in {}",
                        format_elapsed(write_started.elapsed())
                    ));
                }
                write
            });

        match attempt {
            Ok(write) => {
                let report_started = Instant::now();
                let report = report_for(
                    &batch,
                    &model.original,
                    readiness.model_digest.clone(),
                    started_at,
                    true,
                    stages,
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
                progress(&format!(
                    "batch {batch_number}/{planned_batches}: run report written in {}",
                    format_elapsed(report_started.elapsed())
                ));
                accepted.extend(report.writes.accepted.iter().map(PathBuf::from));
                run_reports.push(report_path);
            }
            Err(error) => {
                let report_started = Instant::now();
                let report = report_for(
                    &batch,
                    &model.original,
                    readiness.model_digest.clone(),
                    started_at,
                    false,
                    stages,
                    vec![error.clone()],
                    Vec::new(),
                    vec![batch.target_path.clone()],
                );
                let report_path = write_sentence_run_report(root, &report)
                    .map_err(|error| SentenceGenerateError::Report(error.to_string()))?;
                progress(&format!(
                    "batch {batch_number}/{planned_batches}: failed run report written in {}",
                    format_elapsed(report_started.elapsed())
                ));
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
    progress(&format!(
        "generation finished in {}",
        format_elapsed(command_started.elapsed())
    ));

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

fn progress(message: &str) {
    println!("  {message}");
}

fn format_elapsed(duration: std::time::Duration) -> String {
    let seconds = duration.as_secs_f64();
    if seconds < 1.0 {
        format!("{:.0}ms", seconds * 1000.0)
    } else if seconds < 60.0 {
        format!("{seconds:.1}s")
    } else {
        format!("{:.1}m", seconds / 60.0)
    }
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

type StagedRunResult = Result<StagedEnrichment, String>;

fn run_staged_enrichment_per_sentence<C: SentenceModelClient>(
    client: &C,
    model: &ModelSpec,
    model_digest: Option<&str>,
    batch_number: usize,
    planned_batches: usize,
    batch: &PlannedSentenceBatch,
    stages: &mut Vec<SentenceStageReport>,
) -> StagedRunResult {
    let mut register = Vec::new();
    let mut literal = Vec::new();
    let mut word_breakdown = Vec::new();

    for (row_index, row) in batch.rows.iter().enumerate() {
        let row_number = row_index + 1;
        let row_total = batch.rows.len();

        let (records, stage) = run_stage(
            client,
            model,
            model_digest,
            batch_number,
            planned_batches,
            row_number,
            row_total,
            REGISTER_STAGE_ID,
            row,
            parse_register_stage,
        );
        stages.push(stage);
        register.extend(records?);

        let (records, stage) = run_stage(
            client,
            model,
            model_digest,
            batch_number,
            planned_batches,
            row_number,
            row_total,
            LITERAL_STAGE_ID,
            row,
            parse_literal_stage,
        );
        stages.push(stage);
        literal.extend(records?);

        let (records, stage) = run_stage(
            client,
            model,
            model_digest,
            batch_number,
            planned_batches,
            row_number,
            row_total,
            WORD_BREAKDOWN_FROM_TRANSLATION_STAGE_ID,
            row,
            parse_word_breakdown_stage,
        );
        stages.push(stage);
        word_breakdown.extend(records?);
    }

    Ok(StagedEnrichment {
        register,
        literal,
        word_breakdown,
    })
}

#[allow(clippy::too_many_arguments)]
fn run_stage<C, T, F>(
    client: &C,
    model: &ModelSpec,
    model_digest: Option<&str>,
    batch_number: usize,
    planned_batches: usize,
    row_number: usize,
    row_total: usize,
    stage_id: &str,
    row: &PlannedSentenceRow,
    parser: F,
) -> (Result<Vec<T>, String>, SentenceStageReport)
where
    C: SentenceModelClient,
    F: Fn(&str) -> Result<Vec<T>, crate::sentence_enrichment::EnrichmentError>,
{
    let started = Instant::now();
    let rendered = match render_stage_prompt(stage_id, std::slice::from_ref(row)) {
        Ok(rendered) => rendered,
        Err(error) => {
            let error = error.to_string();
            return (
                Err(error.clone()),
                stage_report(
                    stage_id,
                    "",
                    "",
                    model,
                    model_digest,
                    started,
                    false,
                    Some(error),
                    None,
                ),
            );
        }
    };
    progress(&format!(
        "batch {batch_number}/{planned_batches}: item {row_number}/{row_total} ({}) stage {stage_id} sending 1 item to model",
        row.id
    ));
    let mut raw_response = None;
    let result = client
        .generate(model, &rendered.prompt)
        .map_err(|error| error.to_string())
        .and_then(|output| {
            raw_response = Some(output.text.clone());
            parser(&output.text).map_err(|error| error.to_string())
        });
    let ok = result.is_ok();
    let error = result.as_ref().err().cloned();
    let elapsed = started.elapsed();
    if ok {
        progress(&format!(
            "batch {batch_number}/{planned_batches}: stage {stage_id} response received in {}",
            format_elapsed(elapsed)
        ));
    } else {
        progress(&format!(
            "batch {batch_number}/{planned_batches}: stage {stage_id} failed in {}",
            format_elapsed(elapsed)
        ));
    }
    (
        result,
        stage_report(
            stage_id,
            &rendered.version,
            &rendered.fingerprint,
            model,
            model_digest,
            started,
            ok,
            error,
            raw_response,
        ),
    )
}

#[allow(clippy::too_many_arguments)]
fn stage_report(
    stage_id: &str,
    prompt_version: &str,
    prompt_fingerprint: &str,
    model: &ModelSpec,
    model_digest: Option<&str>,
    started: Instant,
    ok: bool,
    error: Option<String>,
    raw_response: Option<String>,
) -> SentenceStageReport {
    SentenceStageReport {
        stage_id: stage_id.to_string(),
        prompt_version: prompt_version.to_string(),
        prompt_fingerprint: prompt_fingerprint.to_string(),
        model: model.original.clone(),
        model_digest: model_digest.map(ToString::to_string),
        duration_ms: started.elapsed().as_millis(),
        ok,
        error,
        raw_response,
    }
}

#[allow(clippy::too_many_arguments)]
fn failed_batch_outcome(
    root: &ProjectRoot,
    batch: &PlannedSentenceBatch,
    model: &str,
    model_digest: Option<String>,
    started_at: u64,
    planned_batches: usize,
    accepted: Vec<PathBuf>,
    mut run_reports: Vec<PathBuf>,
    stages: Vec<SentenceStageReport>,
    error: String,
) -> Result<SentenceGenerateOutcome, SentenceGenerateError> {
    let report_started = Instant::now();
    let report = report_for(
        batch,
        model,
        model_digest,
        started_at,
        false,
        stages,
        vec![error.clone()],
        Vec::new(),
        vec![batch.target_path.clone()],
    );
    let report_path = write_sentence_run_report(root, &report)
        .map_err(|error| SentenceGenerateError::Report(error.to_string()))?;
    progress(&format!(
        "failed run report written in {}",
        format_elapsed(report_started.elapsed())
    ));
    run_reports.push(report_path);
    Ok(SentenceGenerateOutcome {
        success: false,
        model: model.to_string(),
        planned_batches,
        accepted,
        run_reports,
        message: Some(error),
        recovery: Some(
            "Inspect the run report, fix prompt/model/source issues, then rerun `hindi sentences generate --max-batches 1`.".to_string(),
        ),
    })
}

#[allow(clippy::too_many_arguments)]
fn report_for(
    batch: &PlannedSentenceBatch,
    model: &str,
    model_digest: Option<String>,
    started_at: u64,
    valid: bool,
    stages: Vec<SentenceStageReport>,
    errors: Vec<String>,
    accepted: Vec<PathBuf>,
    skipped: Vec<PathBuf>,
) -> SentenceRunReport {
    let mut stages = stages;
    if valid {
        for stage in &mut stages {
            stage.raw_response = None;
        }
    }
    let prompt_fingerprint = aggregate_stage_fingerprint(&stages);
    SentenceRunReport {
        command: "hindi sentences generate".to_string(),
        status: if valid { "accepted" } else { "failed" }.to_string(),
        source_files: vec![batch.source_file.to_string_lossy().to_string()],
        targets: vec![batch.target_path.to_string_lossy().to_string()],
        model: model.to_string(),
        model_digest,
        prompt_path: STAGED_PROMPT_PATH.to_string(),
        prompt_fingerprint,
        stages,
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

fn aggregate_stage_fingerprint(stages: &[SentenceStageReport]) -> String {
    let payload = stages
        .iter()
        .map(|stage| {
            format!(
                "{}:{}:{}",
                stage.stage_id, stage.prompt_version, stage.prompt_fingerprint
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    content_fingerprint(payload.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::generate_from;
    use crate::config::ModelSpec;
    use crate::ollama::{ModelOutput, ModelReadiness, SentenceModelClient};
    use crate::project::ProjectRoot;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct FakeClient {
        ready: bool,
        outputs: RefCell<VecDeque<String>>,
        prompts: RefCell<Vec<String>>,
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
            prompt: &str,
        ) -> Result<ModelOutput, crate::ollama::ModelClientError> {
            let text = self
                .outputs
                .borrow_mut()
                .pop_front()
                .ok_or(crate::ollama::ModelClientError::MissingField("fake output"))?;
            self.prompts.borrow_mut().push(prompt.to_string());
            Ok(ModelOutput { text })
        }
    }

    #[test]
    fn generates_valid_sentence_batch_from_staged_outputs() {
        let root = fixture_root();
        let project = ProjectRoot::discover_from(&root).unwrap();
        let client = FakeClient {
            ready: true,
            outputs: RefCell::new(VecDeque::from([
                r#"results:
  - id: "0001"
    register: standard
"#
                .to_string(),
                r#"results:
  - id: "0001"
    literal: "here"
"#
                .to_string(),
                r#"results:
  - id: "0001"
    words:
      - hindi: "यहाँ"
        roman: "yahā̃"
        meaning: "here"
"#
                .to_string(),
                r#"results:
  - id: "0002"
    register: standard
"#
                .to_string(),
                r#"results:
  - id: "0002"
    literal: "there"
"#
                .to_string(),
                r#"results:
  - id: "0002"
    words:
      - hindi: "वहाँ"
        roman: "vahā̃"
        meaning: "there"
"#
                .to_string(),
            ])),
            prompts: RefCell::new(Vec::new()),
        };

        let outcome = generate_from(&project, 1, &client).unwrap();

        assert!(outcome.success);
        assert_eq!(client.prompts.borrow().len(), 6);
        assert!(client.prompts.borrow()[0].contains("hindi: \"यहाँ\""));
        assert!(!client.prompts.borrow()[0].contains("hindi: \"वहाँ\""));
        assert!(client.prompts.borrow()[3].contains("hindi: \"वहाँ\""));
        assert!(!client.prompts.borrow()[3].contains("hindi: \"यहाँ\""));
        assert_eq!(outcome.accepted.len(), 1);
        assert!(root
            .join("output/sentences/example_batch_01.json")
            .is_file());
        assert!(root.join("runs/sentences").is_dir());
        let report = fs::read_to_string(
            fs::read_dir(root.join("runs/sentences"))
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .path(),
        )
        .unwrap();
        assert!(report.contains("\"stages\""));
        assert!(report.contains("sentence/register"));
        assert_eq!(report.matches("\"stage_id\"").count(), 6);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn model_not_ready_writes_no_output() {
        let root = fixture_root();
        let project = ProjectRoot::discover_from(&root).unwrap();
        let client = FakeClient {
            ready: false,
            outputs: RefCell::new(VecDeque::new()),
            prompts: RefCell::new(Vec::new()),
        };

        let outcome = generate_from(&project, 1, &client).unwrap();

        assert!(!outcome.success);
        assert!(outcome.recovery.unwrap().contains("ollama run"));
        assert!(!root.join("output/sentences/example_batch_01.json").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn stage_failure_writes_no_accepted_output() {
        let root = fixture_root();
        let project = ProjectRoot::discover_from(&root).unwrap();
        let client = FakeClient {
            ready: true,
            outputs: RefCell::new(VecDeque::from([
                r#"results: []"#.to_string(),
                r#"results:
  - id: "0001"
    literal: "here"
"#
                .to_string(),
                r#"results:
  - id: "0001"
    words:
      - hindi: "यहाँ"
        roman: "yahā̃"
        meaning: "here"
"#
                .to_string(),
                r#"results: []"#.to_string(),
                r#"results:
  - id: "0002"
    literal: "there"
"#
                .to_string(),
                r#"results:
  - id: "0002"
    words:
      - hindi: "वहाँ"
        roman: "vahā̃"
        meaning: "there"
"#
                .to_string(),
            ])),
            prompts: RefCell::new(Vec::new()),
        };

        let outcome = generate_from(&project, 1, &client).unwrap();

        assert!(!outcome.success);
        assert!(outcome
            .message
            .unwrap()
            .contains("Stage sentence/register did not return item 0001"));
        assert!(!root.join("output/sentences/example_batch_01.json").exists());
        assert!(root.join("runs/sentences").is_dir());
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
            root.join("input/sentences/example.yaml"),
            "title: Test\nsubtitle: Unit\nitems:\n  - id: \"0001\"\n    hindi: \"यहाँ\"\n    romanisation: \"yahā̃\"\n    english: \"Here.\"\n  - id: \"0002\"\n    hindi: \"वहाँ\"\n    romanisation: \"vahā̃\"\n    english: \"There.\"\n",
        )
        .unwrap();
        root
    }

    fn temp_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let count = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("{label}-{}-{nanos}-{count}", std::process::id()))
    }
}
