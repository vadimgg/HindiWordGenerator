# Doc Reconciliation Notes

Architecture/persistence is owned by the arch pack ([`arch.md`](./arch.md)
+ `arch/*`). CLI/workflow docs are owned here. This file records the major
architecture decisions that the CLI docs already depend on, plus implementation
checks that should not be lost when coding starts.

---

## 1. Locked by the arch pack

The "Locked architecture decisions" table in `arch.md` resolved the prior
pending items: state split (`status = draft|enriched`, `active` bool,
`qa_checked_at`; `enriching` **derived** from pending runs), `apply` is atomic /
full-validate-before-write / idempotent / `--dry-run`, impact-based edit
invalidation, audio provenance + fingerprint staleness, `meta` table +
`schema_version = 1`, `run_sentences` canonical (no `enrich_run_id`/`qa_run_id`),
**flat internal audio path** `audio/<sentence-id>.mp3`, opaque sentence IDs, and
WAL + `BEGIN IMMEDIATE`.

`active` is the SQLite column for approval. The CLI must say **approve** /
**unapprove** everywhere; it must not expose `active` / `inactive` flags.

## 2. Done in the CLI docs this pass

- `cli/apply.md` — `--dry-run` / `--oldest` / `--all`, non-interactive no-prompt
  with a `multiple_pending_runs` blocked result, atomic/idempotent guarantees.
- `cli/approve.md` — explicit approval commands for sentence ids and deck slugs.
- `cli/edit.md` — impact-based target invalidation + `--keep-derived`; no
  approval flags.
- `cli/audio.md` — flat internal path, missing-**or-stale** default selection.
- `cli/doctor.md` — checks `meta.schema_version` (v1); abandoned pending runs
  instead of "stuck enriching"; stale/broken audio.
- `package-and-agents.md` — internal audio flat; note that exports may re-folder.
- (earlier) `cli/audio.md` gtts-is-networked; `cli/publish.md` four formats +
  `Scope` line + warn-only QA gate; `cli/init.md` `--example`; `CLI.md` result
  contract + exit codes + `--ascii` note.

## 3. Resolved after the arch pass

The arch pack resolved the model gaps that previously blocked CLI docs:

- **Origin / provenance:** `sentences.origin` plus source package/library/sentence
  ids are durable and survive run cleanup.
- **Approval gate:** `active` means approved for study, is allowed only when
  `status = enriched`, and is the default selection for `study`/`anki`.
- **Import policy:** same-library new-format package restore preserves
  approval/QA; cross-library package import resets approval/QA unless
  `--trust-approval` is used.
- **Derived enriching:** persisted sentence lifecycle is only `draft | enriched`;
  `enriching` is derived from pending enrich runs.

## 4. CLI-doc reconciliation status

Done:

- `cli/publish.md` — approved-only default for `study`/`anki` +
  `--include-unapproved`; `package`/`db` lossless.
- `cli/import.md` — origin recording + new-format same-library/cross-library
  approval policy.
- `package-and-agents.md` — truth/in-flight/derived layout and export shapes.
- `cli/show.md` / `cli/ls.md` — origin, approval badge, and derived `enriching`.
- `cli/status.md` + `workflows.md` — explicit approval step before audio/publish.

Still useful to verify during implementation:

- `enrich --force` must clear `active` when automated rewrites change
  study-facing fields.
- `study` and `anki` must never export `draft` rows, even with
  `--include-unapproved`.
- `approve --interactive` is intentionally deferred. Implement the non-interactive
  approval spine first.

---

## Appendix — Deferred test matrix (implementation phase)

- [ ] UTF-8 round-trip with real Hindi/Japanese text (raw → task → reply → DB →
      package → study → Anki).
- [ ] Golden output tests: styled, `--no-color`, `--json`, `--ascii` for every command.
- [ ] Result-contract tests: exactly one of next/done/blocked; no placeholder in `next`.
- [ ] `apply` transaction rollback; idempotency; `--dry-run`; `multiple_pending_runs` blocked.
- [ ] Human-authority overwrite rejection (enrich + qa).
- [ ] Target-edit impact classification + invalidation.
- [ ] Stale-audio detection; missing-audio skip/report for study/anki.
- [ ] Package export → import round-trip (multi-deck, slug dedupe, audio, approval state).
- [ ] Study export schema-version; Anki GUID-update (not duplicate).
- [ ] Sentence-id stability across `deck set --slug`, `edit --to`, re-enrich, import, publish.
- [ ] One-pending-enrich-claim concurrency test (double-claim rejected).
- [ ] Viewer + CLI WAL concurrency / locking.
- [ ] Exit-code contract per command.
