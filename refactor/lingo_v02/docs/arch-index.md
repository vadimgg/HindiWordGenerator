# Lingo v0.2 Architecture Pack — Updated

This pack is the architecture source for the CLI-first Lingo rebuild. It updates the previous design after review and locks the schema and code-architecture decisions that affect implementation.

Claude owns CLI and UI docs. This pack owns architecture, persistence, crate/module boundaries, state machines, domain model, run/apply behavior, reusable mechanics, and schema design.

## Locked architecture decisions

| Area | Decision |
|---|---|
| Canonical store | SQLite `library.db` is the authoring source of truth. |
| MySQL | Appendix only. Do not implement a MySQL adapter for the personal CLI. |
| Prototype compatibility | Do not preserve `collection/batch/section` internals. Rebuild cleanly around `deck`. |
| Sentence lifecycle | Persist only `draft | enriched`. Do **not** persist `enriching`. |
| Visible `enriching` | Derived from pending `runs` + `run_sentences` claims. |
| Curation | `sentences.active` boolean, independent from lifecycle. |
| QA | `sentences.qa_checked_at`, independent from lifecycle and curation. |
| Run participation | `run_sentences` is canonical. No `sentences.enrich_run_id` or `qa_run_id`. |
| Audio path | Internal authoring path is flat: `audio/<sentence-id>.mp3`. |
| Audio staleness | Fingerprint includes profile, backend, voice/model, audio language, and exact target text. |
| Sentence IDs | Permanent opaque IDs. Never parse deck slug or position from an ID. |
| Target edits | Impact-based invalidation. Do not automatically clear `active`. |
| Word keys | Derived through the language profile. |
| SQLite concurrency | `foreign_keys=ON`, `journal_mode=WAL`, `synchronous=NORMAL`, `busy_timeout`, `BEGIN IMMEDIATE` for writes. |
| Schema version | Clean rebuild starts at `meta.schema_version = 1`. |
| Crates | Start with a small workspace and split by dependency pressure, not symmetry. |
| `apply` | Full validation before write, one SQLite transaction, dry-run, idempotent re-apply. |

## Files

- [`arch/00-architecture-decisions.md`](arch/00-architecture-decisions.md)
- [`arch/01-boundary-map.md`](arch/01-boundary-map.md)
- [`arch/02-crate-and-file-structure.md`](arch/02-crate-and-file-structure.md)
- [`arch/03-domain-model.md`](arch/03-domain-model.md)
- [`arch/04-state-machines.md`](arch/04-state-machines.md)
- [`arch/05-application-api.md`](arch/05-application-api.md)
- [`arch/06-sqlite-schema.md`](arch/06-sqlite-schema.md)
- [`arch/07-workspace-and-files.md`](arch/07-workspace-and-files.md)
- [`arch/08-run-handoff-and-apply.md`](arch/08-run-handoff-and-apply.md)
- [`arch/09-prompts-and-reply-codecs.md`](arch/09-prompts-and-reply-codecs.md)
- [`arch/10-audio-publish-import.md`](arch/10-audio-publish-import.md)
- [`arch/11-reusable-utilities.md`](arch/11-reusable-utilities.md)
- [`arch/12-testing-and-evidence.md`](arch/12-testing-and-evidence.md)
- [`arch/13-refactor-plan.md`](arch/13-refactor-plan.md)
- [`arch/appendix-mysql.md`](arch/appendix-mysql.md)
- [`arch/schema_v02.sql`](arch/schema_v02.sql)
- [`arch/mysql_schema.sql`](arch/mysql_schema.sql)

## How to use this pack

Implementation should start with the tracer bullet in `13-refactor-plan.md`: initialize a library, create an extract run, write a reply, apply it transactionally, and show status. Add horizontal command coverage only after that vertical slice proves the domain, store, workspace, and CLI edges.

The UI should later call the same application use cases as the CLI. It should not get its own data path.
