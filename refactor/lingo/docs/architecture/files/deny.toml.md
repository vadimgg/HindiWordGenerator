# `deny.toml`

> **Target kind:** Dependency policy  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../ARCHITECTURE.md)

## Responsibility

Configures dependency-policy checks for licenses, advisories, duplicate crates, and forbidden sources.

## Scope: this file owns

- dependency source policy
- license allowlist
- advisory behavior

## Out of scope: this file must not own

- architecture dependency direction
- Rust lint policy
- application security logic

## Allowed dependencies

- cargo-deny

## Forbidden dependencies and shortcuts

- git dependencies without an explicit review exception
- unknown licenses silently accepted

## Key implementation shape

```text
[advisories]
yanked = "deny"

[sources]
unknown-registry = "deny"
unknown-git = "deny"

[licenses]
confidence-threshold = 0.93
```

## Required tests / evidence

- `cargo deny check` passes in CI
- exceptions include owner, reason, and removal condition

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
