# Backlog

Improvement items to keep visible while the generator and Astro viewer are being
integrated.

Status values: `open`, `in-progress`, `done`.

## Viewer And Anki

| Item | Priority | Status | Notes |
|---|---|---|---|
| Extract shared Anki preview template rendering into `viewer/src/scripts/anki/renderTemplate.js` and add a smoke check that preview fields/templates stay aligned with export fields/templates | P2 | done | Preview and export share `wordToAnkiFields`, `sentenceToAnkiFields`, note templates, and the preview renderer. `npm run check:anki` verifies representative word and sentence previews. |
| Move `data-sentence-card` onto `SentenceCard`'s own `<article>` | P2 | done | Matches the word card DOM pattern and removes wrapper-only selection/search markers. |
| Update `getSentenceIndex()` JSDoc to document the source/topic group field | P2 | done | Keeps the client data contract aligned with `sentenceSearchIndex`. |
| Polish Deliver as the advanced export workflow | P2 | done | Deliver now has custom deck presets, persisted deck choices, selected-card readiness chips, clearer review counts, and shared export services. |
| Improve quick export feedback | P3 | done | Quick export buttons now expose the target deck in tooltip/ARIA text and keep deck context through checking, offline, sending, success, and failure states. |

## QA And Review

| Item | Priority | Status | Notes |
|---|---|---|---|
| Make QA a first-class pre-export workflow | P2 | done | QA covers word/sentence audio and token issues, filters by issue/card type, jumps to affected cards, tracks reviewed issues locally, and gates export with a two-click override. |
| Add optional audit sampling mode | P3 | open | Sample generated cards for reviewer-agent checks before a large export, useful when prompts or source inputs changed. |

## Generation Workflow

| Item | Priority | Status | Notes |
|---|---|---|---|
| Document and test the no-API-key agent workflow | P2 | done | Documented in `NO_API_AGENT_WORKFLOW.md`; `scripts/check-agent-workflows.py` checks generator agents keep the script-assisted check/write workflow visible. |
| Add repair/backfill commands for legacy batches | P2 | done | Added `repair.py audit` for legacy metadata, missing audio/tokens, and phrase-like sentence input drills; added `repair.py audio` for targeted audio backfill; added validation-backed `repair.py tokens` for exact sentence token backfill. |
| Clarify source vs generated data cleanup | P3 | done | Documented in `DATA_SURFACES.md`: source of truth, generated learner-facing data, projections/build artifacts, read-only reference material, allowed writers, and safe cleanup rules. |
| Plan local transcription workflow | P2 | done | Added `TRANSCRIPTION_PLAN.md` with local Whisper backend options, data surfaces, CLI shape, reference-text correction, viewer integration, and phased implementation. |
