# `apps/viewer/astro.config.mjs`

> **Target kind:** Frontend configuration  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../ARCHITECTURE.md)

## Responsibility

Configures a static Astro build whose assets are embedded by the Rust viewer server.

## Scope: this file owns

- static output mode
- base asset settings

## Out of scope: this file must not own

- filesystem API access
- server-side canonical data loading

## Allowed dependencies

- Astro

## Forbidden dependencies and shortcuts

- Node server runtime dependency

## Key implementation shape

```text
import { defineConfig } from "astro/config";
export default defineConfig({ output: "static" });
```

## Required tests / evidence

- static build emits no server entrypoint

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
