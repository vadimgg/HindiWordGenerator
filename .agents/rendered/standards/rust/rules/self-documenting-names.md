# Self-Documenting Names

Names should explain the outcome and side effects.

## Applies When

- naming functions
- naming booleans
- naming modules
- reviewing vague helpers

## Rule

Use names that describe what the caller gets or what changes.

Boolean functions should read as questions.

Functions with side effects should name the effect.

## Bad

```rust
handle()
process()
do_update()
get_data()
```

## Good

```rust
load_active_spec()
write_status_projection()
classify_code_file()
render_context_summary()
has_unresolved_placeholders(doc)
write_code_index(index, path)
```
