# Ownership Tags

Use `@ownership` to document who creates, owns, mutates, borrows, or releases a
resource when that responsibility is not obvious from the type signature.

## Applies When

- resource or file ownership is non-obvious
- references must not outlive source data
- cached or generated objects are owned by a specific component
- cleanup or release responsibility matters

## Rule

`@ownership` is recommended when files, generated data, caches, or external
resources have a non-obvious owner. Do not use it to restate simple variable
ownership.

## Bad

```python
# @ownership This function owns the local string.
```

## Good

```python
# @ownership writer owns output writes; callers must pass through validation.
```
