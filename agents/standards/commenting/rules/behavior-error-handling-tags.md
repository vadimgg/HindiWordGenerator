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
Python code, prefer clear return shapes and exceptions first; use these tags
when runtime data, external JSON, CSV rows, or provider responses make behavior
non-obvious.

## Good

```python
"""
@behavior
1. Validates incoming sentence-card JSON; raises with the batch path if invalid.
2. Removes empty optional fields before writing.
3. Writes only after the full batch passes validation.
"""
```

```python
"""
@error-handling Invalid cards raise ValueError before any output file is written.
"""
```
