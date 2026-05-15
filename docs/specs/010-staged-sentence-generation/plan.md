# Plan

## Design

Keep the command surface stable and replace the generation core with a staged
pipeline. The smallest useful implementation is a shared stage prompt registry,
typed stage parsers, a merger that combines stage outputs by source ID, and an
updated generator that calls the same configured model for each stage before
running the existing validator/writer.

## Modules

| Module | Responsibility |
|---|---|
| `src/sentence_generate.rs` | Orchestrate plan, readiness, stage calls, validation, writes, and user-facing result. |
| `src/sentence_enrichment.rs` or new `src/sentence_stages.rs` | Stage prompt registry, prompt rendering, response parsing, merge logic. |
| `src/run_report.rs` | Add stage metadata to sentence run reports. |
| `src/sentence_validate.rs` | Continue to enforce final candidate validity. |
| `src/accepted_writer.rs` | Continue atomic accepted-output writes. |
| `src/eval.rs` / prompt files | Source of prompt lessons; reuse prompt text or share constants where practical. |
| `docs/DESIGN.md`, `docs/ROADMAP.md` | Update active docs from single enrichment to staged default. |

## Operation Order

1. Audit existing single-call generation tests and prompt helpers.
2. Define stage prompt metadata: ID, version, prompt text, response shape.
3. Implement parsers for:
   - register output
   - literal output
   - word-breakdown output
4. Implement staged merge:
   - require exactly one record per source row per stage
   - reject missing/duplicate/extra IDs
   - copy trusted source fields from `PlannedSentenceBatch`
   - create tokens/words from word-breakdown output
5. Extend run report structs with `stages[]`.
6. Update `sentence_generate` to:
   - plan pending batches
   - check model readiness once
   - call register, literal, word-breakdown stages in order
   - merge and validate
   - write accepted output only after validation succeeds
   - write accepted or failed run report
7. Update active docs to describe staged generation.
8. Validate with unit tests, integration tests, and smoke commands.

Point of no return: accepted output write. Run report writes may happen for
failed attempts, but accepted output must never be written before staged merge
and validation succeed.

## Work Package Sequence

| WP | Purpose |
|---|---|
| WP01 | Define stage prompt registry, response types, and parsing/merge tests. |
| WP02 | Update `hindi sentences generate` orchestration to call stages and validate merged output. |
| WP03 | Extend run reports and docs for staged prompt metadata and failure recovery. |
| WP04 | End-to-end validation with fake model client and one live Ollama smoke run when safe. |

## Risks

| Risk | Mitigation |
|---|---|
| Eval prompt and generation prompt drift. | Share prompt constants or record explicit stage prompt versions/fingerprints in run reports. |
| Multi-stage calls slow down generation. | Keep batch sizes small, record per-stage timings, and avoid full-enrichment in default flow. |
| Stage outputs mismatch by ID. | Merge helper rejects missing, duplicate, or extra IDs. |
| Generated output changes accepted source fields. | Merger copies source fields only from planner data. |
| Failure path writes partial accepted output. | Validation and all stage calls complete before accepted writer runs. |

## Validation

- `cargo fmt --check`
- `cargo test`
- `make check`
- Focused tests for parsers and merger.
- Fake model integration tests for successful staged generation and no-write
  failures.
- Manual live run only after confirming expected output target is safe.
