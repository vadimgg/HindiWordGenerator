# Ownership Tags

Use `@ownership` to document who creates, owns, mutates, borrows, or releases a
resource when that responsibility is not obvious from the type signature.

## Applies When

- Rust lifetimes or ownership are non-obvious
- Swift object ownership matters
- references must not outlive source data
- cached or generated objects are owned by a specific component
- cleanup or release responsibility matters

## Rule

`@ownership` is recommended for Rust and Swift when ownership is subtle. Do not
use it to restate obvious ownership from a simple signature.

## Bad

```rust
/// @ownership This function owns the String argument.
```

## Good

```rust
/// @ownership The index owns normalized path strings; renderers borrow them for output only.
```

```swift
/// @ownership RenderCache owns all VisualObject instances it creates.
///            Do not store references to source GraphNode data inside VisualObjects.
```
