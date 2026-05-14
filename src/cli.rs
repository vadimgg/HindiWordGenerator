#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Help,
    Doctor,
    DoctorHelp,
    SourceIdsHelp,
    SourceIdsCheck,
    SourceIdsMigrate { dry_run: bool },
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
        [command, subcommand, ..] if command == "source" && subcommand == "ids" => {
            Err(CliError::new("Usage: hindi source ids check | migrate [--check]"))
        }
        [command, ..] if command == "sentences" => Err(CliError::new(
            "`hindi sentences plan` is not available yet. M2 adds: hindi sentences plan --max-batches 1",
        )),
        [command, ..] => Err(CliError::new(format!("Unknown command: {command}"))),
    }
}

fn is_help_flag(value: &str) -> bool {
    value == "-h" || value == "--help"
}

pub fn help_text() -> &'static str {
    "Hindi Word Generator\n\nUsage:\n  hindi doctor\n  hindi source ids check\n  hindi source ids migrate [--check]\n\nCommands:\n  doctor       Check project paths, prompts, and Ollama reachability\n  source ids   Validate or migrate source YAML item IDs\n\nM2 adds:\n  hindi sentences plan --max-batches 1"
}

pub fn doctor_help_text() -> &'static str {
    "Hindi Word Generator\n\nUsage:\n  hindi doctor\n\nChecks:\n  project root\n  input/output/audio folders\n  sentence prompt files\n  optional hindi.toml\n  Ollama service reachability\n\nThis command is read-only and writes no learner data."
}

pub fn source_ids_help_text() -> &'static str {
    "Hindi Word Generator\n\nUsage:\n  hindi source ids check\n  hindi source ids migrate [--check]\n\nCommands:\n  check      Validate source IDs without writing files\n  migrate   Add missing source IDs\n\nOptions:\n  --check    Preview migration without writing files"
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
    fn sentences_plan_is_not_exposed_in_m1() {
        let error = parse(["sentences", "plan"]).unwrap_err().to_string();

        assert!(error.contains("not available yet"));
        assert!(error.contains("M2 adds"));
    }

    #[test]
    fn unknown_command_is_usage_error() {
        let error = parse(["generate"]).unwrap_err().to_string();

        assert!(error.contains("Unknown command: generate"));
        assert!(error.contains("Usage:"));
    }
}
