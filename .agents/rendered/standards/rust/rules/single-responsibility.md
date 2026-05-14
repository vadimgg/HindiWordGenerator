# Single Responsibility

Each function does one thing. Each module owns one concern. Each layer owns one
kind of decision.

## Applies When

- adding functions
- reviewing long functions
- deciding where behavior belongs
- splitting modules

## Rule

Single responsibility is not only about short code. It is about keeping reasons
for change separate.

Ask:

- What change would force this function to change?
- What change would force this module to change?
- Is that one reason, or several unrelated reasons?

If a function or module would change for unrelated reasons, split it.

## Function Responsibility

A function should have one primary outcome.

Signs a function is doing too much:

- the name contains `and`
- it has step comments inside the body
- it has more than two nesting levels
- it is longer than one comfortable screen
- it both decides what should happen and performs the side effect
- it parses input, validates business rules, renders output, and writes files
- tests for the function require many unrelated fixtures

Prefer a pipeline of focused functions:

```text
load -> parse -> validate -> transform -> render -> write
```

Do not force every helper to be tiny. A slightly longer function is acceptable
when it owns one coherent algorithm and splitting would hide the logic.

Split when each extracted function can have a meaningful name.

Avoid extraction when the helper would be vague:

```rust
fn handle_step_two(...) {}
fn process_inner(...) {}
```

Prefer extraction when the helper names a domain action:

```rust
fn parse_work_package_table(...) {}
fn validate_dependency_order(...) {}
fn render_task_summary(...) {}
```

## Module Responsibility

A module should own one concept or boundary.

Good module reasons:

- spec lifecycle
- work-package parsing
- dependency validation
- code-map rendering
- filesystem paths
- terminal selection UI

Weak module reasons:

- "helpers"
- "utils"
- "misc"
- "common"
- "stuff used by commands"

Use `mod.rs` to name the concept, and submodules to split responsibilities
inside that concept:

```text
src/tasks/
  mod.rs
  id.rs
  markdown.rs
  readiness.rs
```

The parent module should expose a small API. Submodules can hold the details.

## Layer Responsibility

For this project shape:

- command modules parse arguments and dispatch
- domain modules own behavior
- shared core owns infrastructure only
- models own pure data

Layer rules:

- commands should not parse Markdown, manipulate git state, or write task state
  directly
- domain modules can coordinate business rules and call shared infrastructure
- shared core should not know about specs, tasks, or code-map domain concepts
- models should not perform filesystem, process, or terminal I/O

## Side Effects

Keep decision logic separate from side effects when possible.

Good shape:

```rust
let updated = mark_work_package_done(package, now)?;
write_work_package_file(path, &updated)?;
```

This lets tests cover the decision without writing files.

When decision logic needs outside information, pass it in as data or a small
adapter/trait. Avoid burying live filesystem scans, process execution, git
queries, package-manager checks, or machine state inside code that should be
pure planning logic.

Side-effect functions should say what they affect in the name or comment:

```rust
write_work_package_file(...)
refresh_task_index(...)
render_code_map_to_dir(...)
```

## Review Checklist

Before accepting a function or module, ask:

- Can I state its job in one sentence without "and"?
- Does it have one reason to change?
- Is business logic outside `src/commands/`?
- Is shared infrastructure free of domain-specific decisions?
- Are parsing, validation, rendering, and writing separable?
- Can the decision logic be tested without filesystem/process side effects?
- Are external executors or inventories passed in rather than hidden inside the
  decision?
- Would a future agent know where to add the next related behavior?

## Bad

```rust
fn parse_and_render_and_write(...) {}
```

## Good

```rust
let index = parse_code(root)?;
let rendered = render_markdown(&index);
write_code_map(&rendered, output)?;
```

## Better Module Split

Bad:

```text
src/core/tasks.rs
```

Good:

```text
src/tasks/
  mod.rs
  id.rs
  markdown.rs
  readiness.rs
```

The good version gives each future change a natural home.

For brief, shared `core` is not a catch-all. If behavior knows about specs,
tasks, code maps, branches, quality gates, prompts, standards, terminal
selection, or removed workflow behavior, it should live in the owning domain
folder. `src/core/*` compatibility shims may remain during a migration, but new
behavior belongs in the domain.
