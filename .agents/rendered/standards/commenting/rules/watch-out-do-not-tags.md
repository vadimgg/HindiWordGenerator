# Watch-Out And Do-Not Tags

Use `@watch-out` for non-obvious traps and `@do-not` for hard local rules.

## Applies When

- code has a tempting wrong change
- behavior depends on a subtle invariant
- future edits could silently break workflow state

## Rule

- `@watch-out` explains the trap.
- `@do-not` states a prohibition that prevents subtle bugs.

## Good

```rust
/// @watch-out Markdown output filters low-signal symbols; JSON output must keep them.
/// @do-not Move renderer filtering into the parser.
```
