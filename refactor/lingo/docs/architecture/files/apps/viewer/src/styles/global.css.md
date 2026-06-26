# `apps/viewer/src/styles/global.css`

> **Target kind:** Stylesheet  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../../ARCHITECTURE.md)

## Responsibility

Owns viewer typography, target/romanisation visual hierarchy, responsive layout, and accessible focus styles.

## Scope: this file owns

- presentation-only CSS

## Out of scope: this file must not own

- data-dependent business logic
- hidden accessibility states

## Allowed dependencies

- browser CSS

## Forbidden dependencies and shortcuts

- JavaScript behavior encoded through fragile selectors

## Key implementation shape

```css
:root { font-family: system-ui, sans-serif; }
.secondary { opacity: 0.68; }
button:focus-visible, audio:focus-visible { outline: 2px solid currentColor; }
```

## Required tests / evidence

- contrast and keyboard smoke checks

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
