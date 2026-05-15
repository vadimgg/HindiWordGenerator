# Testing

## Drift This Must Prevent

- Eval writes accepted output.
- Prompt templates assume JSON when source input is YAML-first.
- `--fields` silently drops missing data.
- `--model` appears in docs or help before model switching exists.
- Multiple running Ollama models are accepted silently.

## Coverage Summary

| Change | Risk | Test | Purpose |
|---|---|---|---|
| CLI command | Bad args or stale help | `cargo test cli` | Confirms command shape. |
| Template context | Wrong prompt data | `cargo test eval` | Confirms `items`, `items_yaml`, `input_yaml`. |
| Field selection | Missing/incorrect fields | Unit tests | Confirms errors and defaults. |
| Artifact writes | Eval pollutes source/output | Integration test | Confirms writes stay under `eval/`. |
| Prompt templates | Examples drift | `rg` and smoke run | Confirms templates use `{{#each items}}`. |

## Unit Tests

- Parse `hindi eval` command and help.
- Build default selected fields.
- Build custom selected fields.
- Fail on missing selected field.
- Render a template with `{{#each items}}`.
- Render `items_yaml`.

## Integration Tests

- Fake model eval run writes expected files to an `eval/` folder.
- The same run writes nothing under `output/`.

## Drift Checks

- `rg --fixed-strings "--model" docs/specs/009-prompt-eval-runner src`
  should not show `hindi eval --model`.
- `rg "{{#each items}}" prompts/sentences`.

## Manual Review Checks

- Run one live eval against `register.yaml.hbs` with `--max-items 2`.
- Inspect `summary.txt`, `prompt.txt`, and `response.txt`.

## Not Covered

- Prompt quality ranking is deferred. This spec makes prompt runs structured and
  reproducible first.
