# `crates/lingo-cli/src/viewer_server.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns the localhost-only read-only HTTP server, Rust-to-viewer DTO mapping, static frontend assets, and optional browser launch.

## Scope: this file owns

- bind address/port
- read-only API routes
- viewer DTO serialization
- static asset serving

## Out of scope: this file must not own

- card validation
- canonical file writes
- frontend business rules

## Allowed dependencies

- application ViewerPlan
- HTTP server library

## Forbidden dependencies and shortcuts

- binding non-loopback by default
- generic filesystem browsing endpoints

## Key implementation shape

```rust
#[derive(Serialize)]
struct ViewerCardDto<'a> {
    id: &'a str,
    lead: &'a str,
    secondary: Option<&'a str>,
    english: &'a str,
    audio_url: Option<String>,
}

pub fn serve(plan: ViewerPlan, options: ViewerOptions) -> Result<ViewerSession, ViewerServerError> {
    require_loopback(options.bind)?;
    start_read_only_routes(plan, options)
}
```

## Required tests / evidence

- server binds loopback
- only GET/HEAD routes exist
- path traversal rejected
- DTO contract snapshot

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
