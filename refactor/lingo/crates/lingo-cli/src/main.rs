#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod cli;
mod commands;
mod composition;
mod exit;
mod interaction;
mod output;
mod viewer_server;

use clap::{CommandFactory, FromArgMatches};
use cli::{Cli, ColorArg, Command};
use output::Output;
use std::ffi::OsString;
use std::io::IsTerminal;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = parse_cli();
    let color = color_enabled(&cli);
    let output = Output::new(color);
    let result = match cli.command {
        Command::Init(args) => commands::init::run(args, &output),
        Command::Import(args) => commands::import::run(args, &output),
        Command::Build(args) => commands::build::run(args, &output),
        Command::Check(args) => commands::check::run(args, &output),
        Command::Audio(args) => commands::audio::run(args, &output),
        Command::Package(args) => commands::package::run(args, &output),
        Command::Export(args) => commands::export::run(args, &output),
        Command::Status(args) => commands::status::run(args, &output),
        Command::Lang(args) => commands::lang::run(args, &output),
        Command::Doctor => commands::doctor::run(&output),
        Command::Viewer(args) => commands::viewer::run(args, &output),
    };
    match result {
        Ok(status) => status.code(),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn parse_cli() -> Cli {
    let color = clap_color_choice(std::env::args_os());
    let command = Cli::command().color(color).styles(cli::help_styles());
    let matches = command.get_matches();
    Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.exit())
}

fn clap_color_choice(args: impl IntoIterator<Item = OsString>) -> clap::ColorChoice {
    let mut saw_no_color = false;
    let mut selected = None;
    let mut iter = args.into_iter().skip(1);
    while let Some(arg) = iter.next() {
        let value = arg.to_string_lossy();
        if value == "--no-color" {
            saw_no_color = true;
        } else if value == "--color" {
            selected = iter.next().and_then(|next| color_choice_from_value(&next));
        } else if let Some(next) = value.strip_prefix("--color=") {
            selected = color_choice_from_str(next);
        }
    }
    if saw_no_color {
        clap::ColorChoice::Never
    } else {
        selected.unwrap_or(clap::ColorChoice::Auto)
    }
}

fn color_choice_from_value(value: &OsString) -> Option<clap::ColorChoice> {
    value.to_str().and_then(color_choice_from_str)
}

fn color_choice_from_str(value: &str) -> Option<clap::ColorChoice> {
    match value {
        "always" => Some(clap::ColorChoice::Always),
        "never" => Some(clap::ColorChoice::Never),
        "auto" => Some(clap::ColorChoice::Auto),
        _ => None,
    }
}

fn color_enabled(cli: &Cli) -> bool {
    if cli.no_color {
        return false;
    }
    match cli.color {
        ColorArg::Always => true,
        ColorArg::Never => false,
        ColorArg::Auto => {
            if std::env::var_os("NO_COLOR").is_some() {
                return false;
            }
            let force = std::env::var("CLICOLOR_FORCE").is_ok_and(|value| value != "0");
            let disabled = std::env::var("CLICOLOR").is_ok_and(|value| value == "0");
            force || (!disabled && std::io::stdout().is_terminal())
        }
    }
}
