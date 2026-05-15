# Tasks

## Work Packages

| ID | Title | Agent Type | Status | Dependencies | Acceptance |
|---|---|---|---|---|---|
| WP01 | Review viewer export contract | rust-engineer | done | none | AC01-AC09 |
| WP02 | Implement viewer command | rust-engineer | done | WP01 | AC01-AC02 |
| WP03 | Build export data selection | rust-engineer | done | WP01 | AC03-AC07 |
| WP04 | Implement CLI export artifact | rust-engineer | done | WP03 | AC03-AC07 |
| WP05 | Add end to end smoke checks | rust-engineer | done | WP02, WP04 | AC08-AC09 |
| WP06 | Review viewer export parity | rust-reviewer | done | WP05 | AC01-AC09 |

## Notes

- Keep `hindi viewer` a wrapper around the existing Astro app.
- Keep Rust export scripted and file-based; do not add live AnkiConnect.
- Do not mutate broad real `output/` or `audio/` during automated tests.
