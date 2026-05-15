# Testing

## Drift This Must Prevent

- Generation target filenames drift from planner output.
- Model output overrides trusted source fields.
- Invalid enrichment writes accepted output.
- Ollama lifecycle management sneaks into the CLI.
- Run reports become required input for future planning.
- Real `input/` or `audio/` changes during generation tests.

## Coverage Summary

| Change | Risk | Test | Purpose |
|---|---|---|---|
| Generate CLI | Bad command shape or max-batches parsing. | CLI unit tests. | Keeps command grammar stable. |
| Config/model parsing | Wrong default or unsupported provider accepted. | Unit tests. | Keeps M4 one-model and provider-explicit. |
| Ollama readiness | Model calls happen when API/model unavailable. | Fake client tests. | Proves no accepted writes before readiness. |
| Prompt payload | Trusted fields sent as model-owned fields. | Prompt builder tests. | Preserves YAML authority. |
| JSON extraction | Model fences/prose break generation unnecessarily. | Extractor tests. | Handles realistic local model output. |
| Merge | Model overwrites source fields. | Merge tests with malicious extra fields. | Keeps trusted source fields trusted. |
| Pipeline | Validation failure writes output. | Fake invalid model integration test. | Protects accepted output. |
| Run report | Missing diagnostics after failure. | Report serialization tests. | Makes failed attempts inspectable. |

## Unit Tests

- `sentences generate --max-batches 1` parses.
- `sentences generate --max-batches 0` fails.
- Missing `hindi.toml` defaults to `ollama:translategemma:12b`.
- Explicit `[models].sentence_generation` is respected.
- `ollama:translategemma:12b` parses into provider/model.
- Unsupported provider fails with recovery text.
- Prompt payload includes `id`, `hindi`, `romanisation`, `english`, tags.
- Prompt payload excludes `source_ref`, output target, title, subtitle.
- Extractor accepts raw JSON.
- Extractor accepts fenced JSON.
- Extractor rejects no-JSON response.
- Merge ignores model-returned trusted fields.
- Invalid enrichment returns validation errors and no accepted write.

## Integration Tests

- Fake Ollama happy path writes one temp accepted output file and report.
- Fake Ollama not-ready path writes no accepted output.
- Fake Ollama invalid response writes failed report and no accepted output.
- Collision path refuses write and records skipped/failed report.

## Smoke Tests

- `cargo fmt`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo run -- sentences plan --max-batches 1`
- With Ollama running `translategemma:12b`:
  `cargo run -- sentences generate --max-batches 1`

## Drift Checks

- `git diff --name-only -- input audio`
- Confirm any `output/sentences/` diff came from an intentional successful
  generation smoke test.
- Confirm `runs/sentences/` contains only diagnostic reports.

## Manual Review Checks

- Command output tells the user the model, target, accepted/skipped writes, run
  report path, and next step.
- Failure output prints exact `ollama run <model>` recovery when appropriate.
- No code shells out to `ollama` in M4.

## Not Covered

- Translation quality scoring is not automated in M4.
- Audio and export are not covered.
- Source QA is not covered.
