# `crates/lingo-cli/tests/support/mod.rs`

> **Target kind:** Test support  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../../ARCHITECTURE.md)

## Responsibility

Provides typed test builders and fakes shared by CLI integration tests without leaking test helpers into production crates.

## Scope: this file owns

- temporary workspace builder
- fake prompt/audio/environment adapters
- fixture constructors

## Out of scope: this file must not own

- business assertions hidden behind helpers
- raw unvalidated string soup

## Allowed dependencies

- public application/domain APIs

## Forbidden dependencies and shortcuts

- production feature flags for tests

## Key implementation shape

```rust
pub struct TestApp {
    root: TempDir,
    prompt: FakePromptEngine,
    audio: FakeAudioSynthesizer,
}

impl TestApp {
    pub fn new() -> Self { /* construct typed fakes */ }
    pub fn root(&self) -> &Path { self.root.path() }
}
```

## Required tests / evidence

- builders create valid domain values by default
- invalid fixtures require explicit named methods

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
