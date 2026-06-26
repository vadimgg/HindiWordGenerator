use crate::cli::StatusArgs;
use crate::commands::{CommandResult, current_dir, parse_batch};
use crate::composition::Composition;
use crate::exit::ExitStatus;
use crate::output::Output;
use lingo_application::{NextAction, StatusDeps, StatusRequest, status};

pub fn run(args: StatusArgs, output: &Output) -> CommandResult {
    let composition = Composition::discover(&current_dir()?)?;
    let report = status(
        &StatusDeps {
            workspace: &composition.workspace,
            context: &composition.workspace,
        },
        StatusRequest {
            batch: parse_batch(args.batch.as_deref())?,
            problems_only: args.problems,
        },
    )?;
    let complete = matches!(
        &report.next,
        NextAction::Package | NextAction::Export | NextAction::None
    ) && report.pending_raw.is_empty()
        && report.rows.iter().all(|row| {
            row.source_present
                && row.cards_present
                && row.audio.is_complete()
                && row.problem.is_none()
        });
    output.status(&report);
    Ok(if complete {
        ExitStatus::Success
    } else {
        ExitStatus::ActionRequired
    })
}
