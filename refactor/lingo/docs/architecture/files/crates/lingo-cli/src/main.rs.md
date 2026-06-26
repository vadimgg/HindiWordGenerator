# `crates/lingo-cli/src/main.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns process startup, top-level error handling, and final exit code only.

## Scope: this file owns

- parse-dispatch boundary
- fatal error rendering call
- process exit

## Out of scope: this file must not own

- command behavior
- adapter construction details
- domain validation

## Allowed dependencies

- cli parser
- composition
- exit mapping

## Forbidden dependencies and shortcuts

- direct workspace/provider logic

## Key implementation shape

```rust
fn main() -> ExitCode {
    match run() {
        Ok(status) => status.into(),
        Err(error) => {
            eprintln!("{}", render_fatal(&error));
            ExitCode::FAILURE
        }
    }
}
```

## Required tests / evidence

- help/version handled by clap
- no panic/unwrap path
- fatal error produces non-zero exit

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
