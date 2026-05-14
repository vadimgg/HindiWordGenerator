# Error Handling

Errors should explain what failed and preserve useful context.

## Applies When

- reading or writing files
- running processes
- parsing user input
- validating project state

## Rule

- No `unwrap()` in production code paths.
- Use `anyhow::Context` for filesystem and process errors.
- Return `Result` for expected failures.
- Do not silently swallow errors unless best-effort behavior is documented.
- Tests may use `unwrap()` when panic is the right failure signal.

## Bad

```rust
let text = std::fs::read_to_string(path).unwrap();
```

## Good

```rust
let text = std::fs::read_to_string(path)
    .with_context(|| format!("failed to read {}", path.display()))?;
```
