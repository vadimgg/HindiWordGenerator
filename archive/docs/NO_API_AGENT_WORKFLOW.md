# No-API Agent Workflow

Use this workflow when an agent should generate a tiny test batch without using
provider credentials. The agent performs the enrichment, but project scripts
still own planning, validation, output paths, and writes.

## When To Use

- The user does not want to configure `OPENAI_API_KEY` or `ANTHROPIC_API_KEY`.
- The goal is a small test batch for the viewer, QA, or reviewer agents.
- The user explicitly allows replacing the current test output, or the planned
  batch is naturally pending.

## Workflow

1. Plan a small pending slice:

```bash
python3 process.py check --type sentences --batch-size 5
python3 process.py check --type words --batch-size 5
```

2. Use only one planned batch unless the user approves more.

The selected batch object must provide:

- `type`
- `stem`
- `batch_num`
- `total_batches`
- `count`
- `source`

3. Read the matching generation prompt:

```text
generation_prompt_sentences.txt
generation_prompt_words.txt
```

4. Enrich exactly the planned YAML `source` slice into raw JSON.

5. Write the raw JSON to a temporary file outside `output/`.

6. Validate and write through the project writer:

```bash
python3 process.py write sentences <stem> <batch_num> <total_batches> <count> /tmp/generated-sentences.json
python3 process.py write words <stem> <batch_num> <total_batches> <count> /tmp/generated-words.json
```

7. Backfill audio only after the JSON has passed validation:

```bash
uv run main.py audio output/sentences/<stem>_batch_<nn>.json
uv run main.py audio output/words/<stem>_batch_<nn>.json
```

8. Report the batch path, validation result, audio result, and any quality
concerns. Do not silently continue after validation failure.

## Guardrails

- Do not manually split YAML input.
- Do not write directly to `output/`.
- Do not use `main.py run` unless API-backed generation was requested.
- Do not use `--force` unless the user explicitly asks to regenerate existing
  cards.
- Do not edit source YAML files during generation unless the user separately
  approves an input repair.
- Do not replace existing output unless the user explicitly says test output may
  be replaced.

## Validation Commands

```bash
python3 scripts/check-python-contracts.py
cd viewer && npm run check
```
