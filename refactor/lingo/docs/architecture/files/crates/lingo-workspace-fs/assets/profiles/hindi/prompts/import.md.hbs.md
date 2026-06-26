# `crates/lingo-workspace-fs/assets/profiles/hindi/prompts/import.md.hbs`

> **Target kind:** Prompt template asset  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../../../../ARCHITECTURE.md)

## Responsibility

Adds Hindi-specific segmentation and romanisation instructions to the import packet. The Rust packet builder appends the canonical output contract.

## Scope: this file owns

- language-specific model guidance
- profile template variables

## Out of scope: this file must not own

- canonical YAML schema keys
- generic packet instructions
- learner data persistence

## Allowed dependencies

- Handlebars variables supplied by prompt adapter

## Forbidden dependencies and shortcuts

- hardcoded learner identity
- secret values
- duplicated canonical schema

## Key implementation shape

```text
Segment the raw text into useful Hindi study sentences.
Use the {{profile.romanisation_name}} convention for every non-Latin token.
Preserve the source meaning; do not invent missing facts.
```

## Required tests / evidence

- renders with strict mode
- contains no unresolved variables
- canonical schema is supplied separately by Rust

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
