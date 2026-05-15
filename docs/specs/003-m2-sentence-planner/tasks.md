# Tasks

## Work Packages

| ID | Title | Agent Type | Status | Dependencies | Acceptance |
|---|---|---|---|---|---|
| WP01 | Review planner contract and output shape | rust-reviewer | done | none | AC01-AC15 |
| WP02 | Implement sentence planner domain | rust-engineer | done | WP01 | AC02-AC12, AC14-AC15 |
| WP03 | Wire planner CLI and docs | rust-engineer | done | WP02 | AC01, AC09-AC14 |
| WP04 | Review planner read-only safety | rust-reviewer | done | WP03 | AC01-AC15 |

## Notes

- Planner must be read-only.
- Keep `input/`, `output/`, `audio/`, and `runs/` protected from writes.
- Do not run `brief spec complete` until the user explicitly approves closeout.
