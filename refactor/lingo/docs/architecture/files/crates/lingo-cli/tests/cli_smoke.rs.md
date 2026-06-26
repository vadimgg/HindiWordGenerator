# `crates/lingo-cli/tests/cli_smoke.rs`

> **Target kind:** Integration test  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Tests public command grammar, help, exit codes, and key text-output contracts through the compiled `lingo` binary.

## Scope: this file owns

- CLI black-box behavior
- temporary workspace setup
- output snapshots

## Out of scope: this file must not own

- deep domain validation cases
- provider network calls

## Allowed dependencies

- assert_cmd or equivalent
- tempfile

## Forbidden dependencies and shortcuts

- real home/config mutation

## Key implementation shape

```rust
#[test]
fn check_problems_exit_one_and_show_next() {
    let workspace = TestWorkspace::with_invalid_card();
    Command::cargo_bin("lingo").unwrap()
        .current_dir(workspace.root())
        .arg("check")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("Next"));
}
```

## Required tests / evidence

- help for every command
- NO_COLOR
- usage error 2
- no legacy commands

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
