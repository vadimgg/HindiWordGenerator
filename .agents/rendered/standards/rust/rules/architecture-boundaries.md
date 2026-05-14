# Architecture Boundaries

Keep behavior in the owning domain and keep shared core boring.

## Applies When

- adding a command
- moving logic between modules
- reviewing a refactor

## Rule

Dependency direction:

```text
commands -> domains -> core
commands -> core, only for shared infrastructure
domains  -> core
```

- `src/commands/` parses arguments and dispatches.
- `src/specs/` owns spec lifecycle behavior.
- `src/tasks/` owns work-package and task lifecycle behavior.
- `crates/brief-codemap/` owns code-map and code-index behavior.
- `src/quality/` owns test, audit, and contract-promotion workflows.
- `src/vcs/` owns git and branch adapters.
- `src/documents/` owns Markdown, frontmatter, templates, and managed sections.
- `src/prompting/` owns prompt rendering and editor authoring flows.
- `src/standards/` owns coding standards and language profiles.
- `src/terminal_ui/` owns fzf and terminal selection helpers.
- `src/core/` owns boring shared primitives.

Task state lives in work-package frontmatter. Spec progress is derived from
facts such as current branch, task frontmatter, PR metadata, and merge evidence;
do not reintroduce status/event files or generated status views as workflow
authority.

Command modules should load config, call one domain operation, and print the
result. They should not own parsing, validation, lifecycle transitions, git
workflow decisions, or file mutation rules.

Domain modules can coordinate business rules and use infrastructure adapters.
When practical, decision logic should receive inventories, executors, clocks,
or git/filesystem adapters as inputs so it can be tested without live process
execution or machine state.

Keep direct filesystem and process calls close to infrastructure boundaries.
Planning and decision code should not secretly depend on Homebrew, uv, symlink
state, the live working tree, or other machine-specific state when a small
trait or input value would make the boundary explicit.

### Avoid Competing Authority Drift

Competing authority drift happens when two durable surfaces both appear to
answer the same question, such as "which change is active?", "which task is
current?", "what status is this item in?", or "which config value wins?".

Do not let two writable places independently own the same fact. Pick one
authority and make every other surface derived, cached, advisory, or explicitly
validated against the authority.

Good authority shape:

```text
current git branch is authority for active change when branch matches change/<NNN-slug>
docs/work/.current is a last-used pointer for non-change branches
```

Bad authority shape:

```text
current git branch says change/030-cleanup
docs/work/.current says 029-backlog
commands silently trust .current anyway
```

Reviewers should flag any design where:

- a pointer file, cache, generated view, branch name, database row, or
  frontmatter field can disagree with another surface about the same fact
- the design does not say who wins on conflict
- stale convenience data can silently override a stronger runtime authority
- callers can mutate convenience state without going through the owning domain

When a convenience pointer or cache is useful, its API must make the rule
explicit:

```text
if current branch is change/<change-ref>, active change comes from the branch
otherwise fall back to docs/work/.current
if both exist and disagree on a change branch, branch wins or the command stops
with clear guidance
```

### Prefer State Minimalism

State minimalism means persisting the smallest set of authoritative facts and
deriving everything else from those facts. Do not store a field, pointer, or
status just because it is convenient to display if the value can be reliably
derived from a stronger authority.

Derivable state duplication happens when code persists a fact that can already
be computed from existing information. That duplicated fact becomes another
surface that can drift.

Good state shape:

```text
PR merged state is derived from GitHub or git merge history
task completion is read from the WP file frontmatter
active spec is derived from the current spec branch when possible
```

Bad state shape:

```text
tasks/WP01.md says planned
GitHub PR is still open
another stored field says completed=true
```

Before adding persistent state, ask:

- What exact fact is this storing?
- Can that fact be derived from an existing authority?
- Is the source authority cheap and reliable enough to query or refresh from?
- If this duplicate value goes stale, how would we detect and repair it?
- Is this actually an authority, or should it be a generated view/cache?

Persist base facts and durable user intent. Generate views, labels, summaries,
next actions, and convenience pointers from those facts unless there is a clear
performance, offline, or audit reason to cache them.

New code should import the owning domain directly. Do not reintroduce removed
workflow modules or compatibility shims.

## Bad

Command module directly edits workflow files or parses Markdown.

## Good

Command module calls a domain function that owns the behavior.
