# `crates/lingo-workspace-fs/assets/profiles/hindi/profile.toml`

> **Target kind:** Built-in profile asset  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../../../ARCHITECTURE.md)

## Responsibility

Defines Hindi language facts by selecting Rust-owned policies and provider defaults. It contains data, not executable behavior.

## Scope: this file owns

- profile metadata
- existing romanisation convention ID
- default audio language/voice IDs

## Out of scope: this file must not own

- validation regexes
- API keys
- prompt body
- provider code

## Allowed dependencies

- profile schema owned by workspace config adapter

## Forbidden dependencies and shortcuts

- unknown semantic policy IDs
- secrets

## Key implementation shape

```toml
id = "hindi"
language = "Hindi"
code = "hi"
script = "Devanagari"
direction = "ltr"
romanisation = "iast-tilde"

[audio.gtts]
language = "hi"
```

## Required tests / evidence

- catalog parses it
- all IDs map to Rust-owned values
- no secret-like keys are present

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
