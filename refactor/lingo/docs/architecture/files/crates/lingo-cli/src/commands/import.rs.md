# `crates/lingo-cli/src/commands/import.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../../ARCHITECTURE.md)

## Responsibility

Defines the `import` command arguments, maps them into typed application requests, invokes the owning use case, and renders its typed report.

## Scope: this file owns

- `import` Clap arguments
- boundary parsing into typed values
- one use-case call
- report rendering call

## Out of scope: this file must not own

- workflow policy
- filesystem/provider mechanics
- domain validation
- cross-command utility dumping ground

## Allowed dependencies

- AppContext
- lingo-application `import` API
- output/interaction helpers as needed

## Forbidden dependencies and shortcuts

- direct adapter internals
- legacy aliases
- duplicated next-action logic

## Key implementation shape

```rust
#[derive(clap::Args)]
pub struct Args { pub raw_file: Option<PathBuf>, pub title: Option<String>, pub subtitle: Option<String>, pub apply: Option<PathBuf>, pub print: bool }

    pub fn run(ctx: &AppContext, args: Args, out: &mut dyn Write) -> Result<CommandStatus, CliError> {
        let prepared = prepare_import(&ctx.import_deps(), args.prepare_request())?;
        run_packet_command(prepared, args.prompt_mode()?, |reply| apply_import(&ctx.import_deps(), reply), out)
    }
```

## Required tests / evidence

- `--print` conflicts with `--apply`
- interactive mode calls packet loop once
- render accepted report

## Design notes

- Convert `print/apply/default` into `PromptMode`; do not pass booleans inward.

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
