# Intent Tags

Use `@intent` to explain why code exists.

## Applies When

- adding a module
- adding a struct or type
- adding a non-trivial function

## Rule

`@intent` should say what capability would be lost if this code disappeared.

## Bad

```rust
/// @intent Loads active spec.
pub fn load_active_spec() {}
```

## Good

```rust
/// @intent Resolve the current spec from branch or cache so commands share one active-spec rule.
pub fn load_active_spec() {}
```
