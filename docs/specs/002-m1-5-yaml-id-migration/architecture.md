# Architecture

## Part 1 - What Changed And Why

M1.5 adds a stable identity layer to active YAML source rows. The risk is data
drift: if IDs are assigned inconsistently or rewritten later, the Rust planner
will not be able to trust `source_ref.file + item_id` as the source handle.

The migration should therefore be conservative: validate every source file
first, prepare all edits in memory, then write only active YAML files. Existing
IDs are source authority once present and must not be regenerated.

## Part 2 - Module Ownership

| Module | Owns | Must never |
|---|---|---|
| `src/cli.rs` | Parse `source ids` subcommands and flags. | Parse YAML, allocate IDs, or decide write targets. |
| `src/source.rs` or `src/source_ids.rs` | Load source YAML, validate IDs, allocate missing IDs, render updated YAML. | Print user-facing output or read `std::env::args` directly. |
| `src/project.rs` | Discover the project root and resolve project-relative paths. | Know source ID rules. |
| `src/main.rs` | Call the CLI parser, dispatch to domain functions, print results, and set exit codes. | Contain migration business rules. |

### Review Rules

- Commands parse input and print typed results.
- Source ID rules live in one domain module.
- Existing IDs are never rewritten by normal migration.
- All validation happens before the first source YAML write.
- No code path writes accepted output, audio, runs, or archived CSV files.

## Part 3 - Command Internals

### `hindi source ids check`

What the user sees:

```text
Source IDs

Scope
  sentences  input/sentences/*.yaml
  words      input/words/*.yaml

Result
  files      13
  items      182
  missing    182
  duplicate  0
  malformed  0

Next
  cargo run -- source ids migrate
```

Internal sequence:

```text
src/main.rs
  parse args through src/cli.rs
  discover project root
  call source ID check with write=false

src/source_ids.rs
  discover active source YAML files
  parse files into source documents
  validate ID shape and duplicates
  count missing IDs
  return SourceIdReport

src/main.rs
  print SourceIdReport
  exit 0 when valid and complete
  exit 1 when migration is needed or blocking errors exist
```

### `hindi source ids migrate --check`

Dry-run migration. It computes the same edits as migration mode and prints what
would change, but writes nothing.

### `hindi source ids migrate`

What the user sees:

```text
Source ID Migration

Changed Files
  input/sentences/complete_hindi_chapter_02_sentences.yaml  added 13 ids
  input/words/complete_hindi_chapter_01_words.yaml          added 34 ids

Result
  files changed  13
  ids added      182

Next
  cargo run -- source ids check
```

Internal sequence:

```text
src/main.rs
  parse args
  discover project root
  call migration domain with dry_run flag

src/source_ids.rs
  discover active source YAML files
  parse all files
  validate existing IDs
  allocate missing IDs in file order
  render all changed files in memory
  point of no return
  write changed YAML files
  return SourceIdMigrationReport
```

Reject in review:

- Per-file writes before all files validate.
- ID allocation based on filename, title, or subtitle.
- Reassigning existing IDs because there is a gap.
- Silent repair of duplicate or malformed IDs.
- Any write outside `input/sentences/` and `input/words/`.

## Part 4 - Shared Abstractions

### Source ID Validator

Used by:
- `source ids check`
- `source ids migrate --check`
- `source ids migrate`
- later M2 planner code

Contract:
- Input: parsed source documents with project-relative paths.
- Output: missing ID count, malformed IDs, duplicate IDs by file, and complete
  valid source rows.
- Errors: malformed YAML or duplicate/malformed IDs.
- Never writes files.

### ID Allocator

Used by:
- `source ids migrate`

Contract:
- Input: existing valid IDs and source item order for one file.
- Output: missing IDs filled with next available zero-padded strings.
- Existing IDs are preserved exactly.
- Never generates IDs outside the file-local namespace.

## Part 5 - Data And Drift Risks

### Persistent Files

| File | Written By | Read By | Rule |
|---|---|---|---|
| `input/sentences/*.yaml` | `source ids migrate` | M2 planner, future generation | Source authority. Existing IDs are stable once committed. |
| `input/words/*.yaml` | `source ids migrate` | Future word planner/generation | Source authority. Existing IDs are stable once committed. |
| `docs/ROADMAP.md` | Humans/agents | Humans/agents | M1.5 status row should become done after migration lands. |
| `docs/specs/002-m1-5-yaml-id-migration/**` | Brief/humans/agents | Brief/humans/agents | Spec and task workflow context only. |

### Drift Scenario A - IDs Reassigned After Source Edits

**How it happens.** Migration treats IDs as positional and regenerates them
after a new item is inserted.

**What breaks.** Later planner output would think old accepted cards point to
different source rows.

**Detection.** Idempotency tests and a fixture where existing IDs have gaps.

**Resolution.** Preserve existing IDs and allocate only missing IDs.

> **Review flag.** Reject any implementation that clears or rewrites an
> existing `id`.

### Drift Scenario B - Output Backfill Sneaks Into Migration

**How it happens.** Migration tries to add `source_ref` to old generated JSON
while source IDs are being created.

**What breaks.** Python-era cards gain untrusted lineage and `output/` stops
being append-only learner data.

**Detection.** `git diff --name-only -- output audio runs archive/python/legacy-input`
must be empty.

**Resolution.** Leave old output lineage-less. M2 reports missing lineage.

> **Review flag.** Reject writes outside active YAML source files.

## Part 6 - Code Review Checklist

| Area | Reject | Accept |
|---|---|---|
| CLI layer | YAML parsing, path walking, or ID allocation in `cli.rs`. | Parse command/flags and dispatch typed requests. |
| Source module | Printing directly to stdout. | Typed report returned to `main.rs`. |
| ID stability | Existing IDs regenerated, renumbered, or normalized. | Existing IDs preserved exactly. |
| Write ordering | Writes one file before all source files validate. | Full validation first, write phase second. |
| Scope | Writes `output/`, `audio/`, `runs/`, or archived CSV files. | Writes only active source YAML files. |
| Errors | Duplicate/malformed IDs silently repaired. | Blocking error with file path and ID. |
| Tests | Only happy-path migration. | Allocation, duplicates, malformed IDs, dry-run, and idempotency. |

## Appendix - Files Removed Or Moved

No files are removed or moved in this spec.

## Appendix - Out-Of-Scope Residue

- Existing Python-era output has no `source_ref`; M2 planner should report this
  as `missing lineage`.
