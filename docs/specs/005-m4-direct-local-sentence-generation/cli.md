# CLI And User Messages

## Purpose

M4 adds the first production generation command:

```bash
hindi sentences generate --max-batches 1
```

The command should be direct and non-interactive. It does not start Ollama or
switch models.

## Commands Touched

| Command | User Goal | Change | Side Effects |
|---|---|---|---|
| `hindi sentences generate --max-batches <n>` | Generate pending sentence cards with the configured local model. | New command. | May write `output/sentences/*.json` and `runs/sentences/*.json` after validation. |
| `hindi sentences plan --max-batches <n>` | Preview pending sentence cards. | Should remain compatible and read-only. | None. |
| `hindi --help` / `hindi sentences --help` | Discover commands. | Shows `sentences generate`. | None. |

## Help Text

| Command | Expected Help Change |
|---|---|
| `hindi --help` | Includes `hindi sentences generate --max-batches <n>`. |
| `hindi sentences --help` | Lists `plan` and `generate`. |

## Success Output

```text
Generate Sentences

Model
  configured        ollama:translategemma:12b
  provider          ollama
  ready             yes

Plan
  planned batches   1
  planned items     5
  target            output/sentences/complete_hindi_chapter_02_sentences_batch_05.json

Generation
  prompt            generation_prompt_sentences_enrichment.txt
  validation        ok
  accepted          1
  skipped           0
  run report        runs/sentences/20260515T093000Z_ollama_translategemma_12b.json

Next
  hindi sentences audio
```

## Progress And Log Messages

Keep output compact. M4 does not need streaming progress unless the model call
is slow enough that the command appears stuck. If progress is added, use one
line before the model call:

```text
Calling local model...
```

## Warning And Error Output

| Scenario | Expected Message | Recovery |
|---|---|---|
| Ollama API/model not ready | Shows `Model not ready`, needed model, and exact `ollama run <model>`. | User starts Ollama/model manually, then reruns generate. |
| Unsupported provider | Shows provider is unsupported and expected `ollama:<model>`. | Fix `hindi.toml`. |
| Planner errors | Shows planner problems and does not call model. | Fix source/output issue and rerun plan/generate. |
| Invalid model JSON | Shows failed validation/extraction and run report path. | Inspect run report, tune prompt/model, rerun. |
| Output collision | Shows target exists and run report path. | Rerun plan or remove unintended stale file manually. |

## Interactive Behavior

- Prompts: none.
- Non-interactive behavior: direct command, clear failure messages.
- Picker or fzf behavior: none.

## Color And Emphasis

No color requirement for M4. Keep plain section headings consistent with
existing Rust CLI output.

## UX Review Notes

The command should not hide the fact that Ollama must be started by the user.
Do not introduce `hindi models prepare` or automatic model switching here.
