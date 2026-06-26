# `apps/viewer/src/lib/api.ts`

> **Target kind:** TypeScript module  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../../ARCHITECTURE.md)

## Responsibility

Owns the browser-side HTTP client and runtime validation of the Rust viewer DTO.

## Scope: this file owns

- GET endpoints
- DTO decoding
- network error mapping

## Out of scope: this file must not own

- canonical card schema
- workspace paths
- mutation endpoints

## Allowed dependencies

- browser fetch
- small runtime schema validator

## Forbidden dependencies and shortcuts

- generic `any` crossing into components

## Key implementation shape

```typescript
export async function loadSession(): Promise<ViewerSession> {
  const response = await fetch("/api/session");
  if (!response.ok) throw new Error(`viewer API failed: ${response.status}`);
  return parseViewerSession(await response.json());
}
```

## Required tests / evidence

- invalid DTO rejected
- non-2xx error surfaced
- no POST/PUT helpers

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
