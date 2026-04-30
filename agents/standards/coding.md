# Coding Standard

Python and CLI standards for this project, adapted from the Brief agent
standards.

## Architecture Boundaries

- Keep `main.py` focused on argument parsing, command dispatch, and readable CLI
  output.
- Keep planning, dedupe, validation, paths, writes, and manifest updates in
  `process.py`.
- Keep LLM provider construction, prompt loading, retries, concurrency, and
  generation orchestration in `generate.py`.
- Keep audio synthesis and `audio` path enrichment in `audio_generator.py`.
- Do not add vague `utils.py`, `helpers.py`, or `common.py` files. Split by
  ownership or side-effect boundary when a split is needed.

## File And Function Size

Size is a review trigger, not an automatic failure.

- Target functions: one comfortable screen, roughly 40 lines.
- Target files: roughly 200 lines when practical.
- Files around 300 lines need either a split plan or a short reason to stay
  together.
- Files over 500 lines are refactor candidates unless they are generated,
  data/config, or one cohesive algorithm with a documented reason.
- Keep nesting to two levels when practical.

Existing large files should not be churned just to satisfy the threshold. When
touching a large file, prefer extracting a coherent ownership slice if the task
already creates a natural boundary.

Current known large-file triggers:

- `process.py`: planning, parsing, validation, writing, and manifest behavior
  currently live together.
- `generate.py`: model setup, retries, orchestration, display, and persistence
  delegation currently live together.

## Single Responsibility

Functions and modules should have one reason to change.

Prefer this shape:

```text
load -> parse -> validate -> transform -> render -> write
```

Split when a function both decides what should happen and performs unrelated
side effects. Do not split when the extracted helper would have a vague name
like `handle_step`, `process_inner`, or `do_work`.

## Reuse Before New Helpers

Before adding a parser, path helper, batch helper, schema checker, or output
scanner:

- look for existing helpers in `process.py`
- check whether `main.py` already has display/planning helpers
- check whether `generate.py` already owns the orchestration concern
- check whether `audio_generator.py` already owns the audio concern

Only add a new abstraction when it reduces real duplication or clarifies an
ownership boundary.

## Constants And Magic Values

- Shared paths belong in path constants or path helpers.
- Repeated JSON keys, schema key sets, model defaults, retry limits, batch sizes,
  and output labels should live in one clear place.
- Avoid scattering raw strings for pipeline types, directories, and schema field
  names when a local registry or constant already exists.

## Names

- Use names that describe the result or side effect.
- Boolean helpers should read like questions.
- Side-effect functions should name the effect, such as `write_*`, `save_*`,
  `update_*`, or `attach_*`.
- Avoid vague names like `handle`, `process`, `do_update`, and `get_data` when a
  more specific name is available.

## Error Handling

Errors should explain what failed and preserve useful context.

- File, JSON, process, and validation failures should include the relevant path,
  stem, batch, or pipeline type.
- Do not silently swallow errors unless best-effort behavior is documented.
- Expected user-facing failures should produce actionable messages.
- For CLI output, include what failed, why when known, and the next concrete
  command or file to inspect when possible.

## CLI Output

CLI output should be explicit, calm, and actionable.

Good command output answers:

- what happened
- what was skipped or deferred
- what gaps or warnings exist
- what command to run next

Errors should be short but specific. Avoid noisy debug output unless `--verbose`
is enabled.

## Testing And Validation

Behavior changes need focused validation.

- Parser, planner, validation, and audio-path changes should have focused tests
  when practical.
- If tests do not exist yet, run the smallest meaningful command and report it.
- Good validation commands include:
  - `uv run main.py check`
  - `uv run main.py check --type words --max-batches 1`
  - `uv run main.py check --type sentences --max-batches 1`
  - `uv run main.py run --dry-run --max-batches 1`
- Report validation that could not be run and why.

## Comments

Comments should explain intent, contracts, behavior, or traps that are not
obvious from names and structure.

Use lightweight tags for non-trivial modules or functions when they help:

- `@intent`: what this module/function exists to do
- `@behavior`: important dynamic steps, especially for loose JSON or external data
- `@error-handling`: how failures are surfaced
- `@design`: why a non-obvious pattern exists
- `@why-not`: why a tempting alternative is intentionally avoided
- `@watch-out`: local trap future agents should not miss
- `@do-not`: hard project rule

Do not add comments that merely restate code.

