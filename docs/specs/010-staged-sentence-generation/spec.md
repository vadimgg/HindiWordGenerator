# Staged Sentence Generation

## Scope

Replace the current single-prompt sentence enrichment internals with staged
local-model generation for `hindi sentences generate --max-batches <n>`.
Generation should call focused built-in prompts for register, literal
translation, and word breakdown, merge those outputs with trusted YAML/planner
data, validate the resulting candidate batch, and write accepted output only
after validation succeeds.

## Problem

The direct generation path currently asks one prompt for the whole enrichment
payload. Our eval runs showed that focused prompts are more reliable:
`register`, `literal`, `word-breakdown`, and
`word-breakdown-from-translation` performed well, while full enrichment was
slower and sometimes failed token/word or register behavior. We should turn the
good eval evidence into the actual generation pipeline.

## Goals

- Keep the public command shape stable:
  `hindi sentences generate --max-batches <n>`.
- Use staged model calls for each planned batch:
  register, literal, and word breakdown from existing translation.
- Use the existing configured `sentence_generation` model for all stages.
- Reuse or share the built-in prompt text proven by `hindi eval`.
- Merge stage outputs by source item ID.
- Build candidate sentence JSON with Rust-owned source fields and lineage.
- Validate the merged candidate with the existing sentence validator before
  writing accepted output.
- Record per-stage prompt fingerprints, model timings, and validation results
  in sentence run reports.
- Preserve append-only accepted output behavior and existing failure recovery.

## Non-Goals

- Do not add new user-facing generation commands.
- Do not add `--model`, model switching, or `hindi models prepare`.
- Do not add source QA, source repair, review/accept, or regeneration flows.
- Do not use the full-enrichment prompt as the normal generation path.
- Do not change source YAML format.
- Do not change accepted sentence JSON schema beyond what M3 already allows.
- Do not implement model comparison analytics in this spec.

## Acceptance Criteria

| ID | Criteria |
|---|---|
| AC01 | `hindi sentences generate --max-batches <n>` still parses and prints the same high-level success/failure sections. |
| AC02 | Generation plans pending batches from YAML/output at runtime and exits before model calls when the planner reports source/output errors. |
| AC03 | Generation checks the configured `sentence_generation` model once before stage calls and uses that same model for all stages. |
| AC04 | For each planned batch, generation runs focused stages for register, literal, and word breakdown keyed by source ID. |
| AC05 | Rust copies trusted fields from YAML/planner data: title, subtitle, Hindi, romanisation, English, tags, `source_ref`, fingerprint, and target path. |
| AC06 | Stage outputs are merged by source ID; missing, duplicate, or extra item IDs fail the batch before accepted output is written. |
| AC07 | Merged output creates valid `literal`, `register`, `tokens`, `words`, and optional `anki_tags` fields for every sentence. |
| AC08 | Existing sentence validation remains the acceptance gate; no partially valid batch is written. |
| AC09 | Accepted output is written through the existing atomic writer and still refuses collisions. |
| AC10 | Run reports include stage-level metadata: prompt ID/path, prompt fingerprint, model, duration, and success/error for each stage. |
| AC11 | Failed stage, merge, or validation writes no accepted output and writes a failed run report with actionable recovery text. |
| AC12 | Unit/integration tests cover successful staged merge, missing stage item, stage parse error, validation failure, and no-write failure behavior. |
| AC13 | Active docs stop describing default generation as one full-enrichment prompt. |

## Architecture Notes

### Files And Folders Changed

- `src/sentence_generate.rs`
- `src/sentence_enrichment.rs` or replacement staged module(s)
- `src/run_report.rs`
- `src/eval_prompts/*.yaml.hbs` or shared prompt module if prompts are
  extracted
- `docs/DESIGN.md`
- `docs/ROADMAP.md`
- `docs/specs/010-staged-sentence-generation/*`

### Workflow State Touched

- Writes accepted sentence batches under `output/sentences/` only after
  validation.
- Writes run reports under `runs/sentences/`.
- Reads source YAML from `input/sentences/`.
- Reads prompt templates compiled into Rust or project prompt files, depending
  on the implementation decision in [architecture.md](architecture.md).

### External Effects And Reuse

- Calls local Ollama through the existing `SentenceModelClient`.
- Reuses `generation_plan`, `validate_sentence_batch`, `write_sentence_batch`,
  `write_sentence_run_report`, and source fingerprint helpers.
- Reuses prompt lessons from `hindi eval` without depending on ignored `eval/`
  artifacts.

## Testing Plan

### Unit Tests

- Render each staged prompt with multiple source rows.
- Parse each stage output from raw JSON/YAML and fenced response text if
  supported.
- Merge register/literal/word outputs into a candidate sentence batch.
- Reject missing, duplicate, and extra source IDs per stage.
- Reject punctuation tokens/words through the existing validator.
- Verify stage metadata appears in run report structs.

### Integration Tests

- Fake model client returns successful staged outputs and one accepted batch is
  written.
- Fake model client fails a stage and no accepted output is written.
- Fake model client returns invalid word breakdown and validation blocks the
  write.
- Planner error path still exits before model calls.

### Smoke Tests

- `cargo test`
- `make check`
- `hindi sentences plan --max-batches 1`
- `hindi sentences generate --max-batches 1` with a loaded Ollama model, only
  after the implementation is ready for real accepted output.

### Drift / Consistency Checks

- `rg "generation_prompt_sentences_enrichment.txt|full-enrichment|single-prompt" docs src`
- `rg "hindi sentences generate" docs README.md src`
- Verify run report stage prompt IDs match the staged prompts actually used.

### Not Covered In This Spec

- Full model-quality benchmarking is covered by `hindi eval`, not by this
  generation implementation spec.
- Source QA remains deferred until the basic generation path is reliable.

## Open Questions

- Should generation prompt templates be compiled into Rust like eval prompts,
  or remain project files for easier editing? Recommendation: compile staged
  generation prompts into Rust for versioned fingerprints, while leaving the old
  root prompt as archived/reference material.
