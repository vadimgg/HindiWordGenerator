# Tasks

## Work Packages

| ID | Title | Agent Type | Status | Dependencies | Acceptance |
|---|---|---|---|---|---|
| WP01 | Add staged prompt parsing and merge internals | rust-engineer | planned | none | AC04, AC05, AC06, AC07, AC12 |
| WP02 | Wire staged generation and run reports | rust-engineer | planned | WP01 | AC01, AC02, AC03, AC08, AC09, AC10, AC11, AC12 |
| WP03 | Validate staged generation and update docs | rust-engineer | planned | WP01, WP02 | AC12, AC13 |

## Notes

- Add work packages with `brief task new --name "Task name"`.
- Keep this file as an index. Detailed scope, validation, and boundaries belong
  in `tasks/WP*.md`.
- Implement work packages in order. WP02 depends on WP01's stage parser/merge
  contract; WP03 should not begin until the staged generator path exists.
