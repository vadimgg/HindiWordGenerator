#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Help,
    Doctor,
    DoctorHelp,
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
    "Hindi Word Generator\n\nUsage:\n  hindi doctor\n\nCommands:\n  doctor    Check project paths, prompts, and Ollama reachability\n\nM2 adds:\n  hindi sentences plan --max-batches 1"
}

pub fn doctor_help_text() -> &'static str {
    "Hindi Word Generator\n\nUsage:\n  hindi doctor\n\nChecks:\n  project root\n  input/output/audio folders\n  sentence prompt files\n  optional hindi.toml\n  Ollama service reachability\n\nThis command is read-only and writes no learner data."
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
