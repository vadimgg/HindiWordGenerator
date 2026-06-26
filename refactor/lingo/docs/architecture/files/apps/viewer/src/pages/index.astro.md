# `apps/viewer/src/pages/index.astro`

> **Target kind:** Astro page  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../../ARCHITECTURE.md)

## Responsibility

Owns the viewer page composition and session-only lead toggle.

## Scope: this file owns

- page layout
- loading/error/empty states
- composition of card components

## Out of scope: this file must not own

- canonical schema parsing
- workspace writes
- audio URL construction policy

## Allowed dependencies

- viewer API client
- CardView component

## Forbidden dependencies and shortcuts

- direct filesystem access

## Key implementation shape

```text
---
import CardView from "../components/CardView.astro";
---
<main>
  <header><button id="toggle-lead">Swap lead</button></header>
  <section id="cards" aria-live="polite"></section>
</main>
<script>/* fetch /api/session and render safe DTOs */</script>
```

## Required tests / evidence

- empty/loading/error states
- toggle is session-only

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
