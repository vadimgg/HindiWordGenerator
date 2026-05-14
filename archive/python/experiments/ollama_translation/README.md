# Ollama Translation Experiments

This experiment checks whether a local Ollama model can translate the approved
Hindi sentence cards into English before we try to ask it for full card JSON.

It reads from `output/sentences/`, writes reports to `results/`, and never
modifies real card output.

The main harness loads prompt adapters from `adapters/`. The model is known only
to the harness; adapters only define the prompt and input payload shape.

The harness samples a small set of approved sentence cards, then runs every
adapter call one by one. The `--batch-size` option controls how many cards are
sampled from each saved output batch; it does not send a batch of sentences to
the model.

Each result JSON includes:

- `model`
- `input_sentence`
- `full_prompt`
- `input_payload`
- `timing.duration_seconds`
- `result`

Each summary report includes timing at three levels:

- `timing.wall_seconds`: real elapsed time for the whole run.
- `summary.model_call_seconds`: summed call time across all model tasks.
- `by_test.<test>.avg_model_call_seconds`: average time per sentence for that
  adapter test.

`normalized_match_count` is a strict comparison against the approved English:
the script lowercases text, normalizes punctuation/spacing, and then checks for
exact equality. It is useful for catching paraphrase/drift, but it is not a
semantic quality score. For `register_detection`, the same count compares the
generated register against the approved card register.

```bash
uv run experiments/ollama_translation/compare_translation.py --model ollama:gemma4:latest --type sentences --batch-size 5 --max-batches 4 --concurrency 1 --adapters all
```

Every run records `ollama_runtime`, with the raw `ollama ps` output, parsed
running models, the selected running model, and whether the requested Ollama
model appears to be running.

For `ollama:<model>` runs, the harness now checks `ollama ps` before making any
model calls. If the requested model is not loaded, it exits with a clear error
listing the requested model and any running models.

Each test has a timeout, defaulting to 180 seconds. The harness stops the run on
the first timeout so slow models do not waste a full adapter matrix. Use
`--keep-going-on-timeout` only when you intentionally want to collect failures
for every adapter.

Run one sentence through all adapters:

```bash
uv run experiments/ollama_translation/compare_translation.py --model ollama:gemma4:latest --type sentences --batch-size 1 --max-batches 1 --concurrency 1 --adapters all
```

Current adapters:

- `hindi_simple`: Hindi sentence to English and romanisation.
- `hindi_strict`: stricter Hindi sentence to English and romanisation.
- `hindi_word_breakdown`: Hindi sentence to English, romanisation, and word meanings.
- `register_detection`: Hindi sentence to `formal`, `neutral`, or `informal`.
- `hindi_gloss_guided`: Hindi sentence to gloss-guided final translation.
- `source_row_issue_detection`: source-row QA that flags known bad inputs, such as awkward English or romanisation mismatch.
- `source_row_simple`: `Hindi (romanisation);English` input row to clean output.
- `source_row_word_breakdown`: source-row input to clean output plus word meanings.

Each adapter result is written to:

```text
results/<model>/<experiment_id>/<experiment_id>_<test_name>_<sentence>.json
```

Each run also writes:

```text
results/<model>/<experiment_id>/<experiment_id>_summary.json
```

Build a compact comparison report from saved summaries:

```bash
uv run experiments/ollama_translation/report.py
uv run experiments/ollama_translation/report.py experiments/ollama_translation/results/<model>/<experiment_id>
```

For the full CLI report across Ollama and agent benchmarks, use:

```bash
python3 experiments/translation_report.py experiments/ollama_translation/results/<model>/<experiment_id> experiments/agent_translation/results/<model>/<experiment_id>
```

By default this prints:

- benchmark comparison
- evaluation glossary
- evaluator verdict table
- model summary
- final 1-5 score comparison by prompt experiment

Print evaluator verdicts recorded on result files:

```bash
python3 experiments/translation_report.py evaluations experiments/ollama_translation/results/<model>/<experiment_id>
```

Print the evaluation glossary with the verdict table:

```bash
python3 experiments/translation_report.py evaluations experiments/ollama_translation/results/<model>/<experiment_id> --glossary
```

Print evaluator bullet-point comments, sorted with the worst verdicts first:

```bash
python3 experiments/translation_report.py comments experiments/ollama_translation/results/<model>/<experiment_id>
```

Include comments in the default full report:

```bash
python3 experiments/translation_report.py experiments/ollama_translation/results/<model>/<experiment_id> --comments
```

Print only the benchmark comparison:

```bash
python3 experiments/translation_report.py compare experiments/ollama_translation/results/<model>/<experiment_id>
```

Use smaller limits while testing a new local model:

```bash
uv run experiments/ollama_translation/compare_translation.py --model ollama:gemma4:latest --type sentences --batch-size 1 --max-batches 1 --concurrency 1
```

Ollama context note: the harness sends one fresh request per test and does not
reuse chat history between calls. No explicit context-clear command is needed
for this API path.

After a run, build a packet for the Hindi language evaluator agent:

```bash
python3 experiments/translation_evaluation/build_evaluator_packet.py experiments/ollama_translation/results/<model>/<experiment_id>/<experiment_id>_summary.json
```
