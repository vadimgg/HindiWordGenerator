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
/// @reads-file docs/work/.current
/// @contract Returns the active change pointer; callers must not parse `.current` directly.
pub fn load_active_change(...)
```

For dynamic paths, use the stable pattern:

```rust
/// @reads-file docs/work/<change>/tasks/T*.md
```

## File Writes

Use `@writes-file` when code directly writes durable state.

```rust
/// @writes-file data/followups.jsonl append-only
/// @affects Adds one follow-up JSONL record. Does not update workflow state.
pub fn append_followup_item(...)
```

If the write is destructive or rewrites a whole file, say so:

```rust
/// @writes-file data/followups.jsonl rewrite-preserve-order
/// @contract Preserves all entries except the selected status/timestamp fields.
pub fn write_followup_items(...)
```

## Generated Views

Use `@refreshes-file` when code regenerates a file from another authority.
Pair it with `@reads-file` and, when useful, `@invariant`.

```rust
/// @reads-file docs/work/<change>/tasks/T*.md
/// @refreshes-file docs/work/<change>/tasks.md
/// @invariant `tasks.md` is derived from work-package files, never task authority.
pub fn refresh_task_index(...)
```

Reviewers should treat multiple refreshers for one generated file as a drift
risk unless they all call one shared domain function.

## Caches

Use `@cache` when a type, module, or function owns in-memory data that survives
long enough to go stale, grow unexpectedly, or diverge from its source.

Do not tag short local collections used inside one obvious calculation.

```rust
/// @cache Stores parsed work packages for one command run.
/// @cache-source docs/work/<change>/tasks.md and docs/work/<change>/tasks/T*.md
/// @cache-invalidation Rebuild for each command invocation; never reuse across runs.
struct TaskIndexCache { ... }
```

For long-lived caches, include the invalidation rule:

```rust
/// @cache Stores project config by root path.
/// @cache-source .brief/config.json
/// @cache-invalidation Clear when config file mtime changes.
static CONFIG_CACHE: ...
```

## Protected Mutation

If cached or loaded data is mutable, expose domain operations instead of the
raw collection.

Good:

```rust
/// @cache Stores notes by id for this session.
/// @cache-source docs/backlog/notes.jsonl
/// @cache-invalidation Rebuild after add_note or delete_note_by_id.
struct NotesCache { ... }

impl NotesCache {
    fn delete_note_by_id(&mut self, id: NoteId) -> Result<()> { ... }
}
```

Bad:

```rust
/// @cache Stores notes.
pub fn notes_mut(&mut self) -> &mut HashMap<NoteId, Note>
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
