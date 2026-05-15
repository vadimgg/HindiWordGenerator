# Plan

## Design

Keep the validator strict and add a narrow normalization step after model JSON is
extracted and before candidate validation. The generator should also emit
timestamped progress lines around slow Ollama phases so a real run is visibly
alive.

## Modules

| Module | Responsibility |
|---|---|
| `src/sentence_enrichment.rs` | Merge trusted source fields and normalize model enrichment output before validation. |
| `src/sentence_generate.rs` | Orchestrate generation, print progress/timing, write accepted output/run reports. |
| `src/sentence_validate.rs` | Remains strict; punctuation tokens are still invalid here. |

## Operation Order

1. Load project root, config, prompt, and sentence plan.
2. Print planned batch/model information.
3. Check Ollama readiness and print elapsed time.
4. Send the prompt and print when the model response returns.
5. Extract model JSON, merge trusted source fields, and remove non-word token
   entries from candidate output.
6. Validate the normalized candidate batch.
7. If valid, write accepted output atomically.
8. Always write a run report.
9. Print accepted/failed status and the next command.

## Work Package Sequence

| WP | Purpose |
|---|---|
| WP01 | Normalize model token output while keeping validator strict. |
| WP02 | Add visible generation progress and timing output. |
| WP03 | Validate with full checks and one live Ollama smoke run. |

## Risks

| Risk | Mitigation |
|---|---|
| Cleanup hides bad model output. | Only remove `tokens[]` entries whose `kind` is not `word`; validation still checks word IDs, reconstruction, source coverage, and register. |
| Progress output becomes noisy. | Print one line per major phase, not per token or field. |
| Live Ollama run writes duplicate accepted output. | Keep collision refusal and all-or-nothing validation unchanged. |

## Validation

- `make check`
- `cargo run -- doctor`
- `cargo run -- source ids check`
- `cargo run -- sentences plan --max-batches 1`
- `cargo run -- sentences generate --max-batches 1`
