# `crates/lingo-cli/src/exit.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Maps typed command outcomes to the documented process exit codes: 0 success, 1 actionable/fatal problem, 2 usage error handled by Clap.

## Scope: this file owns

- exit status enum
- mapping from reports

## Out of scope: this file must not own

- business success criteria
- printing

## Allowed dependencies

- application report facts

## Forbidden dependencies and shortcuts

- domain mutation

## Key implementation shape

```rust
pub enum CommandStatus { Success, ActionRequired }

impl From<CommandStatus> for ExitCode {
    fn from(value: CommandStatus) -> Self {
        match value { CommandStatus::Success => ExitCode::SUCCESS, CommandStatus::ActionRequired => ExitCode::FAILURE }
    }
}
```

## Required tests / evidence

- check/status problem results map to 1
- successful no-op maps to 0

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
