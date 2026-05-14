# Translation Quality Evaluation

The benchmark table measures exact output agreement, which is useful but too
strict for translation quality. The normal evaluation workflow batches one
model run into a compact packet, so the evaluator sees the rubric once and
returns one `items[]` list keyed by `result_file`.

Build a model-run packet from an experiment result directory:

```bash
python3 experiments/translation_evaluation/build_evaluator_packet.py experiments/ollama_translation/results/<model>/<experiment_id>
```

Find result files that still need evaluation:

```bash
python3 experiments/translation_evaluation/pending_evaluations.py experiments/ollama_translation/results/<model>/<experiment_id>
```

Generate one evaluator packet beside each pending result file:

```bash
python3 experiments/translation_evaluation/pending_evaluations.py \
  experiments/ollama_translation/results/<model>/<experiment_id> \
  --write-model-run-packet
```

By default, model-run packets include only result files missing
`evaluation.verdict`. Use `--include-evaluated` when intentionally re-reviewing
a run.

The model-run packet includes:

- model and experiment metadata once
- the evaluator output contract once
- one compact item per result file:
  - source sentence
  - reference card values
  - model result
  - prompt experiment name
  - source result file path

The Hindi evaluator must return raw JSON with `items[]`. Human-readable
findings go in each item's `bullet_points` list:

```json
{
  "verdict": "weak",
  "summary": "Romanisation is the main weakness.",
  "items": [
    {
      "verdict": "usable",
      "test_name": "source_row_simple_translation_romanisation",
      "result_file": "experiments/ollama_translation/results/...",
      "english_accuracy": 5,
      "natural_english": 5,
      "romanisation_accuracy": 3,
      "word_breakdown_accuracy": null,
      "register_accuracy": null,
      "learner_usefulness": 4,
      "issues": ["bad romanisation"],
      "bullet_points": [
        "English preserves the Hindi meaning.",
        "Romanisation is readable but misses nasalisation."
      ],
      "comment": "Usable for translation quality, not enough for final cards."
    }
  ]
}
```

Save that evaluator JSON and record it back into the summary and individual
result files:

```bash
python3 experiments/translation_evaluation/record_evaluation.py \
  experiments/ollama_translation/results/<model>/<experiment_id>/<experiment_id>_summary.json \
  experiments/ollama_translation/results/<model>/<experiment_id>/<experiment_id>_evaluation.json
```

This updates each per-test result JSON in place, adding its individual
`evaluation`, and also records the run-level verdict on the summary.

The scripts still support single-result packets for debugging one specific
test, but model-run batch evaluation is the default workflow.
