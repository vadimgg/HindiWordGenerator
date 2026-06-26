use crate::cli::{DisplayLeadArg, ViewerArgs};
use crate::commands::{CommandResult, command_error, current_dir, parse_batch};
use crate::composition::Composition;
use crate::exit::ExitStatus;
use crate::output::Output;
use crate::viewer_server::serve;
use lingo_application::{ViewerDeps, ViewerRequest, prepare_viewer};
use lingo_domain::DisplayLead;
use std::path::{Path, PathBuf};
use std::process::Command;

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
    let viewer = viewer_app_dir();
    build_viewer(&viewer, composition.workspace.root().path())?;
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
        viewer.join("dist"),
        plan,
    )?;
    Ok(ExitStatus::Success)
}

fn viewer_app_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("apps")
        .join("viewer")
}

fn build_viewer(viewer: &Path, workspace: &Path) -> CommandResult {
    if !viewer.join("package.json").is_file() {
        return Err(command_error(format!(
            "viewer app is missing at {}",
            viewer.display()
        )));
    }
    let status = Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir(viewer)
        .env("LINGO_WORKSPACE_ROOT", workspace)
        .status()?;
    if status.success() {
        Ok(ExitStatus::Success)
    } else {
        Err(command_error(
            "viewer build failed; run `(cd refactor/lingo/apps/viewer && npm install)` once, then retry",
        ))
    }
}
