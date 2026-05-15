# Tasks

## Work Packages

| ID | Title | Agent Type | Status | Dependencies | Acceptance |
|---|---|---|---|---|---|

| WP01 | Review validator and writer contract | plan-reviewer | done | none | AC01, AC02, AC03, AC04, AC05, AC06, AC07, AC08, AC09, AC10, AC11, AC12, AC13, AC14, AC15, AC16, AC17, AC18, AC19, AC20 |

| WP02 | Implement sentence schema and validator | rust-engineer | done | WP01 | AC01, AC02, AC03, AC04, AC05, AC06, AC07, AC08, AC09, AC10, AC11, AC12, AC13, AC14 |

| WP03 | Implement atomic accepted output writer | rust-engineer | done | WP02 | AC15, AC16, AC17, AC18 |

| WP04 | Add viewer word id compatibility | astro-viewer | done | WP03 | AC19, AC20 |

| WP05 | Review validation writer safety | rust-reviewer | done | WP04 | AC01, AC02, AC03, AC04, AC05, AC06, AC07, AC08, AC09, AC10, AC11, AC12, AC13, AC14, AC15, AC16, AC17, AC18, AC19, AC20 |

## Notes

- Add work packages with `brief task new --name "Task name"`.
- Keep this file as an index. Detailed scope, validation, and boundaries belong
  in `tasks/WP*.md`.
