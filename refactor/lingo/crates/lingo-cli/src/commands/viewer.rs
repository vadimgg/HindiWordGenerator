use crate::cli::{DisplayLeadArg, ViewerArgs};
use crate::commands::{CommandResult, current_dir, parse_batch};
use crate::composition::Composition;
use crate::exit::ExitStatus;
use crate::output::Output;
use crate::viewer_server::serve;
use lingo_application::{ViewerDeps, ViewerRequest, prepare_viewer};
use lingo_domain::DisplayLead;

pub fn run(args: ViewerArgs, output: &Output) -> CommandResult {
    let composition = Composition::discover(&current_dir()?)?;
    let plan = prepare_viewer(
        &ViewerDeps {
            workspace: &composition.workspace,
            context: &composition.workspace,
        },
        ViewerRequest {
            batch: parse_batch(args.batch.as_deref())?,
            lead: args.lead.map(|lead| match lead {
                DisplayLeadArg::Romanisation => DisplayLead::Romanisation,
                DisplayLeadArg::Target => DisplayLead::Target,
            }),
        },
    )?;
    let url = format!("http://127.0.0.1:{}", args.port);
    println!(
        "Viewer  {} card(s)\n\n  url  {url}\n\nPress Ctrl-C to stop.",
        plan.cards.len()
    );
    if !args.no_open {
        if let Err(error) = open::that(&url) {
            output.note(&format!("browser was not opened automatically: {error}"));
        }
    }
    serve(
        args.port,
        composition.workspace.root().path().to_path_buf(),
        plan,
    )?;
    Ok(ExitStatus::Success)
}
