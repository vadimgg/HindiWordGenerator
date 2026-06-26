use crate::cli::PackageArgs;
use crate::commands::{CommandResult, current_dir, parse_batch};
use crate::composition::Composition;
use crate::exit::ExitStatus;
use crate::output::Output;
use lingo_application::{PackageDeps, PackageRequest, package};

pub fn run(args: PackageArgs, output: &Output) -> CommandResult {
    let composition = Composition::discover(&current_dir()?)?;
    let report = package(
        &PackageDeps {
            workspace: &composition.workspace,
            context: &composition.workspace,
            publisher: &composition.package,
        },
        PackageRequest {
            batch: parse_batch(args.batch.as_deref())?,
            destination: args.destination,
        },
    )?;
    output.packaged(&report);
    Ok(ExitStatus::Success)
}
