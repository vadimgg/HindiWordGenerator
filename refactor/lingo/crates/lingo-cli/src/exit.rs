use std::process::ExitCode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitStatus {
    Success,
    ActionRequired,
}

impl ExitStatus {
    pub fn code(self) -> ExitCode {
        match self {
            Self::Success => ExitCode::SUCCESS,
            Self::ActionRequired => ExitCode::from(1),
        }
    }
}
