use crate::cli::EvalReportOutput;
use crate::ollama::{HttpOllamaClient, ModelClientError, ModelOutput, RunningModel};
use crate::project::{ProjectRoot, ProjectRootError};
use crate::source_identity::content_fingerprint;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const DEFAULT_FIELDS: &[&str] = &["id", "hindi", "romanisation", "english"];
const GRADE_RESPONSE_MARKER: &str = "## Paste Grader Response Below";

pub trait EvalModelClient {
    fn running_models(&self) -> Result<Vec<RunningModel>, EvalError>;
    fn generate(&self, model: &str, prompt: &str) -> Result<ModelOutput, EvalError>;
}

pub struct HttpEvalModelClient {
    client: HttpOllamaClient,
}

impl Default for HttpEvalModelClient {
    fn default() -> Self {
        Self {
            client: HttpOllamaClient,
        }
    }
}

impl EvalModelClient for HttpEvalModelClient {
    fn running_models(&self) -> Result<Vec<RunningModel>, EvalError> {
        self.client.running_models().map_err(EvalError::Model)
    }

    fn generate(&self, model: &str, prompt: &str) -> Result<ModelOutput, EvalError> {
        self.client
            .generate_model(model, prompt)
            .map_err(EvalError::Model)
    }
}

#[derive(Debug)]
pub enum EvalError {
    Project(ProjectRootError),
    Io { path: PathBuf, source: io::Error },
    Model(ModelClientError),
    UnknownPrompt(String),
    Input(String),
    Template(handlebars::RenderError),
    TemplateRegistration(handlebars::TemplateError),
    Json(serde_json::Error),
    Yaml(serde_yaml::Error),
    Editor(String),
    Grade(String),
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::Project(error) => write!(formatter, "{error}"),
            EvalError::Io { path, source } => write!(
                formatter,
                "Could not read or write {}\n\n{source}",
                path.display()
            ),
            EvalError::Model(error) => write!(formatter, "{error}"),
            EvalError::UnknownPrompt(prompt_id) => write!(
                formatter,
                "Unknown prompt id {prompt_id:?}.\n\nSupported prompt ids:\n{}",
                supported_prompt_list()
            ),
            EvalError::Input(message) => write!(formatter, "{message}"),
            EvalError::Template(error) => {
                write!(formatter, "Could not render prompt template.\n\n{error}")
            }
            EvalError::TemplateRegistration(error) => {
                write!(formatter, "Could not register prompt template.\n\n{error}")
            }
            EvalError::Json(error) => write!(formatter, "Could not parse JSON.\n\n{error}"),
            EvalError::Yaml(error) => write!(formatter, "Could not parse YAML.\n\n{error}"),
            EvalError::Editor(message) => write!(formatter, "{message}"),
            EvalError::Grade(message) => write!(formatter, "{message}"),
        }
    }
}

impl From<ProjectRootError> for EvalError {
    fn from(error: ProjectRootError) -> Self {
        EvalError::Project(error)
    }
}

impl From<serde_json::Error> for EvalError {
    fn from(error: serde_json::Error) -> Self {
        EvalError::Json(error)
    }
}

impl From<serde_yaml::Error> for EvalError {
    fn from(error: serde_yaml::Error) -> Self {
        EvalError::Yaml(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalRunReport {
    selected_model: String,
    model_source: String,
    input_path: PathBuf,
    prompt_id: String,
    item_count: usize,
    fields: Vec<String>,
    render_ms: u128,
    model_ms: u128,
    total_ms: u128,
    run_path: PathBuf,
}

impl EvalRunReport {
    pub fn render(&self) -> String {
        let mut output = String::from("Eval Prompt\n\n");
        output.push_str("Model\n");
        output.push_str(&format!("  selected   {}\n", self.selected_model));
        output.push_str(&format!("  source     {}\n\n", self.model_source));
        output.push_str("Input\n");
        output.push_str(&format!("  file       {}\n", self.input_path.display()));
        output.push_str(&format!("  prompt id  {}\n", self.prompt_id));
        output.push_str(&format!("  items      {}\n", self.item_count));
        output.push_str(&format!("  fields     {}\n\n", self.fields.join(",")));
        output.push_str("Timing\n");
        output.push_str(&format!("  render     {}\n", format_ms(self.render_ms)));
        output.push_str(&format!("  model      {}\n", format_ms(self.model_ms)));
        output.push_str(&format!("  total      {}\n\n", format_ms(self.total_ms)));
        output.push_str("Output\n");
        output.push_str(&format!("  folder     {}\n", self.run_path.display()));
        output.push_str("  prompt     prompt.txt\n");
        output.push_str("  response   response.txt\n");
        output.push_str("  meta       meta.json\n\n");
        output.push_str("Next\n");
        output.push_str(&format!(
            "  hindi eval grade {}\n",
            prompt_scoped_run_id(&self.prompt_id, &self.run_path)
        ));
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalGradeReport {
    run_path: PathBuf,
    prompt_id: String,
    response_source: GradeResponseSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GradeResponseSource {
    Editor,
    File(PathBuf),
}

impl EvalGradeReport {
    pub fn render(&self) -> String {
        let mut output = String::from("Eval Grade\n\n");
        output.push_str("Run\n");
        output.push_str(&format!("  folder     {}\n", self.run_path.display()));
        output.push_str(&format!("  prompt id  {}\n\n", self.prompt_id));
        match &self.response_source {
            GradeResponseSource::Editor => {
                output.push_str("Editor\n");
                output.push_str("  opened     grade_packet.md\n");
                output.push_str("  response   grade_response.txt\n\n");
            }
            GradeResponseSource::File(path) => {
                output.push_str("Import\n");
                output.push_str(&format!("  source     {}\n", path.display()));
                output.push_str("  response   grade_response.txt\n\n");
            }
        }
        output.push_str("Result\n");
        output.push_str("  parsed     ok\n");
        output.push_str("  grade      grade.json\n\n");
        output.push_str("Next\n");
        output.push_str(&format!("  less {}/summary.txt\n", self.run_path.display()));
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalSummaryReport {
    rows: Vec<EvalSummaryRow>,
    color: bool,
    verbose: bool,
    output: EvalReportOutput,
    history: bool,
    hidden_history_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EvalSummaryRow {
    prompt_id: String,
    model: String,
    input_path: String,
    source_items: Vec<EvalSourceItem>,
    item_count: usize,
    model_ms: u128,
    score: Option<GradeTotal>,
    verdict: Option<String>,
    summary: Option<String>,
    run_id: String,
    response: Option<String>,
    model_label: String,
    prompt_version: Option<String>,
    prompt_fingerprint: Option<String>,
    current_prompt: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EvalSourceItem {
    input_path: String,
    id: String,
    hindi: String,
    romanisation: String,
    english: String,
}

impl EvalSummaryReport {
    pub fn render(&self) -> String {
        let mut output = format!("{}\n\n", self.section_title("Eval Report"));
        if self.rows.is_empty() {
            if self.hidden_history_count > 0 && !self.history {
                output.push_str("No eval runs match the current built-in prompt fingerprints.\n\n");
                output.push_str(&format!(
                    "{}\n",
                    dim_or_plain(
                        &format!(
                            "{} older prompt runs hidden; use `hindi eval report --history` to compare them.",
                            self.hidden_history_count
                        ),
                        self.color
                    )
                ));
            } else {
                output.push_str("No eval runs found under eval/.\n");
            }
            return output;
        }

        let grouped_rows = grouped_eval_rows(&self.rows);
        for (index, group) in grouped_rows.iter().enumerate() {
            output.push_str(&format!(
                "{}  {}  {}  {}\n",
                self.section_title(&format!("Eval Set {}", index + 1)),
                dim_or_plain(&group.input_path, self.color),
                dim_or_plain(
                    &format!(
                        "items {}",
                        format_item_ids(&group.item_ids, group.item_count)
                    ),
                    self.color
                ),
                dim_or_plain(&format!("{} runs", group.rows.len()), self.color)
            ));
            output.push_str(&format!(
                "{}\n\n",
                dim_or_plain(
                    &format!(
                        "scope: every result below grades this full item set · {}",
                        self.prompt_scope_text()
                    ),
                    self.color
                )
            ));

            output.push_str(&format!(
                "{}\n",
                self.subsection_title("Evaluated Sentences")
            ));
            for item in &group.source_items {
                output.push_str(&format!("#{}\n", item.id));
                output.push_str(&format!("  Hindi    {}\n", item.hindi));
                output.push_str(&format!("  Roman    {}\n", item.romanisation));
                output.push_str(&format!("  English  {}\n\n", item.english));
            }

            output.push_str(&format!("{}\n", self.subsection_title("Results")));
            output.push_str(&self.render_result_table(&group.rows));
            output.push_str(&self.render_result_summary(&group.rows));
            if self.verbose {
                output.push_str("Run Folder points to eval/<prompt-id>/<run-folder>/.\n");
            }

            output.push_str(&format!("\n{}\n", self.subsection_title("Notes")));
            let mut hidden_info_notes = 0usize;
            for row in ordered_note_rows(&group.rows) {
                if self.hide_info_note(row) {
                    hidden_info_notes += 1;
                    continue;
                }
                let summary = row.summary.as_deref().unwrap_or("not graded yet");
                let label = format!(
                    "{}  {} / {}",
                    note_symbol(row),
                    short_prompt_id(&row.prompt_id),
                    row.display_model()
                );
                let colored_label = color_note_label(&label, row, self.color);
                output.push_str(&format!("  {}\n", colored_label));
                output.push_str(&wrap_note(summary, 4, 92));
                if self.verbose {
                    output.push_str(&format!(
                        "    {}\n",
                        dim_or_plain(
                            &format!(
                                "score {} · verdict {} · run {}",
                                format_grade(row.score.as_ref()),
                                row.verdict.as_deref().unwrap_or("not graded"),
                                short_run_id(&row.run_id)
                            ),
                            self.color
                        )
                    ));
                }
                if self.should_show_output(row) {
                    output.push_str(&render_model_output(row, self.color));
                }
            }
            if hidden_info_notes > 0 {
                let hidden_text = format!(
                    "ℹ  {hidden_info_notes} passing runs have informational notes only; use --verbose to show all notes."
                );
                output.push_str(&format!(
                    "  {}\n",
                    if self.color {
                        paint(&hidden_text, "36")
                    } else {
                        hidden_text
                    }
                ));
            }

            if index + 1 < grouped_rows.len() {
                output.push('\n');
            }
        }

        output
    }

    fn prompt_scope_text(&self) -> String {
        if self.history {
            "showing all prompt versions".to_string()
        } else if self.hidden_history_count > 0 {
            format!(
                "current prompt fingerprints only; {} older runs hidden",
                self.hidden_history_count
            )
        } else {
            "current prompt fingerprints only".to_string()
        }
    }

    fn render_result_table(&self, rows: &[&EvalSummaryRow]) -> String {
        let headers = if self.verbose {
            vec!["Test / Model", "Score", "Time", "Verdict", "Run Folder"]
        } else {
            vec!["Test / Model", "Score", "Time", "Verdict"]
        };
        let plain_rows = grouped_rows_for_table(rows, self.verbose);
        let width_rows = plain_rows
            .iter()
            .map(|row| row.cells.clone())
            .collect::<Vec<_>>();
        let widths = table_widths(&headers, &width_rows);
        let mut output = render_table_header(&headers, &widths, self.color);
        let aligns = if self.verbose {
            vec![
                Align::Left,
                Align::Right,
                Align::Right,
                Align::Left,
                Align::Left,
            ]
        } else {
            vec![Align::Left, Align::Right, Align::Right, Align::Left]
        };
        for table_row in plain_rows {
            let colored = color_table_row(&table_row, self.color, self.verbose);
            output.push_str(&render_table_row_aligned(&colored, &widths, &aligns));
        }
        output
    }

    fn render_result_summary(&self, rows: &[&EvalSummaryRow]) -> String {
        let passed = rows.iter().filter(|row| is_pass(row)).count();
        let failed = rows.iter().filter(|row| is_fail(row)).count();
        let scores = rows
            .iter()
            .filter_map(|row| row.score.as_ref().map(|score| score.pct as usize))
            .collect::<Vec<_>>();
        let avg = if scores.is_empty() {
            "-".to_string()
        } else {
            format!("{}%", scores.iter().sum::<usize>() / scores.len())
        };
        let slowest = rows.iter().max_by_key(|row| row.model_ms);
        let slowest_text = slowest
            .map(|row| {
                format!(
                    "{} / {} {}",
                    short_prompt_id(&row.prompt_id),
                    row.display_model(),
                    format_ms(row.model_ms)
                )
            })
            .unwrap_or_else(|| "-".to_string());
        format!(
            "{}\n\n",
            dim_or_plain(
                &format!(
                    "Summary: {passed} passed  ·  {failed} failed  ·  avg {avg}  ·  slowest {slowest_text}"
                ),
                self.color
            )
        )
    }

    fn should_show_output(&self, row: &EvalSummaryRow) -> bool {
        match self.output {
            EvalReportOutput::None => false,
            EvalReportOutput::Failures => is_fail(row),
            EvalReportOutput::All => true,
        }
    }

    fn hide_info_note(&self, row: &EvalSummaryRow) -> bool {
        !self.verbose && self.output != EvalReportOutput::All && !is_fail(row) && !is_warning(row)
    }

    fn section_title(&self, text: &str) -> String {
        if self.color {
            paint(text, "1;36")
        } else {
            text.to_string()
        }
    }

    fn subsection_title(&self, text: &str) -> String {
        if self.color {
            paint(text, "1;34")
        } else {
            text.to_string()
        }
    }
}

impl EvalSummaryRow {
    fn item_ids(&self) -> Vec<String> {
        self.source_items
            .iter()
            .map(|item| item.id.clone())
            .collect()
    }

    fn display_model(&self) -> String {
        if self.model_label.is_empty() {
            strip_ollama_prefix(&self.model).to_string()
        } else {
            self.model_label.clone()
        }
    }
}

#[derive(Debug)]
struct EvalRowGroup<'a> {
    input_path: String,
    item_ids: Vec<String>,
    item_count: usize,
    source_items: Vec<EvalSourceItem>,
    rows: Vec<&'a EvalSummaryRow>,
}

#[derive(Debug, Clone)]
struct PromptTemplate {
    id: &'static str,
    version: &'static str,
    input_template: &'static str,
    grade_template: &'static str,
    threshold_pct: u8,
}

#[derive(Debug, Deserialize)]
struct SourceYaml {
    title: Option<String>,
    subtitle: Option<String>,
    items: Vec<Map<String, Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EvalMeta {
    run_id: String,
    prompt_id: String,
    #[serde(default)]
    prompt_version: Option<String>,
    #[serde(default)]
    prompt_fingerprint: Option<String>,
    input_path: String,
    fields: Vec<String>,
    max_items: Option<usize>,
    item_count: usize,
    model: String,
    model_digest: Option<String>,
    model_source: String,
    started_at: String,
    finished_at: String,
    timing_ms: EvalTiming,
    artifacts: EvalArtifacts,
}

#[derive(Debug, Serialize, Deserialize)]
struct EvalTiming {
    render: u128,
    model: u128,
    total: u128,
}

#[derive(Debug, Serialize, Deserialize)]
struct EvalArtifacts {
    prompt: String,
    response: String,
    summary: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Grade {
    run_id: String,
    grader: String,
    graded_at: String,
    scores: GradeScores,
    total: GradeTotal,
    verdict: String,
    item_flags: Vec<Value>,
    summary: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
struct GradeScores {
    accuracy: AxisScore,
    completeness: AxisScore,
    format_compliance: AxisScore,
    consistency: AxisScore,
    confidence: AxisScore,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
struct AxisScore {
    score: u8,
    max: u8,
    note: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
struct GradeTotal {
    score: u8,
    max: u8,
    pct: u8,
}

pub fn run_from_current_dir(
    input: &str,
    prompt_id: &str,
    fields: Option<&str>,
    max_items: Option<usize>,
) -> Result<EvalRunReport, EvalError> {
    let root = ProjectRoot::discover_from(current_dir()?)?;
    run_with_client(
        &root,
        input,
        prompt_id,
        fields,
        max_items,
        &HttpEvalModelClient::default(),
    )
}

pub fn run_with_client<C: EvalModelClient>(
    root: &ProjectRoot,
    input: &str,
    prompt_id: &str,
    fields: Option<&str>,
    max_items: Option<usize>,
    client: &C,
) -> Result<EvalRunReport, EvalError> {
    let total_started = Instant::now();
    let started_at = timestamp_string();
    let template = prompt_template(prompt_id)?;
    let prompt_version = template.version.to_string();
    let prompt_fingerprint = prompt_fingerprint(&template);
    let selected_fields = parse_fields(fields)?;
    let input_path = resolve_input_path(root, input);
    let input_relative = relative_to_root(root, &input_path);
    let input_yaml = fs::read_to_string(&input_path).map_err(|source| EvalError::Io {
        path: input_path.clone(),
        source,
    })?;
    let source: SourceYaml = serde_yaml::from_str(&input_yaml)?;
    let items = selected_items(&source, &selected_fields, max_items)?;
    eprintln!("checking running Ollama model");
    let model = select_running_model(client.running_models()?)?;

    let run_id = format!("{}_{}", timestamp_run_id(), model_slug(&model.name));
    let run_path = root.join(Path::new("eval").join(prompt_id).join(&run_id));
    let run_relative = relative_to_root(root, &run_path);
    let render_started = Instant::now();
    let context = json!({
        "input_yaml": input_yaml,
        "items_yaml": serde_yaml::to_string(&items)?,
        "items": items,
        "input_path": input_relative.to_string_lossy(),
        "prompt_id": prompt_id,
        "run_path": run_relative.to_string_lossy(),
        "title": source.title,
        "subtitle": source.subtitle,
    });
    let prompt = render_template(template.input_template, &context)?;
    let render_ms = elapsed_ms(render_started.elapsed());

    eprintln!("sending rendered prompt to model");
    let model_started = Instant::now();
    let output = client.generate(&model.name, &prompt)?;
    let model_ms = elapsed_ms(model_started.elapsed());
    eprintln!("model response received in {}", format_ms(model_ms));

    fs::create_dir_all(&run_path).map_err(|source| EvalError::Io {
        path: run_path.clone(),
        source,
    })?;
    write_file(&run_path.join("prompt.txt"), &prompt)?;
    write_file(&run_path.join("response.txt"), &output.text)?;

    let total_ms = elapsed_ms(total_started.elapsed());
    let finished_at = timestamp_string();
    let meta = EvalMeta {
        run_id: prompt_scoped_run_id(prompt_id, &run_path),
        prompt_id: prompt_id.to_string(),
        prompt_version: Some(prompt_version),
        prompt_fingerprint: Some(prompt_fingerprint),
        input_path: input_relative.to_string_lossy().to_string(),
        fields: selected_fields.clone(),
        max_items,
        item_count: context["items"].as_array().map_or(0, Vec::len),
        model: format!("ollama:{}", model.name),
        model_digest: model.digest,
        model_source: "Ollama /api/ps".to_string(),
        started_at,
        finished_at,
        timing_ms: EvalTiming {
            render: render_ms,
            model: model_ms,
            total: total_ms,
        },
        artifacts: EvalArtifacts {
            prompt: "prompt.txt".to_string(),
            response: "response.txt".to_string(),
            summary: "summary.txt".to_string(),
        },
    };
    write_json(&run_path.join("meta.json"), &meta)?;
    write_file(&run_path.join("summary.txt"), &render_summary(&meta, None))?;

    Ok(EvalRunReport {
        selected_model: meta.model.clone(),
        model_source: meta.model_source.clone(),
        input_path: input_relative,
        prompt_id: prompt_id.to_string(),
        item_count: meta.item_count,
        fields: selected_fields,
        render_ms,
        model_ms,
        total_ms,
        run_path: run_relative,
    })
}

pub fn grade_from_current_dir(
    run: &str,
    response_path: Option<&str>,
) -> Result<EvalGradeReport, EvalError> {
    let root = ProjectRoot::discover_from(current_dir()?)?;
    grade_from(&root, run, response_path)
}

pub fn report_from_current_dir(
    color: bool,
    verbose: bool,
    output: EvalReportOutput,
    history: bool,
) -> Result<EvalSummaryReport, EvalError> {
    let root = ProjectRoot::discover_from(current_dir()?)?;
    report_from(&root, color, verbose, output, history)
}

fn report_from(
    root: &ProjectRoot,
    color: bool,
    verbose: bool,
    output: EvalReportOutput,
    history: bool,
) -> Result<EvalSummaryReport, EvalError> {
    let eval_root = root.join("eval");
    if !eval_root.is_dir() {
        return Ok(EvalSummaryReport {
            rows: Vec::new(),
            color,
            verbose,
            output,
            history,
            hidden_history_count: 0,
        });
    }

    let mut meta_paths = Vec::new();
    collect_meta_paths(&eval_root, &mut meta_paths)?;
    meta_paths.sort();

    let mut rows = Vec::new();
    for meta_path in meta_paths {
        let run_path = meta_path.parent().unwrap_or(&eval_root).to_path_buf();
        let meta_content = fs::read_to_string(&meta_path).map_err(|source| EvalError::Io {
            path: meta_path.clone(),
            source,
        })?;
        let meta: EvalMeta = serde_json::from_str(&meta_content)?;
        let grade = read_optional_grade(&run_path)?;
        let source_items = source_items_for_meta(root, &meta);
        let response = read_optional_response(&run_path)?;
        let current_prompt = matches_current_prompt(&meta);
        rows.push(EvalSummaryRow {
            prompt_id: meta.prompt_id,
            model: meta.model,
            input_path: meta.input_path,
            source_items,
            item_count: meta.item_count,
            model_ms: meta.timing_ms.model,
            score: grade.as_ref().map(|grade| grade.total.clone()),
            verdict: grade.as_ref().map(|grade| grade.verdict.clone()),
            summary: grade.as_ref().map(|grade| grade.summary.clone()),
            run_id: relative_to_root(root, &run_path)
                .to_string_lossy()
                .to_string(),
            response,
            model_label: String::new(),
            prompt_version: meta.prompt_version,
            prompt_fingerprint: meta.prompt_fingerprint,
            current_prompt,
        });
    }

    let total_rows = rows.len();
    if !history {
        rows.retain(|row| row.current_prompt);
    }
    let hidden_history_count = total_rows.saturating_sub(rows.len());

    rows.sort_by(|left, right| {
        left.prompt_id
            .cmp(&right.prompt_id)
            .then_with(|| left.run_id.cmp(&right.run_id))
    });

    Ok(EvalSummaryReport {
        rows: label_repeated_model_runs(rows, history),
        color,
        verbose,
        output,
        history,
        hidden_history_count,
    })
}

pub fn grade_from(
    root: &ProjectRoot,
    run: &str,
    response_path: Option<&str>,
) -> Result<EvalGradeReport, EvalError> {
    let run_path = resolve_run_path(root, run);
    if !run_path.is_dir() {
        return Err(EvalError::Input(format!("Eval run not found: {run}.")));
    }
    let meta_path = run_path.join("meta.json");
    if !meta_path.is_file() {
        return Err(EvalError::Input(
            "Eval run is missing meta.json.".to_string(),
        ));
    }
    let meta_content = fs::read_to_string(&meta_path).map_err(|source| EvalError::Io {
        path: meta_path.clone(),
        source,
    })?;
    let meta: EvalMeta = serde_json::from_str(&meta_content)?;
    let template = prompt_template(&meta.prompt_id)?;
    let model_prompt_path = run_path.join("prompt.txt");
    let model_prompt = fs::read_to_string(&model_prompt_path).map_err(|source| EvalError::Io {
        path: model_prompt_path,
        source,
    })?;
    let model_response_path = run_path.join("response.txt");
    let response = fs::read_to_string(&model_response_path).map_err(|source| EvalError::Io {
        path: model_response_path,
        source,
    })?;
    let context = json!({
        "run_id": meta.run_id,
        "prompt_id": meta.prompt_id,
        "input_path": meta.input_path,
        "fields": meta.fields,
        "item_count": meta.item_count,
        "model": meta.model,
        "prompt": model_prompt,
        "response": response,
        "threshold_pct": template.threshold_pct,
    });
    let grade_prompt = render_template(template.grade_template, &context)?;
    write_file(&run_path.join("grade_prompt.txt"), &grade_prompt)?;
    let packet =
        format!("## Grading Prompt\n\n{grade_prompt}\n\n{GRADE_RESPONSE_MARKER}\n\n```yaml\n```\n");
    let packet_path = run_path.join("grade_packet.md");
    write_file(&packet_path, &packet)?;
    let (grade_response, response_source) = if let Some(response_path) = response_path {
        let path = resolve_input_path(root, response_path);
        let content = fs::read_to_string(&path).map_err(|source| EvalError::Io {
            path: path.clone(),
            source,
        })?;
        (
            content,
            GradeResponseSource::File(relative_to_root(root, &path)),
        )
    } else {
        eprintln!("opening grade_packet.md in $EDITOR");
        open_editor(&packet_path)?;
        let packet_after = fs::read_to_string(&packet_path).map_err(|source| EvalError::Io {
            path: packet_path.clone(),
            source,
        })?;
        (
            extract_grade_response(&packet_after)?,
            GradeResponseSource::Editor,
        )
    };
    write_file(&run_path.join("grade_response.txt"), &grade_response)?;
    let grade = parse_grade(&grade_response)?;
    write_json(&run_path.join("grade.json"), &grade)?;
    let meta_for_summary: EvalMeta = serde_json::from_str(&meta_content)?;
    write_file(
        &run_path.join("summary.txt"),
        &render_summary(&meta_for_summary, Some(&grade)),
    )?;

    Ok(EvalGradeReport {
        run_path: relative_to_root(root, &run_path),
        prompt_id: meta_for_summary.prompt_id,
        response_source,
    })
}

fn prompt_template(prompt_id: &str) -> Result<PromptTemplate, EvalError> {
    prompt_templates()
        .into_iter()
        .find(|template| template.id == prompt_id)
        .ok_or_else(|| EvalError::UnknownPrompt(prompt_id.to_string()))
}

fn prompt_fingerprint(template: &PromptTemplate) -> String {
    let content = format!(
        "id:{}\nversion:{}\nthreshold:{}\n--- input ---\n{}\n--- grade ---\n{}",
        template.id,
        template.version,
        template.threshold_pct,
        template.input_template,
        template.grade_template
    );
    content_fingerprint(content.as_bytes())
}

fn matches_current_prompt(meta: &EvalMeta) -> bool {
    let Some(stored_fingerprint) = meta.prompt_fingerprint.as_deref() else {
        return false;
    };
    let Ok(template) = prompt_template(&meta.prompt_id) else {
        return false;
    };
    stored_fingerprint == prompt_fingerprint(&template)
}

fn collect_meta_paths(directory: &Path, paths: &mut Vec<PathBuf>) -> Result<(), EvalError> {
    for entry in fs::read_dir(directory).map_err(|source| EvalError::Io {
        path: directory.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| EvalError::Io {
            path: directory.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_meta_paths(&path, paths)?;
        } else if path.file_name().and_then(|value| value.to_str()) == Some("meta.json") {
            paths.push(path);
        }
    }
    Ok(())
}

fn read_optional_grade(run_path: &Path) -> Result<Option<Grade>, EvalError> {
    let path = run_path.join("grade.json");
    if !path.is_file() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path).map_err(|source| EvalError::Io {
        path: path.clone(),
        source,
    })?;
    serde_json::from_str(&content)
        .map(Some)
        .map_err(EvalError::Json)
}

fn read_optional_response(run_path: &Path) -> Result<Option<String>, EvalError> {
    let path = run_path.join("response.txt");
    if !path.is_file() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path).map_err(|source| EvalError::Io {
        path: path.clone(),
        source,
    })?;
    Ok(Some(content))
}

fn source_items_for_meta(root: &ProjectRoot, meta: &EvalMeta) -> Vec<EvalSourceItem> {
    let input_path = resolve_input_path(root, &meta.input_path);
    let Ok(content) = fs::read_to_string(&input_path) else {
        return Vec::new();
    };
    let Ok(source) = serde_yaml::from_str::<SourceYaml>(&content) else {
        return Vec::new();
    };
    let input_relative = relative_to_root(root, &input_path)
        .to_string_lossy()
        .to_string();
    let limit = meta
        .max_items
        .unwrap_or(meta.item_count)
        .min(source.items.len());
    let mut source_items = Vec::new();
    for item in source.items.iter().take(limit) {
        let id = item_string(item, "id").unwrap_or_else(|| "<unknown>".to_string());
        source_items.push(EvalSourceItem {
            input_path: input_relative.clone(),
            id,
            hindi: item_string(item, "hindi").unwrap_or_default(),
            romanisation: item_string(item, "romanisation").unwrap_or_default(),
            english: item_string(item, "english").unwrap_or_default(),
        });
    }
    source_items
}

fn grouped_eval_rows(rows: &[EvalSummaryRow]) -> Vec<EvalRowGroup<'_>> {
    let mut groups_by_key: BTreeMap<String, EvalRowGroup<'_>> = BTreeMap::new();
    for row in rows {
        let item_ids = row.item_ids();
        let key = format!("{}#{}", row.input_path, item_ids.join(","));
        groups_by_key
            .entry(key)
            .and_modify(|group| group.rows.push(row))
            .or_insert_with(|| EvalRowGroup {
                input_path: row.input_path.clone(),
                item_ids,
                item_count: row.item_count,
                source_items: row.source_items.clone(),
                rows: vec![row],
            });
    }
    groups_by_key.into_values().collect()
}

fn label_repeated_model_runs(mut rows: Vec<EvalSummaryRow>, history: bool) -> Vec<EvalSummaryRow> {
    let mut counts: BTreeMap<(String, String), usize> = BTreeMap::new();
    for row in &rows {
        *counts.entry(label_group_key(row, history)).or_default() += 1;
    }
    let mut seen: BTreeMap<(String, String), usize> = BTreeMap::new();
    for row in &mut rows {
        let key = label_group_key(row, history);
        let count = *counts.get(&key).unwrap_or(&1);
        let mut base = strip_ollama_prefix(&row.model).to_string();
        if history {
            base.push(' ');
            base.push_str(&prompt_history_label(row));
        }
        if count > 1 {
            let entry = seen.entry(key).or_default();
            *entry += 1;
            row.model_label = format!("{base} #{}", *entry);
        } else {
            row.model_label = base;
        }
    }
    rows
}

fn label_group_key(row: &EvalSummaryRow, history: bool) -> (String, String) {
    let source_key = format!("{}#{}", row.input_path, row.item_ids().join(","));
    (
        format!("{}#{}", source_key, row.prompt_id),
        model_group_key(row, history),
    )
}

fn model_group_key(row: &EvalSummaryRow, history: bool) -> String {
    if history {
        format!("{}#{}", row.model, prompt_history_label(row))
    } else {
        row.model.clone()
    }
}

fn prompt_history_label(row: &EvalSummaryRow) -> String {
    row.prompt_version
        .as_ref()
        .map(|version| format!("@{version}"))
        .unwrap_or_else(|| "@legacy".to_string())
}

fn item_string(item: &Map<String, Value>, field: &str) -> Option<String> {
    item.get(field)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn table_widths(headers: &[&str], rows: &[Vec<String>]) -> Vec<usize> {
    let mut widths = headers
        .iter()
        .map(|header| visible_len(header))
        .collect::<Vec<_>>();
    for row in rows {
        for (index, cell) in row.iter().enumerate() {
            widths[index] = widths[index].max(visible_len(cell));
        }
    }
    widths
}

#[derive(Debug, Clone, Copy)]
enum Align {
    Left,
    Right,
}

#[derive(Debug, Clone)]
struct ReportTableRow<'a> {
    kind: ReportTableRowKind<'a>,
    cells: Vec<String>,
}

#[derive(Debug, Clone)]
enum ReportTableRowKind<'a> {
    Test,
    Model(&'a EvalSummaryRow),
}

fn render_table_header(headers: &[&str], widths: &[usize], color: bool) -> String {
    let header_cells = headers
        .iter()
        .map(|header| {
            if color {
                paint(header, "1;37")
            } else {
                header.to_string()
            }
        })
        .collect::<Vec<_>>();
    let aligns = vec![Align::Left; headers.len()];
    let mut output = render_table_row_aligned(&header_cells, widths, &aligns);
    output.push_str(&format!(
        "{}\n",
        widths
            .iter()
            .map(|width| "-".repeat(*width))
            .collect::<Vec<_>>()
            .join("  ")
    ));
    output
}

fn render_table_row_aligned(cells: &[String], widths: &[usize], aligns: &[Align]) -> String {
    let mut parts = Vec::new();
    for ((cell, width), align) in cells.iter().zip(widths).zip(aligns) {
        parts.push(match align {
            Align::Left => pad_cell(cell, *width),
            Align::Right => pad_cell_left(cell, *width),
        });
    }
    format!("{}\n", parts.join("  "))
}

fn pad_cell(cell: &str, width: usize) -> String {
    let padding = width.saturating_sub(visible_len(cell));
    format!("{cell}{}", " ".repeat(padding))
}

fn pad_cell_left(cell: &str, width: usize) -> String {
    let padding = width.saturating_sub(visible_len(cell));
    format!("{}{}", " ".repeat(padding), cell)
}

fn visible_len(value: &str) -> usize {
    let mut length = 0usize;
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for code in chars.by_ref() {
                if code == 'm' {
                    break;
                }
            }
        } else {
            length += 1;
        }
    }
    length
}

fn format_item_ids(item_ids: &[String], item_count: usize) -> String {
    if item_ids.is_empty() {
        return item_count.to_string();
    }
    item_ids.join(",")
}

fn format_grade(score: Option<&GradeTotal>) -> String {
    score
        .map(|score| format!("{}/{} {}%", score.score, score.max, score.pct))
        .unwrap_or_else(|| "-".to_string())
}

fn format_score_pct(score: Option<&GradeTotal>) -> String {
    score
        .map(|score| format!("{}%", score.pct))
        .unwrap_or_else(|| "-".to_string())
}

fn grouped_rows_for_table<'a>(
    rows: &[&'a EvalSummaryRow],
    verbose: bool,
) -> Vec<ReportTableRow<'a>> {
    let mut by_prompt: BTreeMap<String, Vec<&EvalSummaryRow>> = BTreeMap::new();
    for row in rows {
        by_prompt
            .entry(row.prompt_id.clone())
            .or_default()
            .push(*row);
    }

    let mut table_rows = Vec::new();
    for (prompt_id, mut prompt_rows) in by_prompt {
        prompt_rows.sort_by(|left, right| {
            strip_ollama_prefix(&left.model)
                .cmp(strip_ollama_prefix(&right.model))
                .then_with(|| left.run_id.cmp(&right.run_id))
        });
        let test_name = short_prompt_id(&prompt_id);
        table_rows.push(ReportTableRow {
            kind: ReportTableRowKind::Test,
            cells: if verbose {
                vec![
                    test_name,
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                ]
            } else {
                vec![test_name, String::new(), String::new(), String::new()]
            },
        });
        for row in prompt_rows {
            let mut cells = vec![
                format!("  {}", row.display_model()),
                if verbose {
                    format_grade(row.score.as_ref())
                } else {
                    format_score_pct(row.score.as_ref())
                },
                format_ms(row.model_ms),
                verdict_symbol(row),
            ];
            if verbose {
                cells.push(short_run_id(&row.run_id));
            }
            table_rows.push(ReportTableRow {
                kind: ReportTableRowKind::Model(row),
                cells,
            });
        }
    }
    table_rows
}

fn color_table_row(row: &ReportTableRow<'_>, color: bool, verbose: bool) -> Vec<String> {
    match &row.kind {
        ReportTableRowKind::Test => row
            .cells
            .iter()
            .enumerate()
            .map(|(index, cell)| {
                if index == 0 {
                    color_test_name(cell, color)
                } else {
                    cell.clone()
                }
            })
            .collect(),
        ReportTableRowKind::Model(summary_row) => {
            let mut cells = vec![
                color_model(&row.cells[0], color),
                color_grade(&row.cells[1], summary_row.score.as_ref(), color),
                color_time(&row.cells[2], summary_row.model_ms, color),
                color_verdict_symbol(&row.cells[3], summary_row, color),
            ];
            if verbose {
                cells.push(color_run_id(&row.cells[4], color));
            }
            cells
        }
    }
}

fn color_grade(text: &str, score: Option<&GradeTotal>, color: bool) -> String {
    if !color {
        return text.to_string();
    }
    let code = match score.map(|score| score.pct) {
        Some(pct) if pct >= 85 => "32",
        Some(pct) if pct >= 70 => "33",
        Some(_) => "31",
        None => "2",
    };
    paint(text, code)
}

fn color_test_name(text: &str, color: bool) -> String {
    if color {
        paint(text, "1;35")
    } else {
        text.to_string()
    }
}

fn color_model(text: &str, color: bool) -> String {
    if color {
        paint(text, "36")
    } else {
        text.to_string()
    }
}

fn color_time(text: &str, ms: u128, color: bool) -> String {
    if !color {
        return text.to_string();
    }
    let code = if ms <= 5_000 {
        "32"
    } else if ms <= 20_000 {
        "33"
    } else {
        "31"
    };
    paint(text, code)
}

fn color_verdict_symbol(text: &str, row: &EvalSummaryRow, color: bool) -> String {
    if !color {
        return text.to_string();
    }
    if is_pass(row) {
        paint(text, "32")
    } else if is_fail(row) {
        paint(text, "31")
    } else {
        paint(text, "33")
    }
}

fn color_run_id(text: &str, color: bool) -> String {
    if color {
        paint(text, "2")
    } else {
        text.to_string()
    }
}

fn paint(text: &str, code: &str) -> String {
    format!("\x1b[{code}m{text}\x1b[0m")
}

fn dim_or_plain(text: &str, color: bool) -> String {
    if color {
        paint(text, "2")
    } else {
        text.to_string()
    }
}

fn wrap_note(text: &str, indent: usize, width: usize) -> String {
    let prefix = " ".repeat(indent);
    let mut output = String::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && visible_len(&line) + 1 + visible_len(word) > width {
            output.push_str(&prefix);
            output.push_str(line.trim_end());
            output.push('\n');
            line.clear();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        output.push_str(&prefix);
        output.push_str(line.trim_end());
        output.push('\n');
    }
    output
}

fn render_model_output(row: &EvalSummaryRow, color: bool) -> String {
    let Some(response) = row.response.as_deref() else {
        return format!("    {}\n", dim_or_plain("model output unavailable", color));
    };
    let mut output = format!("    {}\n", dim_or_plain("model output:", color));
    let cleaned = strip_optional_fence(response).trim();
    let mut count = 0usize;
    for line in cleaned.lines() {
        if count >= 18 {
            output.push_str(&format!(
                "      {}\n",
                dim_or_plain(
                    "... output truncated; inspect response.txt for full text",
                    color
                )
            ));
            break;
        }
        let trimmed = line.trim_end();
        if trimmed.is_empty() && output.ends_with("\n\n") {
            continue;
        }
        output.push_str("      ");
        output.push_str(trimmed);
        output.push('\n');
        count += 1;
    }
    output
}

fn short_prompt_id(prompt_id: &str) -> String {
    prompt_id
        .strip_prefix("sentence/")
        .unwrap_or(prompt_id)
        .to_string()
}

fn strip_ollama_prefix(model: &str) -> &str {
    model.strip_prefix("ollama:").unwrap_or(model)
}

fn short_run_id(run_id: &str) -> String {
    run_id.rsplit('/').next().unwrap_or(run_id).to_string()
}

fn short_fingerprint(fingerprint: &str) -> &str {
    fingerprint
        .strip_prefix("sha256:")
        .unwrap_or(fingerprint)
        .get(..12)
        .unwrap_or(fingerprint)
}

fn is_pass(row: &EvalSummaryRow) -> bool {
    row.verdict
        .as_deref()
        .is_some_and(|verdict| verdict.eq_ignore_ascii_case("pass"))
}

fn is_fail(row: &EvalSummaryRow) -> bool {
    row.verdict
        .as_deref()
        .is_some_and(|verdict| verdict.eq_ignore_ascii_case("fail"))
}

fn is_warning(row: &EvalSummaryRow) -> bool {
    !is_fail(row) && row.score.as_ref().is_some_and(|score| score.pct < 90)
}

fn verdict_symbol(row: &EvalSummaryRow) -> String {
    if is_pass(row) {
        "✓ pass".to_string()
    } else if is_fail(row) {
        "✗ fail".to_string()
    } else {
        "? not graded".to_string()
    }
}

fn note_symbol(row: &EvalSummaryRow) -> &'static str {
    if is_fail(row) {
        "✗"
    } else if is_warning(row) {
        "⚠"
    } else {
        "ℹ"
    }
}

fn color_note_label(text: &str, row: &EvalSummaryRow, color: bool) -> String {
    if !color {
        return text.to_string();
    }
    if is_fail(row) {
        paint(text, "31")
    } else if is_warning(row) {
        paint(text, "33")
    } else {
        paint(text, "36")
    }
}

fn ordered_note_rows<'a>(rows: &[&'a EvalSummaryRow]) -> Vec<&'a EvalSummaryRow> {
    let mut ordered = rows.to_vec();
    ordered.sort_by(|left, right| {
        note_priority(left)
            .cmp(&note_priority(right))
            .then_with(|| left.prompt_id.cmp(&right.prompt_id))
            .then_with(|| left.model.cmp(&right.model))
            .then_with(|| left.run_id.cmp(&right.run_id))
    });
    ordered
}

fn note_priority(row: &EvalSummaryRow) -> u8 {
    if is_fail(row) {
        0
    } else if is_warning(row) {
        1
    } else {
        2
    }
}

fn prompt_templates() -> Vec<PromptTemplate> {
    vec![
        PromptTemplate {
            id: "sentence/source-qa",
            version: "v2",
            input_template: include_str!("eval_prompts/sentence_source_qa.yaml.hbs"),
            grade_template: include_str!("eval_prompts/sentence_source_qa.grade.yaml.hbs"),
            threshold_pct: 80,
        },
        PromptTemplate {
            id: "sentence/english",
            version: "v2",
            input_template: include_str!("eval_prompts/sentence_english.yaml.hbs"),
            grade_template: include_str!("eval_prompts/sentence_english.grade.yaml.hbs"),
            threshold_pct: 80,
        },
        PromptTemplate {
            id: "sentence/literal",
            version: "v3",
            input_template: include_str!("eval_prompts/sentence_literal.yaml.hbs"),
            grade_template: include_str!("eval_prompts/sentence_literal.grade.yaml.hbs"),
            threshold_pct: 80,
        },
        PromptTemplate {
            id: "sentence/register",
            version: "v3",
            input_template: include_str!("eval_prompts/sentence_register.yaml.hbs"),
            grade_template: include_str!("eval_prompts/sentence_register.grade.yaml.hbs"),
            threshold_pct: 80,
        },
        PromptTemplate {
            id: "sentence/word-breakdown",
            version: "v2",
            input_template: include_str!("eval_prompts/sentence_word_breakdown.yaml.hbs"),
            grade_template: include_str!("eval_prompts/sentence_word_breakdown.grade.yaml.hbs"),
            threshold_pct: 75,
        },
        PromptTemplate {
            id: "sentence/word-breakdown-from-translation",
            version: "v3",
            input_template: include_str!(
                "eval_prompts/sentence_word_breakdown_from_translation.yaml.hbs"
            ),
            grade_template: include_str!(
                "eval_prompts/sentence_word_breakdown_from_translation.grade.yaml.hbs"
            ),
            threshold_pct: 75,
        },
        PromptTemplate {
            id: "sentence/full-enrichment",
            version: "v2",
            input_template: include_str!("eval_prompts/sentence_full_enrichment.yaml.hbs"),
            grade_template: include_str!("eval_prompts/sentence_full_enrichment.grade.yaml.hbs"),
            threshold_pct: 70,
        },
    ]
}

fn supported_prompt_list() -> String {
    prompt_templates()
        .iter()
        .map(|template| format!("  {}", template.id))
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_fields(fields: Option<&str>) -> Result<Vec<String>, EvalError> {
    let values = fields
        .map(|value| value.split(',').collect::<Vec<_>>())
        .unwrap_or_else(|| DEFAULT_FIELDS.to_vec());
    let fields = values
        .into_iter()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    if fields.is_empty() {
        return Err(EvalError::Input(
            "--fields must include at least one field.".to_string(),
        ));
    }
    Ok(fields)
}

fn selected_items(
    source: &SourceYaml,
    fields: &[String],
    max_items: Option<usize>,
) -> Result<Vec<Map<String, Value>>, EvalError> {
    let limit = max_items.unwrap_or(source.items.len());
    source
        .items
        .iter()
        .take(limit)
        .map(|item| select_fields(item, fields))
        .collect()
}

fn select_fields(
    item: &Map<String, Value>,
    fields: &[String],
) -> Result<Map<String, Value>, EvalError> {
    let item_id = item
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("<unknown>");
    let mut selected = Map::new();
    for field in fields {
        let value = item.get(field).ok_or_else(|| {
            EvalError::Input(format!("Field {field:?} is missing from item {item_id:?}."))
        })?;
        selected.insert(field.clone(), value.clone());
    }
    Ok(selected)
}

fn select_running_model(models: Vec<RunningModel>) -> Result<RunningModel, EvalError> {
    match models.as_slice() {
        [] => Err(EvalError::Input(
            "No Ollama model is currently running.\n\nRun\n  ollama run translategemma:12b"
                .to_string(),
        )),
        [model] => Ok(model.clone()),
        _ => Err(EvalError::Input(format!(
            "More than one Ollama model is running.\n\nRunning models:\n{}",
            models
                .iter()
                .map(|model| format!("  {}", model.name))
                .collect::<Vec<_>>()
                .join("\n")
        ))),
    }
}

fn render_template(template: &str, context: &Value) -> Result<String, EvalError> {
    let mut registry = Handlebars::new();
    registry.register_escape_fn(handlebars::no_escape);
    registry
        .register_template_string("prompt", template)
        .map_err(EvalError::TemplateRegistration)?;
    registry
        .render("prompt", context)
        .map_err(EvalError::Template)
}

fn resolve_input_path(root: &ProjectRoot, input: &str) -> PathBuf {
    let path = Path::new(input);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn resolve_run_path(root: &ProjectRoot, run: &str) -> PathBuf {
    let path = Path::new(run);
    if path.is_absolute() {
        path.to_path_buf()
    } else if run == "eval" || run.starts_with("eval/") {
        root.join(path)
    } else {
        root.join("eval").join(path)
    }
}

fn relative_to_root(root: &ProjectRoot, path: &Path) -> PathBuf {
    path.strip_prefix(root.path()).unwrap_or(path).to_path_buf()
}

fn prompt_scoped_run_id(prompt_id: &str, run_path: &Path) -> String {
    let run_id = run_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    format!("{prompt_id}/{run_id}")
}

fn write_file(path: &Path, content: &str) -> Result<(), EvalError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| EvalError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(path, content).map_err(|source| EvalError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), EvalError> {
    let content = serde_json::to_string_pretty(value)?;
    write_file(path, &format!("{content}\n"))
}

fn current_dir() -> Result<PathBuf, EvalError> {
    std::env::current_dir().map_err(|source| EvalError::Io {
        path: PathBuf::from("."),
        source,
    })
}

fn open_editor(path: &Path) -> Result<(), EvalError> {
    let editor = std::env::var("EDITOR")
        .map_err(|_| EvalError::Editor("$EDITOR is not set.".to_string()))?;
    let mut parts = editor.split_whitespace();
    let Some(command) = parts.next() else {
        return Err(EvalError::Editor("$EDITOR is not set.".to_string()));
    };
    let status = Command::new(command)
        .args(parts)
        .arg(path)
        .status()
        .map_err(|error| EvalError::Editor(format!("Could not open $EDITOR.\n\n{error}")))?;
    if !status.success() {
        return Err(EvalError::Editor(format!(
            "$EDITOR exited with status {status}."
        )));
    }
    Ok(())
}

fn extract_grade_response(packet: &str) -> Result<String, EvalError> {
    let Some((_, response)) = packet.split_once(GRADE_RESPONSE_MARKER) else {
        return Err(EvalError::Grade(format!(
            "Could not find grade response marker: {GRADE_RESPONSE_MARKER}"
        )));
    };
    let response = strip_optional_fence(response.trim());
    if response.trim().is_empty() {
        return Err(EvalError::Grade(
            "Grade response is empty. Paste YAML or JSON under the marker.".to_string(),
        ));
    }
    Ok(response.trim().to_string())
}

fn strip_optional_fence(response: &str) -> &str {
    let response = response.trim();
    let Some(rest) = response.strip_prefix("```") else {
        return response;
    };
    let Some(line_end) = rest.find('\n') else {
        return response;
    };
    let after_open = &rest[line_end + 1..];
    let Some(close) = after_open.rfind("```") else {
        return response;
    };
    let inside = after_open[..close].trim();
    if !inside.is_empty() {
        return inside;
    }
    let tail = after_open[close + 3..].trim();
    if tail.is_empty() {
        inside
    } else {
        tail
    }
}

fn parse_grade(response: &str) -> Result<Grade, EvalError> {
    if let Ok(grade) = serde_json::from_str::<Grade>(response) {
        validate_grade(&grade)?;
        return Ok(grade);
    }
    let grade: Grade = serde_yaml::from_str(response).map_err(|_| {
        EvalError::Grade("Could not parse grader response as YAML or JSON.".to_string())
    })?;
    validate_grade(&grade)?;
    Ok(grade)
}

fn validate_grade(grade: &Grade) -> Result<(), EvalError> {
    for (name, axis) in [
        ("accuracy", &grade.scores.accuracy),
        ("completeness", &grade.scores.completeness),
        ("format_compliance", &grade.scores.format_compliance),
        ("consistency", &grade.scores.consistency),
        ("confidence", &grade.scores.confidence),
    ] {
        if axis.max != 4 || !(1..=4).contains(&axis.score) {
            return Err(EvalError::Grade(format!(
                "Grade axis {name:?} must use score 1-4 with max 4."
            )));
        }
    }
    if grade.total.max != 20 || grade.total.score > 20 {
        return Err(EvalError::Grade(
            "Grade total must use score 0-20 with max 20.".to_string(),
        ));
    }
    if grade.verdict.trim().is_empty() {
        return Err(EvalError::Grade("Grade verdict is required.".to_string()));
    }
    Ok(())
}

fn render_summary(meta: &EvalMeta, grade: Option<&Grade>) -> String {
    let mut output = String::from("Eval Run\n\n");
    output.push_str("Prompt\n");
    output.push_str(&format!("  id        {}\n", meta.prompt_id));
    output.push_str(&format!(
        "  version   {}\n",
        meta.prompt_version.as_deref().unwrap_or("legacy")
    ));
    output.push_str(&format!(
        "  hash      {}\n",
        meta.prompt_fingerprint
            .as_deref()
            .map(short_fingerprint)
            .unwrap_or("missing")
    ));
    output.push_str(&format!("  input     {}\n", meta.input_path));
    output.push_str(&format!("  items     {}\n", meta.item_count));
    output.push_str(&format!("  fields    {}\n\n", meta.fields.join(",")));
    output.push_str("Model\n");
    output.push_str(&format!("  selected  {}\n", meta.model));
    output.push_str(&format!("  source    {}\n\n", meta.model_source));
    output.push_str("Timing\n");
    output.push_str(&format!(
        "  render    {}\n",
        format_ms(meta.timing_ms.render)
    ));
    output.push_str(&format!(
        "  model     {}\n",
        format_ms(meta.timing_ms.model)
    ));
    output.push_str(&format!(
        "  total     {}\n\n",
        format_ms(meta.timing_ms.total)
    ));
    output.push_str("Artifacts\n");
    output.push_str(&format!("  prompt    {}\n", meta.artifacts.prompt));
    output.push_str(&format!("  response  {}\n", meta.artifacts.response));
    output.push_str("  meta      meta.json\n\n");
    output.push_str("Grade\n");
    match grade {
        Some(grade) => {
            output.push_str(&format!("  verdict   {}\n", grade.verdict));
            output.push_str(&format!(
                "  score     {}/{} ({}%)\n",
                grade.total.score, grade.total.max, grade.total.pct
            ));
            output.push_str(&format!("  summary   {}\n", grade.summary));
            output.push_str("  details   grade.json\n");
        }
        None => output.push_str("  status    not graded\n"),
    }
    output
}

fn model_slug(model: &str) -> String {
    let mut slug = model
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    while slug.contains("__") {
        slug = slug.replace("__", "_");
    }
    slug.trim_matches('_').to_string()
}

fn timestamp_run_id() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0));
    format!("unix_{}", duration.as_millis())
}

fn timestamp_string() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0));
    format!("unix:{}", duration.as_secs())
}

fn elapsed_ms(duration: Duration) -> u128 {
    duration.as_millis()
}

fn format_ms(ms: u128) -> String {
    if ms >= 1000 {
        format!("{:.1}s", (ms as f64) / 1000.0)
    } else {
        format!("{ms}ms")
    }
}

#[cfg(test)]
mod tests {
    use super::{
        extract_grade_response, model_slug, parse_fields, parse_grade, render_template,
        resolve_run_path, selected_items, EvalModelClient,
    };
    use crate::ollama::{ModelOutput, RunningModel};
    use crate::project::ProjectRoot;
    use serde_json::{json, Map, Value};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug)]
    struct FakeClient {
        models: Vec<RunningModel>,
        response: String,
    }

    impl EvalModelClient for FakeClient {
        fn running_models(&self) -> Result<Vec<RunningModel>, super::EvalError> {
            Ok(self.models.clone())
        }

        fn generate(&self, _model: &str, _prompt: &str) -> Result<ModelOutput, super::EvalError> {
            Ok(ModelOutput {
                text: self.response.clone(),
            })
        }
    }

    #[test]
    fn defaults_fields() {
        assert_eq!(
            parse_fields(None).unwrap(),
            vec!["id", "hindi", "romanisation", "english"]
        );
    }

    #[test]
    fn renders_each_items_template() {
        let rendered = render_template(
            "{{#each items}}{{id}} {{hindi}}\n{{/each}}{{items_yaml}}",
            &json!({
                "items": [{"id": "0001", "hindi": "यहाँ"}],
                "items_yaml": "- id: '0001'\n  hindi: यहाँ\n"
            }),
        )
        .unwrap();

        assert!(rendered.contains("0001 यहाँ"));
        assert!(!rendered.contains("items_yaml"));
    }

    #[test]
    fn selects_fields_and_fails_on_missing() {
        let source = super::SourceYaml {
            title: None,
            subtitle: None,
            items: vec![Map::from_iter([
                ("id".to_string(), Value::String("0001".to_string())),
                ("hindi".to_string(), Value::String("यहाँ".to_string())),
            ])],
        };

        let selected =
            selected_items(&source, &["id".to_string(), "hindi".to_string()], Some(1)).unwrap();
        assert_eq!(selected.len(), 1);
        assert!(selected_items(&source, &["english".to_string()], Some(1))
            .unwrap_err()
            .to_string()
            .contains("Field \"english\" is missing"));
    }

    #[test]
    fn resolves_run_path_under_eval_when_needed() {
        let root_path = temp_path("eval-root");
        fs::create_dir_all(root_path.join("docs")).unwrap();
        fs::create_dir_all(root_path.join("input")).unwrap();
        fs::create_dir_all(root_path.join("output")).unwrap();
        fs::write(root_path.join("docs/DESIGN.md"), "").unwrap();
        fs::write(root_path.join("docs/ROADMAP.md"), "").unwrap();
        let root = ProjectRoot::discover_from(&root_path).unwrap();

        assert_eq!(
            resolve_run_path(&root, "sentence/register/run1"),
            root_path.join("eval/sentence/register/run1")
        );
        assert_eq!(
            resolve_run_path(&root, "eval/sentence/register/run1"),
            root_path.join("eval/sentence/register/run1")
        );
        fs::remove_dir_all(root_path).unwrap();
    }

    #[test]
    fn extracts_grade_response_from_marker_and_fence() {
        let response = extract_grade_response(
            "## Grading Prompt\nx\n\n## Paste Grader Response Below\n\n```yaml\nverdict: pass\n```\n",
        )
        .unwrap();

        assert_eq!(response, "verdict: pass");
    }

    #[test]
    fn extracts_grade_response_pasted_after_empty_fence() {
        let response = extract_grade_response(
            "## Paste Grader Response Below\n\n```yaml\n```\nverdict: pass\n",
        )
        .unwrap();

        assert_eq!(response, "verdict: pass");
    }

    #[test]
    fn parses_grade_from_yaml() {
        let grade = parse_grade(
            r#"
run_id: sentence/register/run1
grader: human
graded_at: unix:1
scores:
  accuracy: { score: 4, max: 4, note: "" }
  completeness: { score: 4, max: 4, note: "" }
  format_compliance: { score: 4, max: 4, note: "" }
  consistency: { score: 4, max: 4, note: "" }
  confidence: { score: 4, max: 4, note: "" }
total: { score: 20, max: 20, pct: 100 }
verdict: pass
item_flags: []
summary: Good.
"#,
        )
        .unwrap();

        assert_eq!(grade.total.score, 20);
    }

    #[test]
    fn writes_eval_run_artifacts_with_fake_model() {
        let root_path = temp_path("eval-run-root");
        fs::create_dir_all(root_path.join("docs")).unwrap();
        fs::create_dir_all(root_path.join("input/sentences")).unwrap();
        fs::create_dir_all(root_path.join("output")).unwrap();
        fs::write(root_path.join("docs/DESIGN.md"), "").unwrap();
        fs::write(root_path.join("docs/ROADMAP.md"), "").unwrap();
        fs::write(
            root_path.join("input/sentences/sample.yaml"),
            r#"
title: Sample
subtitle: One
items:
  - id: "0001"
    hindi: "यहाँ"
    romanisation: "yahā̃"
    english: "Here."
"#,
        )
        .unwrap();
        let root = ProjectRoot::discover_from(&root_path).unwrap();
        let report = super::run_with_client(
            &root,
            "input/sentences/sample.yaml",
            "sentence/register",
            None,
            Some(1),
            &FakeClient {
                models: vec![RunningModel {
                    name: "test-model:1b".to_string(),
                    digest: None,
                }],
                response: "model output".to_string(),
            },
        )
        .unwrap();

        assert!(root_path
            .join(&report.run_path)
            .join("prompt.txt")
            .is_file());
        assert!(root_path
            .join(&report.run_path)
            .join("response.txt")
            .is_file());
        assert!(root_path.join(&report.run_path).join("meta.json").is_file());
        assert!(!root_path.join("output/response.txt").exists());
        fs::remove_dir_all(root_path).unwrap();
    }

    #[test]
    fn imports_grade_response_from_file_without_editor() {
        let root_path = temp_path("eval-grade-root");
        fs::create_dir_all(root_path.join("docs")).unwrap();
        fs::create_dir_all(root_path.join("input")).unwrap();
        fs::create_dir_all(root_path.join("output")).unwrap();
        fs::create_dir_all(root_path.join("eval/sentence/register/run1")).unwrap();
        fs::write(root_path.join("docs/DESIGN.md"), "").unwrap();
        fs::write(root_path.join("docs/ROADMAP.md"), "").unwrap();
        let current_fingerprint =
            super::prompt_fingerprint(&super::prompt_template("sentence/register").unwrap());
        fs::write(
            root_path.join("eval/sentence/register/run1/meta.json"),
            format!(
                r#"{{
  "run_id": "sentence/register/run1",
  "prompt_id": "sentence/register",
  "prompt_version": "v2",
  "prompt_fingerprint": "{current_fingerprint}",
  "input_path": "input/sentences/sample.yaml",
  "fields": ["id", "hindi", "romanisation", "english"],
  "max_items": 1,
  "item_count": 1,
  "model": "ollama:test",
  "model_digest": null,
  "model_source": "Ollama /api/ps",
  "started_at": "unix:1",
  "finished_at": "unix:2",
  "timing_ms": {{ "render": 1, "model": 2, "total": 3 }},
  "artifacts": {{ "prompt": "prompt.txt", "response": "response.txt", "summary": "summary.txt" }}
}}"#,
            ),
        )
        .unwrap();
        fs::write(
            root_path.join("eval/sentence/register/run1/prompt.txt"),
            "task: sentence_register_detection\nitems: []",
        )
        .unwrap();
        fs::write(
            root_path.join("eval/sentence/register/run1/response.txt"),
            "results: []",
        )
        .unwrap();
        fs::write(
            root_path.join("grade.yaml"),
            r#"
run_id: sentence/register/run1
grader: test
graded_at: unix:3
scores:
  accuracy: { score: 4, max: 4, note: "" }
  completeness: { score: 4, max: 4, note: "" }
  format_compliance: { score: 4, max: 4, note: "" }
  consistency: { score: 4, max: 4, note: "" }
  confidence: { score: 4, max: 4, note: "" }
total: { score: 20, max: 20, pct: 100 }
verdict: pass
item_flags: []
summary: Imported grade.
"#,
        )
        .unwrap();
        let root = ProjectRoot::discover_from(&root_path).unwrap();

        super::grade_from(&root, "sentence/register/run1", Some("grade.yaml")).unwrap();

        assert!(root_path
            .join("eval/sentence/register/run1/grade_prompt.txt")
            .is_file());
        assert!(root_path
            .join("eval/sentence/register/run1/grade_packet.md")
            .is_file());
        assert!(root_path
            .join("eval/sentence/register/run1/grade_response.txt")
            .is_file());
        assert!(root_path
            .join("eval/sentence/register/run1/grade.json")
            .is_file());
        fs::remove_dir_all(root_path).unwrap();
    }

    #[test]
    fn renders_eval_report_with_source_rows_and_grade() {
        let root_path = temp_path("eval-report-root");
        fs::create_dir_all(root_path.join("docs")).unwrap();
        fs::create_dir_all(root_path.join("input/sentences")).unwrap();
        fs::create_dir_all(root_path.join("output")).unwrap();
        fs::create_dir_all(root_path.join("eval/sentence/register/run1")).unwrap();
        fs::write(root_path.join("docs/DESIGN.md"), "").unwrap();
        fs::write(root_path.join("docs/ROADMAP.md"), "").unwrap();
        fs::write(
            root_path.join("input/sentences/sample.yaml"),
            r#"
title: Sample
subtitle: One
items:
  - id: "0001"
    hindi: "यहाँ"
    romanisation: "yahā̃"
    english: "Here."
"#,
        )
        .unwrap();
        let current_fingerprint =
            super::prompt_fingerprint(&super::prompt_template("sentence/register").unwrap());
        fs::write(
            root_path.join("eval/sentence/register/run1/meta.json"),
            format!(
                r#"{{
  "run_id": "sentence/register/run1",
  "prompt_id": "sentence/register",
  "prompt_version": "v2",
  "prompt_fingerprint": "{current_fingerprint}",
  "input_path": "input/sentences/sample.yaml",
  "fields": ["id", "hindi", "romanisation", "english"],
  "max_items": 1,
  "item_count": 1,
  "model": "ollama:test-model:1b",
  "model_digest": null,
  "model_source": "Ollama /api/ps",
  "started_at": "unix:1",
  "finished_at": "unix:2",
  "timing_ms": {{ "render": 1, "model": 2100, "total": 2200 }},
  "artifacts": {{ "prompt": "prompt.txt", "response": "response.txt", "summary": "summary.txt" }}
}}"#,
            ),
        )
        .unwrap();
        fs::write(
            root_path.join("eval/sentence/register/run1/grade.json"),
            r#"{
  "run_id": "sentence/register/run1",
  "grader": "test",
  "graded_at": "unix:3",
  "scores": {
    "accuracy": { "score": 4, "max": 4, "note": "" },
    "completeness": { "score": 4, "max": 4, "note": "" },
    "format_compliance": { "score": 3, "max": 4, "note": "" },
    "consistency": { "score": 4, "max": 4, "note": "" },
    "confidence": { "score": 4, "max": 4, "note": "" }
  },
  "total": { "score": 19, "max": 20, "pct": 95 },
  "verdict": "pass",
  "item_flags": [],
  "summary": "Clean register result."
}"#,
        )
        .unwrap();
        let root = ProjectRoot::discover_from(&root_path).unwrap();

        let rendered = super::report_from(&root, false, true, super::EvalReportOutput::None, false)
            .unwrap()
            .render();

        assert!(rendered.contains("Hindi    यहाँ"));
        assert!(rendered.contains("Roman    yahā̃"));
        assert!(rendered.contains("English  Here."));
        assert!(rendered.contains("register"));
        assert!(rendered.contains("test-model:1b"));
        assert!(rendered.contains("95%"));
        assert!(rendered.contains("pass"));
        assert!(rendered.contains("Clean register result."));
        fs::remove_dir_all(root_path).unwrap();
    }

    #[test]
    fn creates_filesystem_safe_model_slug() {
        assert_eq!(model_slug("translategemma:12b"), "translategemma_12b");
    }

    fn temp_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{label}-{}-{nanos}", std::process::id()))
    }
}
