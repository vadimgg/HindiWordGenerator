# `crates/lingo-cli/src/cli.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Defines the top-level Clap grammar and delegates each command to its owned command module.

## Scope: this file owns

- global flags
- command enum
- argument vocabulary
- help metadata

## Out of scope: this file must not own

- manual token parsing
- workflow logic
- long handwritten help strings

## Allowed dependencies

- clap
- commands module argument types

## Forbidden dependencies and shortcuts

- application services directly

## Key implementation shape

```rust
#[derive(clap::Parser)]
#[command(name = "lingo", version, about)]
pub struct Cli {
    #[arg(long, global = true)]
    pub no_color: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand)]
pub enum Command {
    Init(commands::init::Args),
    Import(commands::import::Args),
    Build(commands::build::Args),
    Check(commands::check::Args),
    Audio(commands::audio::Args),
    Package(commands::package::Args),
    Export(commands::export::Args),
    Status(commands::status::Args),
    Lang(commands::lang::Args),
    Doctor(commands::doctor::Args),
    Viewer(commands::viewer::Args),
}
```

## Required tests / evidence

- command help snapshots
- unknown command/usage exit 2
- no legacy `hindi` or `sentences <verb>` aliases

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
