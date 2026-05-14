# Abstractions Over Conditionals

Prefer traits, registries, and focused implementations over growing conditional
dispatch.

## Applies When

- matching on language names
- matching on output formats
- branching on type-name strings
- adding a new behavior to a large dispatcher

## Rule

A `match` on a language string or type-name string is a design smell.

Adding a language, parser, renderer, or output format should usually require
adding a new implementation, not editing a large dispatcher.

Acceptable exceptions:

- a single central registry
- CLI enum dispatch
- small conversions at system boundaries

## Bad

```rust
match language {
    "rust" => parse_rust(path),
    "typescript" => parse_typescript(path),
    _ => parse_plain(path),
}
```

## Good

```rust
profile.parser().parse(path)
```
