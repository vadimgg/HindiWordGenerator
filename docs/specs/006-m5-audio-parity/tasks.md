# Tasks

## Work Packages

| ID | Title | Agent Type | Status | Dependencies | Acceptance |
|---|---|---|---|---|---|
| WP02 | Review audio parity contract | rust-engineer | planned | none | AC01-AC11 |
| WP01 | Implement sentence audio scanner | rust-engineer | planned | WP02 | AC02-AC05 |
| WP03 | Add TTS backend boundary | rust-engineer | planned | WP01 | AC09-AC11 |
| WP04 | Write audio files safely | rust-engineer | planned | WP03 | AC04-AC06, AC09-AC10 |
| WP05 | Patch accepted JSON audio metadata | rust-engineer | planned | WP04 | AC03, AC07-AC10 |
| WP07 | Wire audio CLI and reports | rust-engineer | planned | WP05 | AC01-AC02, AC09 |
| WP06 | Review audio safety and parity | rust-reviewer | planned | WP07 | AC01-AC11 |

## Notes

- Follow the dependency order above. The numeric IDs are not perfectly ordered
  because the tasks were created quickly, but dependencies are the source of
  truth.
- Keep automated tests on temp fixtures. Do not run real audio generation
  against project `output/` or `audio/` unless the user explicitly asks.
