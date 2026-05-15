# Plan

## Design

Implement `hindi eval run` / `hindi eval grade` as a separate prompt
workbench, not as part of sentence generation. `run` reads YAML source, builds
a small Handlebars context from a built-in prompt ID, sends the rendered prompt
to the one currently running Ollama model, then writes diagnostic artifacts
under `eval/<prompt-id>/<run-id>/`. `grade` loads an eval run, renders the
paired grading prompt, opens an editor packet with a paste area, and persists
the pasted structured grader response.

## Modules

| Module | Responsibility |
|---|---|
| `src/cli.rs` | Parse `hindi eval run`, `hindi eval grade`, options, and help text. |
| `src/eval.rs` | Load input, resolve prompt IDs, select fields/items, render templates, call Ollama, write eval artifacts, render reports, parse grade responses. |
| `src/ollama.rs` | Expose `/api/ps` running model lookup and reusable generation call if needed. |
| `src/main.rs` | Wire command execution and exit codes. |

## Operation Order

1. Parse `hindi eval run <prompt-id> <input-yaml>`, optional `--fields`, and
   optional `--max-items`.
2. Discover project root and resolve paths.
3. Resolve the built-in prompt ID and validate its paired input/grading
   templates.
4. Load the input YAML and parse top-level `title`, `subtitle`, `items`.
5. Select items and fields.
6. Check Ollama `/api/ps` running models; require exactly one.
7. Render the built-in input prompt with YAML-first
   Handlebars context.
8. Send the rendered prompt to the selected model.
9. Write `eval/<prompt-id>/<run-id>/prompt.txt`, `response.txt`, `meta.json`,
   and `summary.txt`.
10. Print selected model, timing, output folder, and next inspection command.
11. For `hindi eval grade <run-id-or-path>`, resolve either an `eval/...` path or a
    prompt-scoped run ID. If the argument does not start with `eval/`, prepend
    `eval/` and resolve from the project root. Then load `meta.json`, render the
    paired grading prompt, and write `grade_prompt.txt` and `grade_packet.md`.
    If `--response <path>` is present, read the grader response from that file;
    otherwise open `grade_packet.md` in `$EDITOR` and extract the pasted
    response. Parse it as YAML or JSON, then write `grade_response.txt`,
    `grade.json`, and update `summary.txt`.

## Work Package Sequence

| WP | Purpose |
|---|---|
| WP01 | Add eval CLI shape, built-in prompt IDs, and template context rendering. |
| WP02 | Add Ollama model detection, request execution, and eval artifact writes. |
| WP03 | Add paired sentence prompts, grading schema/flow, and live smoke validation. |

## Risks

| Risk | Mitigation |
|---|---|
| Eval accidentally writes accepted output. | Keep all writes under ignored `eval/`; add tests. |
| Model choice is surprising. | Print model and source as Ollama `/api/ps`; fail when zero or multiple models are running. |
| Prompt IDs feel less flexible than paths. | Keep IDs stable and built in; add custom prompt paths later only if the built-in set proves too rigid. |
| Template context grows too fast. | Keep v1 variables small and documented; defer nested selectors. |
| YAML rendering is unstable. | Use a small helper and tests for selected item rendering. |
| `$EDITOR` flow is awkward in automation. | Keep grade import simple and file-based internally so a future non-interactive import can reuse it. |

## Validation

- `cargo fmt --check`
- `cargo test eval`
- `cargo test cli`
- `make check`
- Live `cargo run -- eval run ...` smoke test when one Ollama model is running.
