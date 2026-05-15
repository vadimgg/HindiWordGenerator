# Tasks

## Work Packages

| ID | Title | Agent Type | Status | Dependencies | Acceptance |
|---|---|---|---|---|---|

| WP01 | Review M4 generation contract | plan-reviewer | done | none | AC01, AC02, AC03, AC04, AC05, AC06, AC07, AC08, AC09, AC10, AC11, AC12, AC13, AC14, AC15, AC16, AC17, AC18, AC19, AC20, AC21, AC22 |

| WP02 | Implement Ollama client and model readiness | rust-engineer | done | WP01 | AC05, AC06, AC07, AC08, AC09 |

| WP03 | Build enrichment prompt and response extraction | hindi-prompt-tuner | done | WP02 | AC10, AC11, AC12, AC13, AC14 |

| WP04 | Wire sentence generation pipeline | rust-engineer | done | WP03 | AC01, AC02, AC03, AC04, AC15, AC16, AC17, AC21, AC22 |

| WP05 | Add run reports and generation output UX | cli-ux-reviewer | done | WP04 | AC18, AC19, AC20 |

| WP06 | Review M4 generation safety | rust-reviewer | done | WP05 | AC01, AC02, AC03, AC04, AC05, AC06, AC07, AC08, AC09, AC10, AC11, AC12, AC13, AC14, AC15, AC16, AC17, AC18, AC19, AC20, AC21, AC22 |

## Notes

- Add work packages with `brief task new --name "Task name"`.
- Keep this file as an index. Detailed scope, validation, and boundaries belong
  in `tasks/WP*.md`.
