use crate::commands::{CommandResult, current_dir};
use crate::composition::Composition;
use crate::exit::ExitStatus;
use crate::output::Output;
use lingo_application::{DoctorDeps, doctor};

pub fn run(output: &Output) -> CommandResult {
    let composition = Composition::discover(&current_dir()?)?;
    let report = doctor(&DoctorDeps {
        context: &composition.workspace,
        environment: &composition.environment,
    })?;
    output.doctor(&report);
    Ok(if report.required_checks_passed() {
        ExitStatus::Success
    } else {
        ExitStatus::ActionRequired
    })
}
