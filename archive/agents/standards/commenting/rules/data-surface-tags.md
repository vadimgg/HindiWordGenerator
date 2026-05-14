# Data Surface Tags

Use data surface tags when code reads, writes, refreshes, or caches data that
can drift from another surface.

These tags exist so code maps and reviewers can answer:

- Which functions read this file?
- Which functions write this file?
- Which files are generated views?
- Which in-memory caches can become stale?
- Is mutable data protected behind the owning module's functions?

## Tags

| Tag | Use when |
|---|---|
| `@reads-file` | Function or module directly reads a persistent file. |
| `@writes-file` | Function or module writes, appends, rewrites, or deletes a persistent file. |
| `@refreshes-file` | Function regenerates a generated view from an authority. |
| `@cache` | Type, module, or function owns in-memory data that can go stale or grow. |
| `@cache-source` | Cache is derived from a file, API, database, event log, or other authority. |
| `@cache-invalidation` | Cache is cleared, rebuilt, or made stale by a specific event or command. |

## Rule

Use these tags only at real data boundaries. Do not tag every thin wrapper or
every local scratch collection.

Prefer tagging the owning function or module that defines the data contract.
If a low-level filesystem helper only receives an arbitrary path, do not tag it
with every possible caller path.

## File Reads

Use `@reads-file` when code directly reads a known persistent file or file
family.

```rust
/// @reads-file docs/specs/.current
/// @contract Returns the active spec pointer; callers must not parse `.current` directly.
pub fn load_active_spec(...)
```

For dynamic paths, use the stable pattern:

```rust
/// @reads-file docs/specs/<spec>/status.events.jsonl
```

## File Writes

Use `@writes-file` when code directly writes durable state.

```rust
/// @writes-file docs/backlog/backlog.jsonl append-only
/// @affects Adds one backlog JSONL record. Does not update workflow state.
pub fn append_backlog_item(...)
```

If the write is destructive or rewrites a whole file, say so:

```rust
/// @writes-file docs/backlog/backlog.jsonl rewrite-preserve-order
/// @contract Preserves all entries except the selected status/timestamp fields.
pub fn write_backlog_items(...)
```

## Generated Views

Use `@refreshes-file` when code regenerates a file from another authority.
Pair it with `@reads-file` and, when useful, `@invariant`.

```rust
/// @reads-file docs/specs/<spec>/status.events.jsonl
/// @refreshes-file docs/specs/<spec>/status.json
/// @invariant `status.json` is derived from `status.events.jsonl`, never authority.
pub fn refresh_status(...)
```

Reviewers should treat multiple refreshers for one generated file as a drift
risk unless they all call one shared domain function.

## Caches

Use `@cache` when a type, module, or function owns in-memory data that survives
long enough to go stale, grow unexpectedly, or diverge from its source.

Do not tag short local collections used inside one obvious calculation.

```python
"""
@cache Stores parsed output batches for one command run.
@cache-source output/words/ and output/sentences/
@cache-invalidation Rebuild for each command invocation; never reuse across runs.
"""
```

For long-lived caches, include the invalidation rule:

```python
"""
@cache Stores provider config by project root.
@cache-source .env
@cache-invalidation Clear when config file mtime changes.
"""
```

## Protected Mutation

If cached or loaded data is mutable, expose domain operations instead of the
raw collection.

Good:

```python
"""
@cache Stores output batch summaries by stem for this command run.
@cache-source output/sentences/
@cache-invalidation Rebuild after write_batch().
"""
```

Bad:

```python
# @cache Stores output summaries.
def summaries_mut(): ...
```

The bad shape lets callers mutate data without validation, persistence, event
emission, projection refresh, or invariant checks.

## Relationship To Other Tags

- Use `@affects` for the human summary of the side effect.
- Use `@reads-file`, `@writes-file`, and `@refreshes-file` for searchable data
  surfaces.
- Use `@invariant` for the rule that must always hold.
- Use `@ownership` when one module owns mutation or lifecycle of a resource.

## Review Checklist

When these tags appear, reviewers should ask:

- Is there exactly one intended writer for durable state?
- If multiple functions write the same file, do they share one domain path?
- Is every generated view refreshable from its authority?
- Can callers mutate cached data directly?
- Is cache invalidation explicit enough to prevent stale reads?
- Do docs and code agree about which file is authority?
