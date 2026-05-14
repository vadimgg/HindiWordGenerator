# Tag Scope And Syntax

Choose tags by scope and use the comment syntax native to the language.

## Module Or File Level

Required:

- `@intent`

Recommended:

- `@design`
- `@watch-out`
- `@do-not`

Rust:

```rust
//! @intent Provide logical cursor navigation primitives.
//!
//! @design Works on logical buffer positions only.
//! @do-not Do not import from the renderer.
```

Python:

```python
"""
@intent Render graph nodes from dynamic external data.
@do-not Do not import from the graph storage layer.
"""
```

## Struct, Class, Or Type Level

Required:

- `@intent`

Recommended:

- `@invariant`
- `@ownership`
- `@watch-out`

## Function Or Method Level

Required:

- `@intent` for non-trivial functions
- `@contract` for untyped or loosely typed functions

Required when applicable:

- `@affects`
- `@watch-out`
- `@do-not`

Recommended when applicable:

- `@contract`
- `@design`
- `@why-not`
- `@behavior`
- `@error-handling`

## Syntax Reference

Rust items:

```rust
/// @intent ...
///
/// @affects ...
pub fn name() {}
```

Go:

```go
// @intent ...
//
// @watch-out ...
func Name() {}
```

TypeScript:

```typescript
/** @intent ... */
function name() {}
```
