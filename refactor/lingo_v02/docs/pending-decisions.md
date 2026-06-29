# Pending Decisions & Deferred Doc Work

Schema and architecture are owned by a separate pass (gpt-pro). This doc tracks
everything the CLI docs are **waiting on** so we touch each entangled file only
once, after the data model is fixed — plus the open questions and the deferred
test work.

Status legend: **decided** (agreed, awaiting schema to land) · **open** (needs a
call) · **queued** (CLI-doc edit ready to apply once the model is clear).

---

## 1. Decided — awaiting schema/architecture, then conform CLI docs

These are agreed between both review passes; they affect the data model (gpt-pro)
and have a CLI-doc face (me, once his schema lands):

- **State split.** `status: draft | enriching | enriched` + `active: boolean` +
  `qa_checked_at` (nullable). `active` is curation/approval, *not* a pipeline
  stage. Predicates become: publishable = `enriched`; featured = `active`;
  needs-QA = `enriched AND qa_checked_at IS NULL`.
- **`apply` guarantees.** Validates the entire reply before any write; commits in
  one transaction; never half-commits; idempotent for an already-applied reply;
  records reply hash + applied-at + last validation error.
- **Edit invalidation.** Changing `target` invalidates derived fields
  (romanisation/literal/breakdown if AI-authored, breakdown, `qa_checked_at`) and
  marks audio stale; human-authored fields are preserved with a warning, not
  deleted.
- **Audio provenance + staleness.** Track backend/voice/lang/generated-at and a
  hash of the spoken text so audio can be detected as stale when the sentence
  changes.
- **Schema metadata.** A `meta` table holding `schema_version`,
  `created_with_lingo_version`, `last_migrated_at`, `language_profile` — the thing
  [`doctor`](./cli/doctor.md) checks.

## 2. Open — needs a call (gpt-pro)

- `active` as a plain boolean vs a curation enum (`candidate|active|hidden`).
  Recommendation: **boolean** for a single-user tool.
- Where `apply` run metadata lives (extra columns on `runs` vs a side table).
- Audio staleness: recompute-on-read vs flag-on-edit.
- `migrate` command + automatic pre-migration backup behavior.
- Language-profile representation (and a future `lingo profiles` surface) — matters
  most when moving Hindi → Japanese (segmentation, "word", romanisation differ).
- Whether `apply` ever needs `--all` (batch-apply multiple pending runs) or just
  `--oldest`.

## 3. Queued CLI-doc edits — apply after the model is clear

Each bullet is a single-file edit held until §1/§2 resolve, to avoid rework:

- **`cli/edit.md`** — replace `--status active` with `--active` / `--inactive`;
  document the target-edit invalidation cascade and a `--keep-derived` escape
  hatch; state that human fields are preserved-with-warning, never silently
  dropped.
- **`cli/apply.md`** — add `--dry-run` (validate + "would change" counts),
  `--oldest`, non-interactive **no-prompt** behavior with a structured
  `multiple_pending_runs` error; document atomic/idempotent/full-validation
  guarantees and the recorded run metadata. (Exit codes and the next/done/blocked
  contract already live in [`CLI.md`](./CLI.md).)
- **`cli/audio.md`** — document stale-audio detection (regenerate when the spoken
  text changed) and surface provenance in `show` / `--json`.
- **`cli/show.md`, `cli/ls.md`, `cli/status.md`** — render `active` as a separate
  badge/flag, not a `status` value; show the needs-QA predicate consistently.
- **`cli/doctor.md`** — point the schema-version check at the `meta` table; add the
  `migrate` recovery path once it exists.
- **`cli/init.md`** — once `meta` exists, note which version a fresh library is
  created at.

---

## 4. Done in this pass (schema/architecture-independent)

For traceability — already applied, no dependency on the model:

- `cli/audio.md` — corrected "gtts is free and local" → free but **networked**
  (Google Translate TTS); the rest of Lingo stays offline.
- `cli/publish.md` + `package-and-agents.md` — "three formats" → **four**
  (`package|study|anki|db`); added a visible `Scope` line; added the **warn-only**
  QA gate (`--allow-unqa`; `package`/`db` never gated, `study`/`anki` warn).
- `cli/init.md` — added `--example` (writes `raw/example.md`) so `Next:` is a real
  command; without it, prints an instruction block instead of a placeholder
  `Next:`.
- `CLI.md` — formalized the **result contract** (`next` | `done` | `blocked`),
  added the **exit-code table**, and added `--ascii` as a *planned* global flag
  note (our docs are valid UTF-8; this is for poor-rendering terminals/pipelines).

---

## Appendix — Deferred test matrix (implementation phase)

Not blocked on the model; capture now so it isn't lost. Required before the CLI is
considered ready for serious use:

- [ ] UTF-8 round-trip tests with real Hindi/Japanese text
      (raw → task → reply → DB → `ls`/`show` → package → study.sqlite → Anki).
- [ ] Golden output tests for styled, `--no-color`, and `--json` for every command.
- [ ] `Next:`/result-contract tests: executable, no placeholders, correct priority,
      exactly one of next/done/blocked.
- [ ] `apply` transaction tests (failure leaves no partial state).
- [ ] Idempotent `apply` (re-applying an applied run is a no-op or clear error).
- [ ] `apply --dry-run` validation + would-change counts.
- [ ] Human-authority overwrite rejection tests (enrich + qa).
- [ ] Target-edit invalidation behavior.
- [ ] Publish quality-gate behavior for un-QA'd and missing-audio sentences.
- [ ] Package export → import round-trip (multi-deck, slug dedupe, audio).
- [ ] Study export schema-version test.
- [ ] Anki export stable-id / GUID-update test (re-export updates, not duplicates).
- [ ] Sentence-id stability across `deck set --slug`, `edit --to`,
      `enrich --force`, `import`, `publish`.
- [ ] Viewer + CLI concurrency / DB-locking test.
- [ ] Destructive-command confirmation tests (`deck delete`, `runs clean --abandoned`).
- [ ] Correct audio-backend labeling (networked vs offline).
