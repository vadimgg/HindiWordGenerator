# Agent Translation Experiments

Agent translation experiments are produced separately from local Ollama runs so
we can compare local models with a no-context agent baseline.

Results live under:

```text
experiments/agent_translation/results/<model>/<experiment_id>/
```

Each summary follows the same high-level shape as the Ollama experiment summary:
`model`, `experiment_id`, `summary`, `by_test`, and `results`.

Normalize a raw no-context agent JSON response:

```bash
python3 experiments/agent_translation/write_agent_result.py experiments/agent_translation/agent_raw/<file>.json
```

Compare agent and Ollama summaries:

```bash
python3 experiments/translation_report.py experiments/agent_translation/results/<model>/<experiment_id> experiments/ollama_translation/results/<model>/<experiment_id>
```
