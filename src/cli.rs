#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Help,
    Doctor,
    DoctorHelp,
    Viewer,
    ViewerHelp,
    ExportHelp,
    Export {
        source: String,
        topic: String,
    },
    SourceIdsHelp,
    SourceIdsCheck,
    SourceIdsMigrate {
        dry_run: bool,
    },
    SentencesHelp,
    SentencesPlan {
        max_batches: usize,
    },
    SentencesGenerate {
        max_batches: usize,
    },
    SentencesAudio,
    EvalHelp,
    EvalRun {
        input: String,
        prompt_id: String,
        fields: Option<String>,
        max_items: Option<usize>,
    },
    EvalGrade {
        run: String,
        response: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliError {
    message: String,
}

impl CliError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.message.contains("Usage:") {
            write!(formatter, "{}", self.message)
        } else {
            write!(formatter, "{}\n\n{}", self.message, help_text())
        }
    }
}

pub fn parse<I, S>(args: I) -> Result<Command, CliError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();

    match args.as_slice() {
        [] => Ok(Command::Help),
        [flag] if is_help_flag(flag) => Ok(Command::Help),
        [command] if command == "doctor" => Ok(Command::Doctor),
        [command, flag] if command == "doctor" && is_help_flag(flag) => Ok(Command::DoctorHelp),
        [command] if command == "viewer" => Ok(Command::Viewer),
        [command, flag] if command == "viewer" && is_help_flag(flag) => Ok(Command::ViewerHelp),
        [command, flag] if command == "export" && is_help_flag(flag) => Ok(Command::ExportHelp),
        [command, source_flag, source, topic_flag, topic]
            if command == "export" && source_flag == "--source" && topic_flag == "--topic" =>
        {
            Ok(Command::Export {
                source: source.clone(),
                topic: topic.clone(),
            })
        }
        [command, topic_flag, topic, source_flag, source]
            if command == "export" && source_flag == "--source" && topic_flag == "--topic" =>
        {
            Ok(Command::Export {
                source: source.clone(),
                topic: topic.clone(),
            })
        }
        [command, ..] if command == "export" => Err(CliError::new(
            "Usage: hindi export --source <title> --topic <subtitle>",
        )),
        [command, subcommand, flag]
            if command == "source" && subcommand == "ids" && is_help_flag(flag) =>
        {
            Ok(Command::SourceIdsHelp)
        }
        [command, subcommand, action]
            if command == "source" && subcommand == "ids" && action == "check" =>
        {
            Ok(Command::SourceIdsCheck)
        }
        [command, subcommand, action]
            if command == "source" && subcommand == "ids" && action == "migrate" =>
        {
            Ok(Command::SourceIdsMigrate { dry_run: false })
        }
        [command, subcommand, action, flag]
            if command == "source"
                && subcommand == "ids"
                && action == "migrate"
                && flag == "--check" =>
        {
            Ok(Command::SourceIdsMigrate { dry_run: true })
        }
        [command, subcommand, ..] if command == "source" && subcommand == "ids" => Err(
            CliError::new("Usage: hindi source ids check | migrate [--check]"),
        ),
        [command, flag] if command == "sentences" && is_help_flag(flag) => {
            Ok(Command::SentencesHelp)
        }
        [command, action, flag, value]
            if command == "sentences" && action == "plan" && flag == "--max-batches" =>
        {
            parse_positive_usize(value)
                .map(|max_batches| Command::SentencesPlan { max_batches })
                .ok_or_else(|| CliError::new("--max-batches must be a positive integer."))
        }
        [command, action, flag, value]
            if command == "sentences" && action == "generate" && flag == "--max-batches" =>
        {
            parse_positive_usize(value)
                .map(|max_batches| Command::SentencesGenerate { max_batches })
                .ok_or_else(|| CliError::new("--max-batches must be a positive integer."))
        }
        [command, action] if command == "sentences" && action == "plan" => {
            Err(CliError::new("Missing required option: --max-batches <n>"))
        }
        [command, action] if command == "sentences" && action == "generate" => {
            Err(CliError::new("Missing required option: --max-batches <n>"))
        }
        [command, action] if command == "sentences" && action == "audio" => {
            Ok(Command::SentencesAudio)
        }
        [command, ..] if command == "sentences" => Err(CliError::new(
            "Usage: hindi sentences plan --max-batches <n> | generate --max-batches <n> | audio",
        )),
        [command, flag] if command == "eval" && is_help_flag(flag) => Ok(Command::EvalHelp),
        [command, action, ..] if command == "eval" && action == "run" => parse_eval_run(&args[2..]),
        [command, action, ..] if command == "eval" && action == "grade" => {
            parse_eval_grade(&args[2..])
        }
        [command, ..] if command == "eval" => Err(CliError::new(eval_usage_error())),
        [command, ..] => Err(CliError::new(format!("Unknown command: {command}"))),
    }
}

fn parse_eval_grade(args: &[String]) -> Result<Command, CliError> {
    let mut run = None;
    let mut response = None;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--run" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(CliError::new("Missing value for --run."));
                };
                run = Some(value.clone());
            }
            "--response" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(CliError::new("Missing value for --response."));
                };
                response = Some(value.clone());
            }
            value if value.starts_with('-') => {
                return Err(CliError::new(format!(
                    "Unknown eval grade option: {value}\n\n{}",
                    eval_grade_usage_error()
                )));
            }
            value => {
                if run.is_some() {
                    return Err(CliError::new(format!(
                        "Unexpected extra eval run id: {value}\n\n{}",
                        eval_grade_usage_error()
                    )));
                }
                run = Some(value.to_string());
            }
        }
        index += 1;
    }
    Ok(Command::EvalGrade {
        run: run.ok_or_else(|| CliError::new(eval_grade_usage_error()))?,
        response,
    })
}

fn parse_eval_run(args: &[String]) -> Result<Command, CliError> {
    let mut input = None;
    let mut prompt_id = None;
    let mut fields = None;
    let mut max_items = None;
    let mut positional = Vec::new();
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--input" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(CliError::new("Missing value for --input."));
                };
                input = Some(value.clone());
            }
            "--prompt-id" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(CliError::new("Missing value for --prompt-id."));
                };
                prompt_id = Some(value.clone());
            }
            "--fields" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(CliError::new("Missing value for --fields."));
                };
                fields = Some(value.clone());
            }
            "--max-items" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(CliError::new("Missing value for --max-items."));
                };
                max_items = Some(
                    parse_positive_usize(value)
                        .ok_or_else(|| CliError::new("--max-items must be a positive integer."))?,
                );
            }
            value if value.starts_with('-') => {
                return Err(CliError::new(format!(
                    "Unknown eval run option: {value}\n\n{}",
                    eval_run_usage_error()
                )));
            }
            value => {
                positional.push(value.to_string());
            }
        }
        index += 1;
    }
    match positional.as_slice() {
        [] => {}
        [prompt] => {
            if prompt_id.is_some() {
                return Err(CliError::new(format!(
                    "Prompt id was provided twice.\n\n{}",
                    eval_run_usage_error()
                )));
            }
            prompt_id = Some(prompt.clone());
        }
        [prompt, path] => {
            if prompt_id.is_some() || input.is_some() {
                return Err(CliError::new(format!(
                    "Eval run input was provided twice.\n\n{}",
                    eval_run_usage_error()
                )));
            }
            prompt_id = Some(prompt.clone());
            input = Some(path.clone());
        }
        _ => {
            return Err(CliError::new(format!(
                "Too many eval run arguments.\n\n{}",
                eval_run_usage_error()
            )));
        }
    }
    Ok(Command::EvalRun {
        input: input.ok_or_else(|| CliError::new(eval_run_usage_error()))?,
        prompt_id: prompt_id.ok_or_else(|| CliError::new(eval_run_usage_error()))?,
        fields,
        max_items,
    })
}

fn parse_positive_usize(value: &str) -> Option<usize> {
    let parsed = value.parse().ok()?;
    (parsed > 0).then_some(parsed)
}

fn is_help_flag(value: &str) -> bool {
    value == "-h" || value == "--help"
}

pub fn help_text() -> &'static str {
    "Hindi Word Generator\n\nUsage:\n  hindi doctor\n  hindi source ids check\n  hindi source ids migrate [--check]\n  hindi sentences plan --max-batches <n>\n  hindi sentences generate --max-batches <n>\n  hindi sentences audio\n  hindi eval run <prompt-id> <input-yaml> [--fields <list>] [--max-items <n>]\n  hindi eval grade <run-id-or-path> [--response <path>]\n  hindi viewer\n  hindi export --source <title> --topic <subtitle>\n\nCommands:\n  doctor       Check project paths, prompts, and Ollama reachability\n  source ids   Validate or migrate source YAML item IDs\n  sentences    Plan, generate, or backfill sentence batches\n  eval         Run and grade prompt experiments under eval/\n  viewer       Serve the Astro preview/export app\n  export       Write a source/topic Anki import artifact"
}

pub fn doctor_help_text() -> &'static str {
    "Hindi Word Generator\n\nUsage:\n  hindi doctor\n\nChecks:\n  project root\n  input/output/audio folders\n  sentence prompt files\n  optional hindi.toml\n  Ollama service reachability\n\nThis command is read-only and writes no learner data."
}

pub fn viewer_help_text() -> &'static str {
    "Hindi Word Generator\n\nUsage:\n  hindi viewer\n\nServes the Astro viewer from viewer/ and prints the local URL."
}

pub fn export_help_text() -> &'static str {
    "Hindi Word Generator\n\nUsage:\n  hindi export --source <title> --topic <subtitle>\n\nOptions:\n  --source   Match accepted sentence batch title\n  --topic    Match accepted sentence batch subtitle"
}

pub fn source_ids_help_text() -> &'static str {
    "Hindi Word Generator\n\nUsage:\n  hindi source ids check\n  hindi source ids migrate [--check]\n\nCommands:\n  check      Validate source IDs without writing files\n  migrate   Add missing source IDs\n\nOptions:\n  --check    Preview migration without writing files"
}

pub fn sentences_help_text() -> &'static str {
    "Hindi Word Generator\n\nUsage:\n  hindi sentences plan --max-batches <n>\n  hindi sentences generate --max-batches <n>\n  hindi sentences audio\n\nCommands:\n  plan       Preview pending sentence batches without writing output\n  generate   Generate pending sentence batches with the configured local model\n  audio      Backfill missing audio for accepted sentence batches"
}

pub fn eval_help_text() -> &'static str {
    "Hindi Word Generator\n\nUsage:\n  hindi eval run <prompt-id> <input-yaml> [--fields <list>] [--max-items <n>]\n  hindi eval grade <run-id-or-path> [--response <path>]\n\nExamples:\n  hindi eval run sentence/register input/sentences/complete_hindi_chapter_02_sentences.yaml --max-items 2\n  hindi eval grade sentence/register/unix_1778842644180_translategemma_12b\n  hindi eval grade sentence/register/unix_1778842644180_translategemma_12b --response /tmp/grade.yaml\n\nRuns built-in prompt templates against YAML input using the one currently running Ollama model. Writes diagnostics to eval/<prompt-category>/<prompt-name>/<run-id>/ and never writes accepted output.\n\nArguments:\n  <prompt-id>          Built-in prompt id, e.g. sentence/register\n  <input-yaml>         YAML source file\n  <run-id-or-path>     Eval run folder or prompt-scoped run id to grade\n\nOptions:\n  --fields <list>      Comma-separated top-level item fields\n                       Default: id,hindi,romanisation,english\n  --max-items <n>      Limit selected items before rendering\n  --response <path>    Optional YAML/JSON grader response file to import instead of opening $EDITOR\n\nCompatibility aliases:\n  hindi eval run --prompt-id <id> --input <path>\n  hindi eval grade --run <run-id-or-path>"
}

fn eval_usage_error() -> &'static str {
    "Usage:\n  hindi eval run <prompt-id> <input-yaml> [--fields <list>] [--max-items <n>]\n  hindi eval grade <run-id-or-path> [--response <path>]\n\nExamples:\n  hindi eval run sentence/register input/sentences/complete_hindi_chapter_02_sentences.yaml --max-items 2\n  hindi eval grade sentence/register/unix_1778842644180_translategemma_12b"
}

fn eval_run_usage_error() -> &'static str {
    "Missing eval run arguments.\n\nUsage:\n  hindi eval run <prompt-id> <input-yaml> [--fields <list>] [--max-items <n>]\n\nExample:\n  hindi eval run sentence/register input/sentences/complete_hindi_chapter_02_sentences.yaml --max-items 2\n\nCompatibility alias:\n  hindi eval run --prompt-id sentence/register --input input/sentences/complete_hindi_chapter_02_sentences.yaml"
}

fn eval_grade_usage_error() -> &'static str {
    "Missing eval grade run id.\n\nUsage:\n  hindi eval grade <run-id-or-path> [--response <path>]\n\nExamples:\n  hindi eval grade sentence/register/unix_1778842644180_translategemma_12b\n  hindi eval grade sentence/register/unix_1778842644180_translategemma_12b --response /tmp/grade.yaml\n\nCompatibility alias:\n  hindi eval grade --run sentence/register/unix_1778842644180_translategemma_12b"
}

#[cfg(test)]
mod tests {
    use super::{parse, Command};

    #[test]
    fn exposes_doctor_command() {
        assert_eq!(parse(["doctor"]).unwrap(), Command::Doctor);
        assert_eq!(parse(["doctor", "--help"]).unwrap(), Command::DoctorHelp);
        assert_eq!(parse(["viewer"]).unwrap(), Command::Viewer);
        assert_eq!(parse(["viewer", "--help"]).unwrap(), Command::ViewerHelp);
    }

    #[test]
    fn exposes_source_ids_commands() {
        assert_eq!(
            parse(["source", "ids", "--help"]).unwrap(),
            Command::SourceIdsHelp
        );
        assert_eq!(
            parse(["source", "ids", "check"]).unwrap(),
            Command::SourceIdsCheck
        );
        assert_eq!(
            parse(["source", "ids", "migrate"]).unwrap(),
            Command::SourceIdsMigrate { dry_run: false }
        );
        assert_eq!(
            parse(["source", "ids", "migrate", "--check"]).unwrap(),
            Command::SourceIdsMigrate { dry_run: true }
        );
    }

    #[test]
    fn exposes_sentences_plan_command() {
        assert_eq!(
            parse(["sentences", "--help"]).unwrap(),
            Command::SentencesHelp
        );
        assert_eq!(
            parse(["sentences", "plan", "--max-batches", "2"]).unwrap(),
            Command::SentencesPlan { max_batches: 2 }
        );
        assert_eq!(
            parse(["sentences", "generate", "--max-batches", "2"]).unwrap(),
            Command::SentencesGenerate { max_batches: 2 }
        );
        assert_eq!(
            parse(["sentences", "audio"]).unwrap(),
            Command::SentencesAudio
        );
    }

    #[test]
    fn exposes_eval_commands() {
        assert_eq!(parse(["eval", "--help"]).unwrap(), Command::EvalHelp);
        assert_eq!(
            parse([
                "eval",
                "run",
                "sentence/register",
                "input/sentences/sample.yaml"
            ])
            .unwrap(),
            Command::EvalRun {
                input: "input/sentences/sample.yaml".to_string(),
                prompt_id: "sentence/register".to_string(),
                fields: None,
                max_items: None,
            }
        );
        assert_eq!(
            parse([
                "eval",
                "run",
                "sentence/register",
                "input/sentences/sample.yaml",
                "--fields",
                "id,hindi",
                "--max-items",
                "2"
            ])
            .unwrap(),
            Command::EvalRun {
                input: "input/sentences/sample.yaml".to_string(),
                prompt_id: "sentence/register".to_string(),
                fields: Some("id,hindi".to_string()),
                max_items: Some(2),
            }
        );
        assert_eq!(
            parse(["eval", "grade", "sentence/register/run1"]).unwrap(),
            Command::EvalGrade {
                run: "sentence/register/run1".to_string(),
                response: None,
            }
        );
        assert_eq!(
            parse([
                "eval",
                "grade",
                "sentence/register/run1",
                "--response",
                "/tmp/grade.yaml"
            ])
            .unwrap(),
            Command::EvalGrade {
                run: "sentence/register/run1".to_string(),
                response: Some("/tmp/grade.yaml".to_string()),
            }
        );
    }

    #[test]
    fn eval_named_flags_still_work_as_aliases() {
        assert_eq!(
            parse([
                "eval",
                "run",
                "--input",
                "input/sentences/sample.yaml",
                "--prompt-id",
                "sentence/register"
            ])
            .unwrap(),
            Command::EvalRun {
                input: "input/sentences/sample.yaml".to_string(),
                prompt_id: "sentence/register".to_string(),
                fields: None,
                max_items: None,
            }
        );
        assert_eq!(
            parse(["eval", "grade", "--run", "sentence/register/run1"]).unwrap(),
            Command::EvalGrade {
                run: "sentence/register/run1".to_string(),
                response: None,
            }
        );
    }

    #[test]
    fn eval_run_requires_input_and_prompt_id() {
        let error = parse(["eval", "run"]).unwrap_err().to_string();
        assert!(error.contains("hindi eval run <prompt-id> <input-yaml>"));

        let error = parse(["eval", "run", "sentence/register"])
            .unwrap_err()
            .to_string();
        assert!(error.contains("<input-yaml>"));

        let error = parse(["eval", "run", "p", "x.yaml", "--max-items", "0"])
            .unwrap_err()
            .to_string();
        assert!(error.contains("positive integer"));
    }

    #[test]
    fn eval_grade_requires_run_id() {
        let error = parse(["eval", "grade"]).unwrap_err().to_string();

        assert!(error.contains("hindi eval grade <run-id-or-path>"));
    }

    #[test]
    fn exposes_export_command() {
        assert_eq!(
            parse([
                "export",
                "--source",
                "Complete Hindi",
                "--topic",
                "Chapter 02"
            ])
            .unwrap(),
            Command::Export {
                source: "Complete Hindi".to_string(),
                topic: "Chapter 02".to_string()
            }
        );
        assert_eq!(parse(["export", "--help"]).unwrap(), Command::ExportHelp);
        assert!(parse(["export"])
            .unwrap_err()
            .to_string()
            .contains("Usage:"));
    }

    #[test]
    fn sentences_plan_requires_positive_max_batches() {
        let missing = parse(["sentences", "plan"]).unwrap_err().to_string();
        assert!(missing.contains("Missing required option"));

        let zero = parse(["sentences", "plan", "--max-batches", "0"])
            .unwrap_err()
            .to_string();
        assert!(zero.contains("positive integer"));
    }

    #[test]
    fn unknown_command_is_usage_error() {
        let error = parse(["generate"]).unwrap_err().to_string();

        assert!(error.contains("Unknown command: generate"));
        assert!(error.contains("Usage:"));
    }
}
