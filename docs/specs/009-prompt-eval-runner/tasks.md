# Tasks

## Work Packages

| ID | Title | Agent Type | Status | Dependencies | Acceptance |
|---|---|---|---|---|---|
| WP01 | Add eval CLI and template context | rust-engineer | planned | none | AC01, AC03, AC04, AC05 |
| WP02 | Run eval through Ollama and write artifacts | rust-engineer | planned | WP01 | AC02, AC06, AC07, AC09 |
| WP03 | Seed sentence eval prompts, grading prompts, and smoke test | rust-engineer | planned | WP01, WP02 | AC08, AC10, AC11 |

## Notes

- Add work packages with `brief task new --name "Task name"`.
- Keep this file as an index. Detailed scope, validation, and boundaries belong
  in `tasks/WP*.md`.
