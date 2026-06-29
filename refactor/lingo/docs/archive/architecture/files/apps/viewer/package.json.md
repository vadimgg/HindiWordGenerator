# `apps/viewer/package.json`

> **Target kind:** Frontend manifest  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../ARCHITECTURE.md)

## Responsibility

Declares the static viewer frontend build only. Runtime data comes from the localhost Rust viewer server.

## Scope: this file owns

- frontend scripts and dependencies

## Out of scope: this file must not own

- backend/server scripts
- workspace paths
- postinstall mutation

## Allowed dependencies

- Astro and narrowly required frontend libraries

## Forbidden dependencies and shortcuts

- runtime package installation by `lingo viewer`

## Key implementation shape

```json
{
  "private": true,
  "scripts": { "build": "astro build", "check": "astro check" },
  "dependencies": { "astro": "<pinned>" }
}
```

## Required tests / evidence

- lockfile is committed
- build is deterministic in CI

## Design notes

- Pin concrete versions during frontend bootstrap. The placeholder is architectural, not a copy-ready manifest.

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
