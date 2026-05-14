# Testing

Behavior changes need focused validation.

## Applies When

- changing behavior
- adding parsing or rendering logic
- fixing regressions

## Rule

- Add or update focused tests for behavior changes.
- Prefer descriptive test names.
- Keep tests close to the behavior unless integration coverage is needed.
- Run validation commands listed by the task.
- Report when validation cannot be run.

## Bad

Implementation changes with no test or validation note.

## Good

Implementation includes focused tests and reports `cargo test` results.
