# `crates/lingo-prompt/src/error.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns prompt-rendering and reply-parse errors with structured stage/path context.

## Scope: this file owns

- template error
- reply syntax error
- draft conversion error
- stage context

## Out of scope: this file must not own

- terminal recovery wording
- file paths unrelated to prompt run

## Allowed dependencies

- thiserror
- application prompt failure mapping

## Forbidden dependencies and shortcuts

- CLI output

## Key implementation shape

```rust
#[derive(Debug, thiserror::Error)]
pub enum PromptAdapterError {
    #[error("prompt template is missing variable {name}")]
    MissingVariable { name: String },
    #[error("invalid {stage} reply at {path}")]
    InvalidReply { stage: PromptStage, path: FieldPath, message: String },
}
```

## Required tests / evidence

- errors preserve stage and field path
- raw learner text is not copied into error display

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
