# `crates/lingo-workspace-fs/assets/profiles/hindi/prompts/build.md.hbs`

> **Target kind:** Prompt template asset  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../../../../ARCHITECTURE.md)

## Responsibility

Adds Hindi-specific enrichment guidance to the build packet without defining the card schema.

## Scope: this file owns

- language-specific word-breakdown guidance
- register/grammar instructions

## Out of scope: this file must not own

- JSON field names
- source identity rules
- generic validation policy

## Allowed dependencies

- Handlebars context

## Forbidden dependencies and shortcuts

- schema duplication
- provider-specific instructions

## Key implementation shape

```text
Enrich each Hindi source item for a practical learner.
Keep grammar explanations plain. Mark register only when the distinction is real.
Romanise every target-language token using {{profile.romanisation_name}}.
```

## Required tests / evidence

- strict rendering
- no canonical field-name duplication
- worked packet snapshot includes this section once

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
