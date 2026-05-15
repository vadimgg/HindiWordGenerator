# Plan

## Design

Implement `hindi eval input` / `hindi eval grade` as a separate prompt
workbench, not as part of sentence generation. `input` reads YAML source, builds
a small Handlebars context from a built-in prompt ID, sends the rendered prompt
to the one currently running Ollama model, then writes diagnostic artifacts
under `eval/`. `grade` loads an eval run, renders the paired grading prompt,
opens it in `$EDITOR`, and persists the pasted structured grader response.

## Modules

| Module | Responsibility |
|---|---|
| `src/cli.rs` | Parse `hindi eval input`, `hindi eval grade`, options, and help text. |
| `src/eval.rs` | Load input, resolve prompt IDs, select fields/items, render templates, call Ollama, write eval artifacts, render reports, parse grade responses. |
| `src/ollama.rs` | Expose current running model lookup and reusable generation call if needed. |
| `src/main.rs` | Wire command execution and exit codes. |

## Operation Order

1. Parse `hindi eval input --input`, `--prompt-id`, optional `--fields`, and
   optional `--max-items`.
2. Discover project root and resolve paths.
3. Load the input YAML and parse top-level `title`, `subtitle`, `items`.
4. Select items and fields.
5. Check Ollama running models; require exactly one.
6. Resolve the built-in input prompt by ID and render it with YAML-first
   Handlebars context.
7. Send the rendered prompt to the selected model.
8. Write `eval/<run-id>/prompt.txt`, `response.txt`, `result.json`, and
   `summary.txt`.
9. Print selected model, timing, output folder, and next inspection command.
10. For `hindi eval grade --run`, load `result.json`, resolve the paired grading
    prompt, render it, open it in `$EDITOR`, parse the saved response as YAML or
    JSON, then write `grade_prompt.txt`, `grade_response.txt`, `grade.json`, and
    update `summary.txt`.

## Work Package Sequence

| WP | Purpose |
|---|---|
| WP01 | Add eval CLI shape, built-in prompt IDs, and template context rendering. |
| WP02 | Add Ollama model detection, request execution, and eval artifact writes. |
| WP03 | Add paired sentence prompts, grading flow, and live smoke validation. |

## Risks

| Risk | Mitigation |
|---|---|
| Eval accidentally writes accepted output. | Keep all writes under ignored `eval/`; add tests. |
| Model choice is surprising. | Print model and source as `ollama ps`; fail when zero or multiple models are running. |
| Prompt IDs feel less flexible than paths. | Keep IDs stable and built in; add custom prompt paths later only if the built-in set proves too rigid. |
| Template context grows too fast. | Keep v1 variables small and documented; defer nested selectors. |
| YAML rendering is unstable. | Use a small helper and tests for selected item rendering. |
| `$EDITOR` flow is awkward in automation. | Keep grade import simple and file-based internally so a future non-interactive import can reuse it. |

## Validation

- `cargo fmt --check`
- `cargo test eval`
- `cargo test cli`
- `make check`
- Live `cargo run -- eval input ...` smoke test when one Ollama model is running.
