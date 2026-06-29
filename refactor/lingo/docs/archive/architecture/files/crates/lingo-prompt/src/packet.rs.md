# `crates/lingo-prompt/src/packet.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns the complete prompt packet structure and appends Rust-owned output contracts and worked examples to editable language guidance.

## Scope: this file owns

- packet section order
- stage heading
- source identity block
- canonical schema/example rendering
- critical invariant list

## Out of scope: this file must not own

- language-specific prose
- clipboard/editor
- reply parsing

## Allowed dependencies

- application prompt request
- domain wire vocabulary descriptors

## Forbidden dependencies and shortcuts

- schema strings copied from profile templates

## Key implementation shape

```rust
pub fn build_import_packet(request: ImportPromptRequest, language_rules: String) -> PromptPacket {
    PromptPacket::builder(PromptStage::Import)
        .context(request.learner_context())
        .language_rules(language_rules)
        .source_identity(request.source_identity())
        .output_contract(import_output_contract())
        .worked_example(import_worked_example())
        .raw_input(request.raw_text())
        .build()
}
```

## Required tests / evidence

- snapshot asserts section ordering
- schema version appears exactly once
- raw input is fenced safely

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
