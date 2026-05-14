# Contract Tags

Use `@contract` to document caller guarantees and return guarantees that the
type system cannot express.

## Applies When

- arguments must be valid in a relationship the type system cannot enforce
- calls must happen in a certain order
- function behavior depends on external state
- dynamic or loosely typed data crosses a boundary

## Rule

In typed languages, `@contract` is recommended when types are not enough.

In untyped or loosely typed languages, `@contract` is required for functions
with meaningful inputs.

## Bad

```rust
/// @contract Takes a path and returns text.
```

This only restates the signature.

## Good

```rust
/// @contract `project_root` must be an ancestor of parsed files for project-relative paths.
```

```python
"""
@contract node must be a dict with id (str), label (str), and position ({x, y}).
Returns None. Invalid nodes are skipped and logged.
"""
```
