# 12 — Testing and Evidence

## Testing strategy

Use tracer-bullet evidence first, then harden with focused invariants.

The first vertical slice should prove:

```text
init --example
prepare extract run
write reply.yaml
apply --dry-run
apply commit
status shows drafts
prepare enrich run
visible status derives enriching from run_sentences
```

Do not build a huge matrix before the first slice validates the API and schema shape.

## Domain tests

| Area | Evidence |
|---|---|
| ID opacity | valid/invalid parse; no slug extraction API |
| lifecycle | only `draft/enriched` wire names |
| visible status | pending claim produces `Enriching` without changing lifecycle |
| field authority | human fields reject model overwrite |
| target edit impact | no-change/audio-only/semantic cases |
| active curation | semantic edit does not clear active |
| word key | profile participates in derivation |
| audio fingerprint | profile/backend/voice/model/lang/text affect hash |

## Service tests

Use fake repository and fake workspace.

| Use case | Evidence |
|---|---|
| status | ranks pending run before enrich/audio/publish |
| apply target | multiple pending runs returns `ChoiceRequired`, no prompt |
| apply dry-run | validates and writes nothing |
| apply invalid | records validation error, run remains pending |
| apply idempotent | same reply hash is no-op after applied |
| edit target | impact-based invalidation report |
| audio | missing/stale selection policy |
| publish | study/anki warn on unQA'd, package does not |

## SQLite tests

Use temp DB fixtures and real transactions.

| Invariant | Evidence |
|---|---|
| schema version | `meta.schema_version = 1` |
| PRAGMAs | connection enables WAL and foreign keys |
| run_sentences canonical | no sentence run-id columns exist |
| visible enriching | query derives from pending run claim |
| transaction atomicity | invalid apply leaves no partial writes |
| retryable validation | failed apply leaves run pending with error |
| idempotent apply | same reply hash no-op; different hash error |
| deck rename | audio path unchanged |
| tags/tokens | normalized tables round-trip |

## Workspace tests

| Area | Evidence |
|---|---|
| safe paths | reject empty, absolute, `..`, backslash |
| audio path | `audio/<sentence-id>.mp3`, no slug input |
| run.json mirror | repair from DB when stale |
| atomic write | read-back verification catches mismatch |
| init --example | concrete `raw/example.md` exists and next action is copyable |
| no placeholder next | service returns `Blocked` instead of fake command when no raw file exists |

## Handoff tests

| Area | Evidence |
|---|---|
| fence parser | missing/multiple/wrong fence rejected |
| extract codec | valid reply parses to typed DTO |
| enrich codec | invalid register rejected |
| QA codec | unknown correction field rejected |
| format owner | reply format strings come from enum owner |
| human overwrite | validator rejects attempted overwrite |

## Publish/import tests

| Area | Evidence |
|---|---|
| package | export then import round-trip |
| package integrity | written files read back and checksummed |
| study | stable schema fixture opens and rows match snapshot |
| Anki | GUID derived from sentence ID |
| import dry-run | reports duplicate target/different English conflict |
| missing audio | package includes null; study/anki skip/report |

## Architecture evidence

- Cargo dependency audit for forbidden arrows.
- Grep/codemap check that `lingo-domain` has no `rusqlite`, `clap`, filesystem layout, or CLI renderer imports.
- Grep/codemap check that command modules do not contain SQL strings.
- Tests proving fake repository implements service port.
- No broad `utils`, `common`, or magic string vocabulary crates.

## Acceptance checklist

```text
[ ] schema initializes from schema_v02.sql
[ ] all DB connections set WAL + foreign_keys
[ ] status derives enriching from pending run_sentences
[ ] apply --dry-run validates and writes nothing
[ ] apply commit is one transaction
[ ] apply retry leaves failed run pending
[ ] same-hash reapply is idempotent
[ ] different-hash reapply errors
[ ] target semantic edit invalidates AI derived fields only
[ ] target audio-only edit marks audio stale only
[ ] active flag survives target edits
[ ] audio paths are flat by sentence id
[ ] deck slug rename does not move audio
[ ] word keys use language profile
[ ] audio fingerprints use profile/backend/voice/model/lang/text
[ ] package export/import round-trips
[ ] MySQL appendix is not wired into implementation
```
