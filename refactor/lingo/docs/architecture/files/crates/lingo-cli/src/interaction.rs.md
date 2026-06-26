# `crates/lingo-cli/src/interaction.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns editor, clipboard, picker, and packet-loop mechanics at the app edge. It receives prepared packets and returns reply bytes or a named non-interactive outcome.

## Scope: this file owns

- `PromptMode` execution
- clipboard tool detection
- editor invocation
- batch picker

## Out of scope: this file must not own

- prompt rendering/parsing
- canonical writes
- workflow next-action policy

## Allowed dependencies

- std::process
- application prepared/apply APIs

## Forbidden dependencies and shortcuts

- shell interpolation of learner text
- generic boolean mode arguments

## Key implementation shape

```rust
pub enum PromptMode {
    Interactive,
    PrintOnly,
    ApplyFile(PathBuf),
}

pub fn acquire_reply(prepared: &PreparedPrompt, mode: PromptMode) -> Result<ReplyInput, InteractionError> {
    match mode {
        PromptMode::PrintOnly => Ok(ReplyInput::Printed),
        PromptMode::ApplyFile(path) => Ok(ReplyInput::Reply(fs::read_to_string(path)?)),
        PromptMode::Interactive => interactive_editor_loop(prepared),
    }
}
```

## Required tests / evidence

- print-only performs no editor/clipboard call
- apply-file reads exact file
- editor failures retain run packet

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
