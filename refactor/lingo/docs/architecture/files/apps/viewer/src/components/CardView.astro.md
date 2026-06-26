# `apps/viewer/src/components/CardView.astro`

> **Target kind:** Astro component  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../../ARCHITECTURE.md)

## Responsibility

Renders one viewer DTO with accessible target/romanisation hierarchy, English meaning, and optional audio controls.

## Scope: this file owns

- card presentation
- accessible labels
- secondary-line visibility

## Out of scope: this file must not own

- data fetching
- business validation
- audio path resolution

## Allowed dependencies

- typed props

## Forbidden dependencies and shortcuts

- innerHTML with untrusted card text

## Key implementation shape

```text
---
const { card } = Astro.props;
---
<article data-card-id={card.id}>
  <h2>{card.lead}</h2>
  {card.secondary && <p class="secondary">{card.secondary}</p>}
  <p>{card.english}</p>
  {card.audio_url && <audio controls src={card.audio_url}></audio>}
</article>
```

## Required tests / evidence

- HTML escaping
- no-audio state
- secondary hidden state

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
