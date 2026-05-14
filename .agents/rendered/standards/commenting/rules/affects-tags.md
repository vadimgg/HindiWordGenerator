# Affects Tags

Use `@affects` when a function changes state outside itself.

## Applies When

- writing files
- running external commands
- mutating persistent state
- changing process-visible output

## Rule

Omit `@affects` on pure functions. Its absence signals purity.

## Good

```rust
/// @affects Writes the rendered code index to `.brief/code-index.json`.
```
