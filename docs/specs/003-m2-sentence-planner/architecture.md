# Architecture

## Part 1 - What Changed And Why

M2 turns stable source IDs into a read-only sentence generation plan. The main
risk is accidentally treating old output as clean when it has no lineage. The
planner must be honest: missing `source_ref` is reported as `missing lineage`,
and only current source fingerprints count as done.

## Part 2 - Module Ownership

| Module | Owns | Must never |
|---|---|---|
| `src/cli.rs` | Parse `sentences plan --max-batches <n>`. | Read source/output files or derive planner state. |
| `src/main.rs` | Dispatch command and print planner report. | Contain planner business rules. |
| `src/source_ids.rs` or a shared source module | Source YAML parsing and ID validation. | Know accepted output semantics. |
| `src/sentence_plan.rs` | Read accepted sentence output, derive done/pending/deferred/source changed/missing lineage, choose target filenames. | Write files, call models, or print directly. |
| `src/project.rs` | Project root and project-relative path helpers. | Know planner state rules. |

## Part 3 - Command Internals

### `hindi sentences plan --max-batches 1`

What the user sees:

```text
Sentence Plan

Sources
  files              6
  source items       296
  valid ids          296

Accepted Output
  batch files        4
  accepted cards     20
  done               0
  missing lineage    20
  source changed     0

Plan
  max batches        1
  batch size         5
  planned batches    1
  planned items      5
  pending items      296
  deferred items     291

Planned Files
  output/sentences/complete_hindi_chapter_02_sentences_batch_05.json

Next
  M4 adds: hindi sentences generate --max-batches 1
```

Internal sequence:

```text
src/main.rs
  parse args through src/cli.rs
  discover project root
  call sentence planner domain

src/sentence_plan.rs
  load active sentence YAML
  validate source IDs
  compute source fingerprints
  load existing output/sentences JSON files
  classify accepted cards: done, missing lineage, source changed
  derive pending source rows
  apply max-batches limit
  choose next unused batch filenames
  return typed SentencePlan

src/main.rs
  print SentencePlan
  exit 0 when source is valid
  exit 1 when source/output parsing or source ID validation blocks planning
```

Reject in review:

- Planner writes any file under `input/`, `output/`, `audio/`, or `runs/`.
- Missing lineage is counted as done.
- Batch filename selection ignores existing files.
- `--max-batches` is treated as source items instead of output files.
- Hindi is printed without romanisation directly below it.

## Part 4 - Shared Abstractions

### Source Fingerprint

Used by:
- M2 planner
- future M4 generation

Contract:
- Unicode NFC-normalize each field.
- Trim each field.
- Collapse internal whitespace runs to one space.
- Preserve case and punctuation.
- Hash `hindi + "\n" + romanisation + "\n" + english`.

### Accepted Output Reader

Used by:
- M2 planner
- future audit and generation guards

Contract:
- Reads `output/sentences/*.json`.
- Extracts sentence cards and optional `source_ref`.
- Does not validate full learner card shape; M3 owns validator completeness.
- Never writes files.

### Batch Target Selector

Used by:
- M2 planner
- future M4 generation

Contract:
- Input: source file stem and existing output filenames.
- Output: next unused `output/sentences/<stem>_batch_XX.json`.
- Never overwrites or reserves the file.

## Part 5 - Data And Drift Risks

### Persistent Files

| File | Written By | Read By | Rule |
|---|---|---|---|
| `input/sentences/*.yaml` | M1.5 migration and humans | M2 planner | Source authority. Planner only reads. |
| `output/sentences/*.json` | Python archive and future Rust generation | M2 planner/viewer/export | Accepted-card authority. Planner only reads. |
| `docs/ROADMAP.md` | Humans/agents | Humans/agents | Mark planner done only after implementation. |
| `docs/specs/003-m2-sentence-planner/**` | Brief/humans/agents | Brief/humans/agents | Spec/task context only. |

### Drift Scenario A - Python-Era Output Looks Done

**How it happens.** Existing output contains the same Hindi/English but no
`source_ref`.

**What breaks.** Planner skips rows that have never been accepted by Rust
lineage rules.

**Detection.** Fixture output without `source_ref` reports `missing lineage`.

**Resolution.** Count it separately; do not backfill or mark done.

### Drift Scenario B - Source Text Changes After Acceptance

**How it happens.** YAML `hindi`, `romanisation`, or `english` changes after a
Rust-generated card exists.

**What breaks.** Accepted output is no longer current for that source row.

**Detection.** Compare `source_ref.fingerprint` against current source
fingerprint.

**Resolution.** Report `source changed`; do not write repair output in M2.

## Part 6 - Code Review Checklist

| Area | Reject | Accept |
|---|---|---|
| Read-only behavior | Any data write from planner command. | Planner derives and prints only. |
| Missing lineage | Treated as done. | Reported separately. |
| Source changed | Ignored fingerprint mismatch. | Reported separately. |
| Batch target | Hard-coded batch names or collision-prone names. | Next unused zero-padded batch per source stem. |
| CLI | `--max-batches` parsed as item count. | Limits output files. |
| Tests | Only happy path. | Fixtures for missing lineage, done, changed, pending, filename selection. |

## Appendix - Files Removed Or Moved

No files are removed or moved in this spec.

## Appendix - Out-Of-Scope Residue

- Full accepted-output schema validation remains M3.
- Old Python-generated output stays lineage-less.
