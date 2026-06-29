# `crates/lingo-cli/src/output.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns reusable terminal presentation mechanics and maps typed reports into answer-first output without becoming a second truth surface.

## Scope: this file owns

- color policy
- status glyphs
- path/detail formatting
- next block rendering
- machine-output mapping if added

## Out of scope: this file must not own

- domain facts
- next-action decisions
- stable IDs duplicated from owners

## Allowed dependencies

- application reports
- console/terminal library

## Forbidden dependencies and shortcuts

- filesystem/provider calls

## Key implementation shape

```rust
pub fn render_next(out: &mut dyn Write, next: &NextAction, color: ColorPolicy) -> io::Result<()> {
    let Some(hint) = next.command_hint() else { return Ok(()); };
    writeln!(out, "\nNext\n  {}", color.command(hint.as_str()))
}
```

## Required tests / evidence

- snapshot default text output
- NO_COLOR and `--no-color`
- machine output contains same facts when introduced

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
