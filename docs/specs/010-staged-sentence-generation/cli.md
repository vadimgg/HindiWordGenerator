# CLI And User Messages

## Commands Touched

| Command | User Goal | Change | Side Effects |
|---|---|---|---|
| `hindi sentences generate --max-batches <n>` | Generate pending sentence cards safely. | Internals change from one enrichment prompt to staged prompts. | Writes accepted output only after validation; writes run reports under `runs/sentences/`. |

## Help Text

No command shape change. Existing help should remain valid:

```text
hindi sentences generate --max-batches <n>
```

Do not add `--model`, `--stage`, `--prompt`, or source-QA flags in this spec.

## Success Output

The final output should stay compact:

```text
Generate Sentences

  model             ollama:gemma4:latest
  planned batches   1
  accepted batches  1

Accepted Output
  output/sentences/complete_hindi_chapter_02_sentences_batch_05.json

Run Reports
  runs/sentences/1778850000000_ollama_gemma4_latest.json

Next
  hindi sentences audio
```

Optional progress lines may mention stages while the model is running:

```text
planned 1 batch(es) in 4ms
checking model ollama:gemma4:latest
batch 1/1: stage register sending 3 item(s) to model
batch 1/1: stage register response received in 3.2s
batch 1/1: stage literal sending 3 item(s) to model
batch 1/1: stage literal response received in 2.1s
batch 1/1: stage word-breakdown-from-translation sending 3 item(s) to model
batch 1/1: stage word-breakdown-from-translation response received in 12.4s
batch 1/1: validating merged response
batch 1/1: accepted output written
```

## Progress And Log Messages

| Moment | Message | Notes |
|---|---|---|
| After planning | `planned <n> batch(es) in <time>` | Existing behavior; keep. |
| Before model readiness | `checking model <model>` | Existing behavior; keep. |
| Before each stage call | `batch <i>/<n>: stage <stage-id> sending <m> item(s) to model` | Print to stderr/progress stream. |
| After each stage call | `batch <i>/<n>: stage <stage-id> response received in <time>` | Helps identify slow stages. |
| Before validation | `batch <i>/<n>: validating merged response` | Stage merge has completed. |
| After accepted write | `batch <i>/<n>: accepted output written in <time>` | Existing style. |
| After failed report | `batch <i>/<n>: failed run report written in <time>` | Existing style. |

## Warning And Error Output

| Scenario | Expected Message | Recovery |
|---|---|---|
| Planner errors | `Planner found source/output problems.` | `hindi sentences plan --max-batches 1` |
| Model unavailable | `Configured Ollama model is not installed or reachable...` | Print exact `ollama run <model>` command. |
| Stage parse failure | `Stage sentence/register response could not be parsed.` | Inspect run report; rerun after prompt/model fix. |
| Missing stage item | `Stage sentence/literal did not return item 0003.` | Inspect run report; rerun after prompt/model fix. |
| Validation failure | Existing validator errors. | Inspect run report; fix prompt/model/source issue and rerun. |
| Output collision | Existing writer collision error. | Run planner and resolve output state. |

## Interactive Behavior

- Prompts: None.
- Non-interactive behavior: command exits non-zero on failure.
- Picker or fzf behavior: None.

## Color And Emphasis

No new color requirements. Keep existing plain text output scan-friendly.

## UX Review Notes

- Do not show raw model responses in normal command output.
- The user should be able to identify the slow or failing stage from progress
  lines and run report metadata.
- Recovery should stay one command: rerun `hindi sentences generate` after
  fixing the source/prompt/model issue.
