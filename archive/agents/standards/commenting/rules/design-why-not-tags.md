# Design And Why-Not Tags

Use `@design` to preserve an architectural decision. Use `@why-not` to record a
rejected alternative that future agents are likely to suggest again.

## Applies When

- code intentionally uses a non-obvious pattern
- a simpler-looking option was rejected
- deterministic output, layering, or performance depends on a choice
- future refactors are likely to "clean up" something deliberately shaped

## Rule

`@design` says what pattern is used and why.

`@why-not` says what was considered and why it was rejected.

## Bad

```rust
/// @design This is a parser.
```

## Good

```rust
/// @design Builds a structured index first; Markdown and JSON are renderings of that index.
```

```rust
/// @why-not Avoided AST parsing for now to keep parser work dependency-light and language-portable.
```
