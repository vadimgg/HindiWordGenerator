use crate::ollama::{HttpOllamaClient, ModelClientError, ModelOutput, RunningModel};
use crate::project::{ProjectRoot, ProjectRootError};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
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
            "  hindi eval grade --run {}\n",
            prompt_scoped_run_id(&self.prompt_id, &self.run_path)
        ));
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalGradeReport {
    run_path: PathBuf,
    prompt_id: String,
}

impl EvalGradeReport {
    pub fn render(&self) -> String {
        let mut output = String::from("Eval Grade\n\n");
        output.push_str("Run\n");
        output.push_str(&format!("  folder     {}\n", self.run_path.display()));
        output.push_str(&format!("  prompt id  {}\n\n", self.prompt_id));
        output.push_str("Editor\n");
        output.push_str("  opened     grade_packet.md\n");
        output.push_str("  response   grade_response.txt\n\n");
        output.push_str("Result\n");
        output.push_str("  parsed     ok\n");
        output.push_str("  grade      grade.json\n\n");
        output.push_str("Next\n");
        output.push_str(&format!("  less {}/summary.txt\n", self.run_path.display()));
        output
    }
}

#[derive(Debug, Clone)]
struct PromptTemplate {
    id: &'static str,
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

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
struct GradeScores {
    accuracy: AxisScore,
    completeness: AxisScore,
    format_compliance: AxisScore,
    consistency: AxisScore,
    confidence: AxisScore,
}

#[derive(Debug, Deserialize, Serialize)]
struct AxisScore {
    score: u8,
    max: u8,
    note: String,
}

#[derive(Debug, Deserialize, Serialize)]
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

pub fn grade_from_current_dir(run: &str) -> Result<EvalGradeReport, EvalError> {
    let root = ProjectRoot::discover_from(current_dir()?)?;
    grade_from(&root, run)
}

pub fn grade_from(root: &ProjectRoot, run: &str) -> Result<EvalGradeReport, EvalError> {
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
    let response_path = run_path.join("response.txt");
    let response = fs::read_to_string(&response_path).map_err(|source| EvalError::Io {
        path: response_path,
        source,
    })?;
    let context = json!({
        "run_id": meta.run_id,
        "prompt_id": meta.prompt_id,
        "input_path": meta.input_path,
        "fields": meta.fields,
        "item_count": meta.item_count,
        "model": meta.model,
        "response": response,
        "threshold_pct": template.threshold_pct,
    });
    let grade_prompt = render_template(template.grade_template, &context)?;
    write_file(&run_path.join("grade_prompt.txt"), &grade_prompt)?;
    let packet =
        format!("## Grading Prompt\n\n{grade_prompt}\n\n{GRADE_RESPONSE_MARKER}\n\n```yaml\n```\n");
    let packet_path = run_path.join("grade_packet.md");
    write_file(&packet_path, &packet)?;
    eprintln!("opening grade_packet.md in $EDITOR");
    open_editor(&packet_path)?;
    let packet_after = fs::read_to_string(&packet_path).map_err(|source| EvalError::Io {
        path: packet_path.clone(),
        source,
    })?;
    let grade_response = extract_grade_response(&packet_after)?;
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
    })
}

fn prompt_template(prompt_id: &str) -> Result<PromptTemplate, EvalError> {
    prompt_templates()
        .into_iter()
        .find(|template| template.id == prompt_id)
        .ok_or_else(|| EvalError::UnknownPrompt(prompt_id.to_string()))
}

fn prompt_templates() -> Vec<PromptTemplate> {
    vec![
        PromptTemplate {
            id: "sentence/source-qa",
            input_template: include_str!("eval_prompts/sentence_source_qa.yaml.hbs"),
            grade_template: include_str!("eval_prompts/sentence_source_qa.grade.yaml.hbs"),
            threshold_pct: 80,
        },
        PromptTemplate {
            id: "sentence/english",
            input_template: include_str!("eval_prompts/sentence_english.yaml.hbs"),
            grade_template: include_str!("eval_prompts/sentence_english.grade.yaml.hbs"),
            threshold_pct: 80,
        },
        PromptTemplate {
            id: "sentence/literal",
            input_template: include_str!("eval_prompts/sentence_literal.yaml.hbs"),
            grade_template: include_str!("eval_prompts/sentence_literal.grade.yaml.hbs"),
            threshold_pct: 80,
        },
        PromptTemplate {
            id: "sentence/register",
            input_template: include_str!("eval_prompts/sentence_register.yaml.hbs"),
            grade_template: include_str!("eval_prompts/sentence_register.grade.yaml.hbs"),
            threshold_pct: 80,
        },
        PromptTemplate {
            id: "sentence/word-breakdown",
            input_template: include_str!("eval_prompts/sentence_word_breakdown.yaml.hbs"),
            grade_template: include_str!("eval_prompts/sentence_word_breakdown.grade.yaml.hbs"),
            threshold_pct: 75,
        },
        PromptTemplate {
            id: "sentence/word-breakdown-from-translation",
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
