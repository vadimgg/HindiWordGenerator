# Research

## Files

### `docs/ROADMAP.md`

#### R001 - M2 Contract Is Already Defined

Status: confirmed
Kind: implementation
Backlog: none
Confidence: high

What we saw:
- Roadmap says M2 reads sentence YAML and output JSON, reports source validity,
  done, pending, deferred, missing lineage, source changed, and planned output
  filenames.
- `--max-batches` is total output files across the command invocation.

Why it matters:
- The spec should implement this contract directly, not design a broader
  planner.

Recommended action:
- Keep M2 read-only and sentence-only.

### `output/sentences/`

#### R002 - Existing Output Is Python-Era Chapter 02 Batches

Status: confirmed
Kind: data
Backlog: none
Confidence: high

What we saw:
- Four existing output files:
  `complete_hindi_chapter_02_sentences_batch_01.json` through `batch_04.json`.

Why it matters:
- Planner filename selection should choose `batch_05` for the next Chapter 02
  output target.

Recommended action:
- Add filename-selection tests and a smoke expectation around next unused batch.

### `src/source_ids.rs`

#### R003 - Source YAML Parsing Exists But Is ID-Oriented

Status: confirmed
Kind: implementation
Backlog: none
Confidence: medium

What we saw:
- M1.5 source parsing can find active YAML items and validate IDs, but it was
  built for migration rather than planner source rows.

Why it matters:
- M2 should reuse or extract the useful source parsing rather than duplicating
  ID rules in a second module.

Recommended action:
- Keep source parsing helpers small and shared enough for planner use.

## Data Drift Themes Caught

- Missing lineage must not be hidden behind same-content matching.
- Source fingerprinting is the boundary between current and stale accepted
  output.

## Research Decisions

- The planner should not attempt to parse every word/token field in accepted
  output; full card validation is M3.
