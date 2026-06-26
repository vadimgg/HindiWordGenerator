# `crates/lingo-cli/src/commands/status.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../../ARCHITECTURE.md)

## Responsibility

Defines the `status` command arguments, maps them into typed application requests, invokes the owning use case, and renders its typed report.

## Scope: this file owns

- `status` Clap arguments
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
- lingo-application `status` API
- output/interaction helpers as needed

## Forbidden dependencies and shortcuts

- direct adapter internals
- legacy aliases
- duplicated next-action logic

## Key implementation shape

```rust
#[derive(clap::Args)]
pub struct Args { pub batch: Option<String>, pub problems: bool }

    pub fn run(ctx: &AppContext, args: Args, out: &mut dyn Write) -> Result<CommandStatus, CliError> {
        let report = lingo_application::status(&ctx.status_deps(), args.into_request()?)?;
        render_status(&report, out)?; Ok(report.status())
    }
```

## Required tests / evidence

- problem filter mapping
- next command rendering
- read-only wiring

## Design notes

- The renderer never recomputes next action.

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
