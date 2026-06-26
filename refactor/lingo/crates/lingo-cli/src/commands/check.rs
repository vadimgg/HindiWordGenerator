use crate::cli::CheckArgs;
use crate::commands::{CommandResult, current_dir, parse_batch};
use crate::composition::Composition;
use crate::exit::ExitStatus;
use crate::output::Output;
use lingo_application::{CheckDeps, CheckRequest, check};

pub fn run(args: CheckArgs, output: &Output) -> CommandResult {
    let composition = Composition::discover(&current_dir()?)?;
    let report = check(
        &CheckDeps {
            workspace: &composition.workspace,
            context: &composition.workspace,
        },
        CheckRequest {
            batch: parse_batch(args.batch.as_deref())?,
        },
    )?;
    output.checked(&report);
    Ok(if report.is_clean() {
        ExitStatus::Success
    } else {
        ExitStatus::ActionRequired
    })
}
