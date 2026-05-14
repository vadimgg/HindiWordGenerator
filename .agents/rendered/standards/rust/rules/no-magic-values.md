# No Magic Values

Meaningful paths, keys, thresholds, labels, and status strings belong in one
clear place.

## Applies When

- adding file or directory paths
- adding JSON keys
- adding status strings
- adding numeric thresholds
- adding hardcoded lists

## Rule

- File and directory paths live in shared path constants or path helpers.
- Numeric thresholds have named constants.
- JSON keys and status strings should be constants or typed fields.
- Hardcoded lists should be discovered dynamically or live in one registry.

## Bad

```rust
let path = root.join(".brief").join("code-map");
```

## Good

```rust
let path = paths::brief_code_map_dir(root);
```
