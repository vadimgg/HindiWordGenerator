# Rust Standard

Shared Rust rules for Rust engineers, reviewers, and managers reviewing Rust
scope or standards drift.

Load this file first, then load the focused rule files that match the task.

## High-Priority Rules

| Rule | Use when |
|---|---|
| [Domain Types](rules/domain-types.md) | identifiers, paths, status, branches, work-package ids |
| [Abstractions Over Conditionals](rules/abstractions-over-conditionals.md) | replacing language/type dispatch or growing match statements |
| [No Magic Values](rules/no-magic-values.md) | adding paths, labels, thresholds, keys, or status strings |
| [Single Responsibility](rules/single-responsibility.md) | splitting functions or placing behavior in modules |
| [Architecture Boundaries](rules/architecture-boundaries.md) | choosing modules or reviewing placement |
| [Self-Documenting Names](rules/self-documenting-names.md) | naming modules, functions, types, and booleans |
| [Error Handling](rules/error-handling.md) | filesystem, process, parsing, CLI errors |
| [API Boundaries](rules/api-boundaries.md) | function signatures and ownership |
| [Semantic Transitions](rules/semantic-transitions.md) | lifecycle/status updates, close/merge, task state changes |
| [File And Function Size](rules/file-function-size.md) | reviewing large files or functions |
| [Parser And Code Knowledge](rules/parser-code-knowledge.md) | parser, renderer, code-map work |
| [CLI Output](rules/cli-output.md) | command UX, errors, next steps |
| [Testing](rules/testing.md) | behavior changes and validation |

## Default Guidance

- Prefer typed domain values over raw strings.
- Keep command modules as argument parsing and dispatch.
- Keep shared core limited to infrastructure used by multiple domains.
- Keep functions and modules focused on one concern.
- Reuse existing helpers before adding new ones.
- Expose semantic lifecycle operations, not raw state setters.
- Add focused tests for behavior changes.
