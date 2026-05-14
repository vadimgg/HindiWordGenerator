# Tasks

## Work Packages

| ID | Title | Agent Type | Status | Dependencies | Acceptance |
|---|---|---|---|---|---|
| WP01 | Create Rust CLI skeleton and doctor command | rust-engineer | done | none | `cargo run -- doctor` prints the read-only report. |
| WP02 | Add test coverage for M1 behavior | rust-engineer | done | WP01 | `cargo test` covers root discovery, required/optional checks, CLI surface, and Ollama seam. |
| WP03 | Review M1 implementation and docs alignment | rust-reviewer | done | WP01, WP02 | Review confirms scope, data safety, validation, and docs alignment. |

## Notes

- Keep this file as an index. Detailed work-package scope lives in
  `tasks/WP*.md`.
- Do not mark work packages done until implementation and validation are
  complete.
