# `crates/lingo-prompt/src/render.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Implements stage-specific prompt rendering from typed requests and resolved profile templates.

## Scope: this file owns

- strict Handlebars registry
- typed context mapping
- stage dispatch

## Out of scope: this file must not own

- loading files
- canonical schema ownership
- editor interaction

## Allowed dependencies

- packet builder
- application request types

## Forbidden dependencies and shortcuts

- raw `serde_json::Value` flowing beyond rendering boundary

## Key implementation shape

```rust
impl PromptEngine for HandlebarsPromptEngine {
    fn render_import(&self, request: ImportPromptRequest) -> Result<PromptPacket, PromptFailure> {
        let language_rules = self.render_strict(request.template(), request.context())?;
        Ok(build_import_packet(request, language_rules))
    }
}
```

## Required tests / evidence

- missing template variable is an error
- same request renders byte-for-byte identically
- template cannot replace canonical contract section

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
