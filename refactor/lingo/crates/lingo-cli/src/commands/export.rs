use crate::cli::ExportArgs;
use crate::commands::{CommandResult, current_dir};
use crate::composition::Composition;
use crate::exit::ExitStatus;
use crate::output::Output;
use lingo_application::{ExportDeps, ExportRequest, export};
use lingo_domain::BatchId;

pub fn run(args: ExportArgs, output: &Output) -> CommandResult {
    let composition = Composition::discover(&current_dir()?)?;
    let batches = args
        .batch
        .into_iter()
        .map(BatchId::parse)
        .collect::<Result<Vec<_>, _>>()?;
    let report = export(
        &ExportDeps {
            workspace: &composition.workspace,
            context: &composition.workspace,
            exporter: &composition.anki,
        },
        ExportRequest {
            batches,
            all: args.all,
            deck: args.deck,
            destination: args.destination,
        },
    )?;
    output.exported(&report);
    Ok(ExitStatus::Success)
}
