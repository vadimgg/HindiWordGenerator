# Behavior And Error-Handling Tags

Use `@behavior` and `@error-handling` when types do not make behavior and
failure modes clear enough.

## Applies When

- writing Python, JavaScript, Ruby, or non-strict TypeScript
- accepting loose dictionaries, JSON, or external data
- invalid input is skipped, logged, normalized, or partially accepted
- a function intentionally does not raise on malformed input

## Rule

`@behavior` describes the important steps the code performs.

`@error-handling` describes how failures are surfaced.

These tags are especially important in untyped or loosely typed languages. In
typed Rust code, prefer types and `Result` first; use these tags only when they
clarify behavior not visible from the signature.

## Good

```python
"""
@behavior
1. Validates incoming node shape; skips and logs if invalid.
2. Creates VisualObject in cache if not already present.
3. Updates position and label from incoming data.
"""
```

```python
"""
@error-handling Invalid nodes are skipped and logged. Never raises. Returns None on all paths.
"""
```
