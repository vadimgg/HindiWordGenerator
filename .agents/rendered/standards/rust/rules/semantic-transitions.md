# Semantic Transitions

Expose operations named for business intent, not raw state mutation.

## Applies When

- changing lifecycle or status state
- implementing close, merge, approve, start, done, block, or cancel behavior
- syncing derived files such as `meta.json`, indexes, or dashboards
- designing APIs that could create drift if called out of order

## Rule

- Public APIs should be semantic transitions:
  `close_spec()`, `merge_spec()`, `complete_work_package()`.
- Do not expose public raw setters such as `set_spec_phase()` or
  `set_task_status()` when the state has rules or side effects.
- Keep mechanical writes private: update frontmatter, write metadata, refresh
  advisory indexes, update table rows.
- After a transition, call one shared sync path that derives current state from
  durable facts.
- Let facts drive status:
  work-package frontmatter, metadata, git state, PR metadata, and closeout
  artifacts.

## Why

Raw setters let callers skip the behavior that makes a state true. A spec should
not become `merged` because a caller wrote `"merged"` into `meta.json`; it
should become `merged` because git or PR evidence proves the merge.

## Bad

```rust
pub fn set_spec_phase(spec: &ActiveSpec, phase: SpecPhase) -> Result<()> {
    write_meta_phase(spec, phase)?;
    write_display_phase(spec, phase)
}
```

This API allows any caller to mark a spec as closed or merged without running
the close gate or git merge.

## Good

```rust
pub fn close_spec(root: &Path, request: CloseSpecRequest) -> Result<CloseSpecResult> {
    let spec = load_spec_for_close(root, request)?;
    ensure_all_tasks_done(&spec)?;
    write_closeout_docs(&spec)?;
    refresh_spec_indexes(&spec)
}

pub fn merge_spec(root: &Path, request: MergeSpecRequest) -> Result<MergeSpecResult> {
    let spec = load_spec_for_merge(root, request)?;
    ensure_spec_closed(&spec)?;
    ensure_clean_worktree(root)?;
    merge_spec_branch(root, &spec)
}
```

The public functions describe user intent. Private helpers perform mechanical
updates only after the transition succeeds.

## Review Checklist

- Can a caller force a lifecycle state without doing the required work?
- Are frontmatter updates, metadata writes, and advisory index refreshes centralized?
- Does failure stop before writing success state?
- Are human approvals or close gates represented by explicit durable facts?
- Are command modules calling semantic operations rather than mutating files?
