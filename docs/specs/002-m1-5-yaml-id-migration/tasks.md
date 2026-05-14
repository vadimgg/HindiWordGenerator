# Tasks

## Work Packages

| ID | Title | Agent Type | Status | Dependencies | Acceptance |
|---|---|---|---|---|---|
| WP01 | Finalize source ID contract and CLI plan | rust-reviewer | planned | none | AC01-AC13 |
| WP02 | Implement source ID migration domain | rust-engineer | planned | WP01 | AC01-AC09, AC12 |
| WP03 | Wire CLI migrate active YAML and update docs | rust-engineer | planned | WP02 | AC01-AC13 |
| WP04 | Review M1.5 migration safety | rust-reviewer | planned | WP03 | AC01-AC13 |

## Notes

- Keep `output/`, `audio/`, `runs/`, and `archive/python/legacy-input/`
  protected.
- Do not run `brief spec complete` until the user explicitly approves closeout.
