# Tag Scope And Syntax

Choose tags by scope and use the comment syntax native to the language.

## Module Or File Level

Required:

- `@intent`

Recommended:

- `@design`
- `@watch-out`
- `@do-not`

Python:

```python
"""
@intent Plan pending generation batches from source CSV and existing output.
@do-not Treat manifest.json as the only source of completed cards.
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

TypeScript:

```typescript
/** @intent ... */
function name() {}
```

Python:

```python
def write_batch(...):
    """@intent Validate and write one generated batch."""
```
