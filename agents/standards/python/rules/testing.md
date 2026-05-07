# Testing

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
