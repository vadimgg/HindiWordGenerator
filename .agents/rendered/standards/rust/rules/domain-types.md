# Domain Types

Use typed values for concepts with structure, validation, or multiple
representations.

## Applies When

- parsing ids, slugs, branches, or status
- passing paths or counts across module boundaries
- representing workflow state

## Rule

- Paths use `PathBuf` or `&Path`, not `String`.
- States use enums, not string literals.
- Identifiers with validation rules use domain types.
- Parse and validate into domain types at boundaries.

## Bad

```rust
fn load_task(id: String) {}
```

## Good

```rust
fn load_task(id: &WorkPackageId) {}
```
