use anyhow::Result;
use clap::{Parser, Subcommand};

/// @intent Parse CLI arguments and dispatch to focused command handlers.
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Print { message: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    run(cli)
}

/// @intent Keep top-level dispatch thin so command behavior can move into modules.
fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Print { message } => {
            println!("{message}");
            Ok(())
        }
    }
}
