# API Boundaries

Borrow at boundaries unless ownership is required.

## Applies When

- defining public or `pub(crate)` functions
- passing collections or paths
- designing domain APIs

## Rule

- Accept `&Path`, not `PathBuf`, unless taking ownership.
- Accept `&str`, not `String`, unless taking ownership.
- Accept `&[T]`, not `&Vec<T>`.
- Prefer borrowing over cloning.
- Keep visibility narrow: private, `pub(super)`, `pub(crate)`, then `pub`.

## Bad

```rust
pub fn render(paths: &Vec<PathBuf>) {}
```

## Good

```rust
pub fn render(paths: &[PathBuf]) {}
```
