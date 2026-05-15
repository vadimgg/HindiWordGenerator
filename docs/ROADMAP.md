# Roadmap

This is the active implementation plan. Older detailed planning drafts live in
`archive/docs/rust-planning/` and are reference material only.

## Current Status

| Area | Status |
|---|---|
| Python archived | Done |
| YAML source input | Done |
| Active Rust docs simplified | Done |
| Rust binary crate | Done |
| YAML item IDs migrated | Done |
| Sentence planner | Done |
| Validator and writer | Done |
| Local sentence generation | Pending |
| Audio parity | Pending |
| Viewer/export parity | Pending |

## M1: Rust CLI Skeleton

Goal: create one Rust binary crate and a tiny CLI that can inspect the project
without writing learner data.

Commands:

```bash
hindi doctor
```

Acceptance:

- One Rust binary crate exists with internal modules.
- `hindi doctor` reports project root, input/output/audio folders, built-in
  sentence prompt status, optional legacy prompt files, config file status, and
  Ollama reachability.
- No command writes accepted output.
- CLI output follows the Hindi display rule.

## M1.5: YAML ID Migration

Goal: give every source item a stable, file-scoped `id` so the planner can
identify pending vs done items by lineage rather than by position.

Acceptance:

- Every item in `input/sentences/*.yaml` and `input/words/*.yaml` has an `id`
  field.
- IDs are short opaque strings (e.g. `"0001"`), unique within their file.
- IDs are not unique across files; `source_ref.file + item_id` is the joint
  key.
- Existing item ordering is preserved during migration.
- Old generated JSON under `output/` is not modified; lineage is not
  backfilled onto Python-era cards.
- Migration is a one-off YAML edit (either a short script under
  `archive/scripts/` or a manual pass). Once IDs are assigned they are stable
  and committed to the repo; no command regenerates them.

## M2: Sentence Planner

Goal: read YAML sentence sources and existing output, then show pending work.

Commands:

```bash
hindi sentences plan --max-batches 1
```

Acceptance:

- Reads `input/sentences/*.yaml`.
- Reads existing `output/sentences/*.json`.
- Reports source validity, done, pending, deferred, missing lineage, source
  changed, and planned output filenames.
- Planned output filenames use the next unused zero-padded batch number for the
  source stem.
- `--max-batches` is total output files across the command invocation, not
  source items.
- Planner writes nothing.
- Source item IDs are parsed and validated.
- Items in existing output without `source_ref` are reported as
  `missing lineage`; the planner does not backfill.
- Existing YAML fixtures cover one completed item and one pending item.

## M3: Validator And Writer

Goal: validate candidate sentence batches and add safe writes.

Status: Done. Rust now has typed sentence candidate parsing, validation, atomic
accepted-output writer internals, and viewer `word_id` compatibility. The
writer is reusable infrastructure for M4; no normal CLI command writes accepted
sentence output yet.

Acceptance:

- Validates required sentence fields.
- Enforces register labels: `informal`, `standard`, `formal`.
- Enforces word-only `tokens` and `words`.
- Enforces token/word alignment.
- Enforces that every token references one `words[]` entry by `word_id` and
  every `words[]` entry is referenced by at least one token.
- The validator enforces `word_id` for new Rust output. Legacy `word_index`
  tolerance is a viewer concern, not a validator concern: the validator only
  ever inspects new candidate output, which must be `word_id` only.
- Validates `source_ref`.
- Enforces romanisation reconstruction against the `romanisation` string after
  Unicode NFC normalization; spacing and punctuation come from `romanisation`,
  not from the Hindi text.
- Writes through temp files and rename.
- Refuses accepted-output collisions.
- No partially valid batch is written.
- Viewer compatibility for `word_id` is implemented before any real Rust
  output is accepted; the viewer falls back to legacy `word_index` for
  Python-era output.

## M4: Direct Local Sentence Generation

Goal: generate one small sentence batch with the configured local model.

Command:

```bash
hindi sentences generate --max-batches 1
```

Acceptance:

- Checks Ollama readiness before model calls.
- If the configured model is not installed or reachable, prints the exact
  `ollama run ...` command and exits before spending model time.
- Calls the configured sentence model through focused enrichment stages:
  register, literal, and word breakdown from the existing translation.
- Rust copies source fields and lineage from YAML/planner data.
- Sends one structured source row per stage call and expects stage output keyed
  by source ID; extraction may tolerate markdown fences or leading/trailing
  prose before validation.
- Fails the batch before writing accepted output if any stage has missing,
  duplicate, or extra source IDs.
- Validates output before writing.
- Writes accepted output directly by default.
- Leaves a minimal run report under `runs/sentences/` with command, status,
  sources, targets, aggregate prompt fingerprint, per-stage prompt IDs,
  per-stage prompt fingerprints, model name and best available Ollama model
  metadata (digest when obtainable), timings, validation result, and
  accepted/skipped writes.
- Keeps the archived Python full-card prompt as reference only; normal Rust
  generation uses staged prompts rather than the old full-enrichment prompt.
- Viewer-compatible JSON remains under `output/sentences/`.
- Failed validation writes no accepted output and tells the user which run
  folder to inspect before rerunning generation.
- Viewer `word_id` compatibility is present before running M4 against real
  source, so the first new Rust card renders correctly.

## M5: Audio Parity

Goal: backfill sentence audio without regenerating cards.

Command:

```bash
hindi sentences audio
```

Acceptance:

- Scans accepted sentence JSON.
- Creates missing MP3s.
- Adds missing relative `audio` paths.
- Skips existing MP3s and existing audio fields by default.
- Uses temp paths before updating accepted JSON.
- Only adds missing `audio` metadata; it does not change learner content without
  a future explicit repair command.

## M6: Viewer And Export Parity

Goal: make Rust-generated output usable in the existing viewer and export flow.

Commands:

```bash
hindi viewer
hindi export --source "Complete Hindi" --topic "Chapter 02"
```

Acceptance:

- `hindi viewer` serves the Astro viewer, prints the local URL, and opens the
  browser by default.
- Viewer reads accepted output/audio.
- Viewer keeps the M3 `word_id` support and legacy `word_index` fallback while
  serving the full preview/export workflow.
- Refreshing the viewer shows newly generated batches.
- Export can produce a source/topic Anki artifact.
- Viewer export and CLI export share the same export contract once Rust owns
  export generation.

## Later

These are intentionally not in the first implementation path:

- `hindi sentences generate --review`
- `hindi sentences review <run>`
- `hindi sentences accept <run>`
- source QA
- source repair/regenerate commands
- word generation
- local Whisper transcription
- CLI-managed Ollama model switching

Add them after the direct sentence path is working and the workflow pain is
real, not theoretical.
