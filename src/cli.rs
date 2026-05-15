#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Help,
    Doctor,
    DoctorHelp,
    SourceIdsHelp,
    SourceIdsCheck,
    SourceIdsMigrate { dry_run: bool },
    SentencesHelp,
    SentencesPlan { max_batches: usize },
    SentencesGenerate { max_batches: usize },
    SentencesAudio,
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
        write!(formatter, "{}\n\n{}", self.message, help_text())
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
        [command, ..] => Err(CliError::new(format!("Unknown command: {command}"))),
    }
}

fn parse_positive_usize(value: &str) -> Option<usize> {
    let parsed = value.parse().ok()?;
    (parsed > 0).then_some(parsed)
}

fn is_help_flag(value: &str) -> bool {
    value == "-h" || value == "--help"
}

pub fn help_text() -> &'static str {
    "Hindi Word Generator\n\nUsage:\n  hindi doctor\n  hindi source ids check\n  hindi source ids migrate [--check]\n  hindi sentences plan --max-batches <n>\n  hindi sentences generate --max-batches <n>\n  hindi sentences audio\n\nCommands:\n  doctor       Check project paths, prompts, and Ollama reachability\n  source ids   Validate or migrate source YAML item IDs\n  sentences    Plan, generate, or backfill sentence batches"
}

pub fn doctor_help_text() -> &'static str {
    "Hindi Word Generator\n\nUsage:\n  hindi doctor\n\nChecks:\n  project root\n  input/output/audio folders\n  sentence prompt files\n  optional hindi.toml\n  Ollama service reachability\n\nThis command is read-only and writes no learner data."
}

pub fn source_ids_help_text() -> &'static str {
    "Hindi Word Generator\n\nUsage:\n  hindi source ids check\n  hindi source ids migrate [--check]\n\nCommands:\n  check      Validate source IDs without writing files\n  migrate   Add missing source IDs\n\nOptions:\n  --check    Preview migration without writing files"
}

pub fn sentences_help_text() -> &'static str {
    "Hindi Word Generator\n\nUsage:\n  hindi sentences plan --max-batches <n>\n  hindi sentences generate --max-batches <n>\n  hindi sentences audio\n\nCommands:\n  plan       Preview pending sentence batches without writing output\n  generate   Generate pending sentence batches with the configured local model\n  audio      Backfill missing audio for accepted sentence batches"
}

#[cfg(test)]
mod tests {
    use super::{parse, Command};

    #[test]
    fn exposes_doctor_command() {
        assert_eq!(parse(["doctor"]).unwrap(), Command::Doctor);
        assert_eq!(parse(["doctor", "--help"]).unwrap(), Command::DoctorHelp);
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
