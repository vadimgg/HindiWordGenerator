# Invariant Tags

Use `@invariant` to document state that must always be true.

## Applies When

- a struct or module maintains internal consistency
- cached values must match source data
- ids, indexes, or projections must stay synchronized
- violating the rule means corrupt state

## Rule

An invariant is stronger than a normal comment. It describes a condition that
must hold before and after operations.

## Bad

```rust
/// @invariant Stores tasks.
```

## Good

```rust
/// @invariant `tasks.md` is an index derived from `tasks/WP*.md`, never task authority.
```

```rust
/// @invariant Stats are derived from the files array, never maintained separately by callers.
```
