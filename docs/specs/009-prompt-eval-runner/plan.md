# Plan

## Design

Implement `hindi eval` as a separate prompt workbench, not as part of sentence
generation. It reads YAML source, builds a small Handlebars context, sends the
rendered prompt to the one currently running Ollama model, then writes diagnostic
artifacts under `eval/`.

## Modules

| Module | Responsibility |
|---|---|
| `src/cli.rs` | Parse `hindi eval` options and help text. |
| `src/eval.rs` | Load input/prompt, select fields/items, render template, call Ollama, write eval artifacts, render report. |
| `src/ollama.rs` | Expose current running model lookup and reusable generation call if needed. |
| `src/main.rs` | Wire command execution and exit codes. |

## Operation Order

1. Parse `--input`, `--prompt`, optional `--fields`, and optional `--max-items`.
2. Discover project root and resolve paths.
3. Load the input YAML and parse top-level `title`, `subtitle`, `items`.
4. Select items and fields.
5. Check Ollama running models; require exactly one.
6. Render the Handlebars prompt with YAML-first context.
7. Send the rendered prompt to the selected model.
8. Write `eval/<run-id>/prompt.txt`, `response.txt`, `result.json`, and
   `summary.txt`.
9. Print selected model, timing, output folder, and next inspection command.

## Work Package Sequence

| WP | Purpose |
|---|---|
| WP01 | Add eval CLI shape and template context rendering. |
| WP02 | Add Ollama model detection, request execution, and eval artifact writes. |
| WP03 | Add sentence prompt templates and live smoke validation. |

## Risks

| Risk | Mitigation |
|---|---|
| Eval accidentally writes accepted output. | Keep all writes under ignored `eval/`; add tests. |
| Model choice is surprising. | Print model and source as `ollama ps`; fail when zero or multiple models are running. |
| Template context grows too fast. | Keep v1 variables small and documented; defer nested selectors. |
| YAML rendering is unstable. | Use a small helper and tests for selected item rendering. |

## Validation

- `cargo fmt --check`
- `cargo test eval`
- `cargo test cli`
- `make check`
- Live `cargo run -- eval ...` smoke test when one Ollama model is running.
