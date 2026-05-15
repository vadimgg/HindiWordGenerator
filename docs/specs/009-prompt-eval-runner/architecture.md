# Architecture

## How To Read This Document

This is a design audit, not a code tour. Read it looking for wrong module
ownership, duplicated abstractions, unclear write ordering, and data that could
silently drift.

If something here feels vague or uncomfortable, stop and make the design more
concrete before implementation. A bad architectural decision caught here costs
almost nothing. The same decision caught after implementation becomes a
refactor.

This document should answer five questions:

1. What changed and why?
2. Which module owns each part of the behavior?
3. What happens inside each user-facing command?
4. Which files persist, and how can they drift?
5. What should reviewers reject even if the code compiles?

## Part 1 - What Changed And Why

Write one plain-English summary from the user's initial problem statement. Name
the practical change first, then name the main architectural risk.

Keep this section big-picture. Detailed module rules come next.

## Part 2 - Module Ownership

Each module has one job. The `Must never` column is as important as the `Owns`
column. A violation is a design bug even when tests pass.

| Module | Owns | Must never |
|---|---|---|
| `src/commands/...` | Parse CLI args, build typed requests, call the owning domain, print typed results. | Own workflow rules, parse project files directly, or decide data authority. |
| `src/<domain>/...` | Own the business rules for this change. Replace this row with concrete modules after reading the code. | Print user-facing output or read CLI parser structs directly. |
| `src/documents/` | Markdown, frontmatter, template rendering, and managed document sections. | Own spec or task lifecycle decisions. |
| `src/core/` | Small cross-cutting primitives only. | Become a dumping ground for domain behavior. |

### Review Rules

- Commands parse input and print output.
- Domain modules own rules, validation, and state changes.
- Template files own generated document shape, not runtime logic.
- Work-package frontmatter owns task state.
- Generated views are safe to refresh and must not become authority.
- Old files should be deleted or backlogged, not moved without a real owner.

## Part 3 - Command Internals

Add one subsection for each user-facing command touched by this spec. If the
change has no command, replace this section with the relevant public entry point
or workflow.

### `brief example command`

What the user sees:

```text
brief example command --flag value

What Happened
  Summarize the visible result.

Next
  Show the next command or human action.
```

Internal sequence:

```text
src/commands/example.rs
  parse args
  build ExampleRequest
  call src/example/domain.rs

src/example/domain.rs
  validate all inputs and existing state
  prepare writes in memory
  point of no return
  write authority files
  refresh advisory indexes if needed
  return ExampleResult

src/commands/example.rs
  print ExampleResult
```

Abstraction requirements:

- The command layer should not contain domain rules.
- The domain layer should accept a typed request and return a typed result.
- All validation should happen before the first write.
- User-facing output should be built from the typed result.

Reject in review:

- Command modules containing slug rules, ID allocation, path building, or status
  routing.
- Domain modules calling `println!()` or reading clap parser structs.
- Template rendering inlined into a domain when a shared renderer should be
  used.
- Silent fallback to old, legacy, or undocumented data sources.
- Disk writes before all validation has passed.

## Part 4 - Shared Abstractions

Name shared helpers before implementation starts. If two modules need the same
logic, it should usually exist once with tests.

| Abstraction | Used By | Contract | Must Not |
|---|---|---|---|
| Name the helper once it is known. | List every caller. | State input, output, and error behavior. | State what would make the helper too broad or unsafe. |

Use this section for helpers such as template rendering, slug normalization,
active-spec resolution, ID allocation, parsing, frontmatter mutation, generated
index refresh, or output formatting.

For each important helper, add a short subsection:

```text
### helper_name

Used by:
- command or module A
- command or module B

Contract:
- input
- output
- errors
- what it never reads or writes

Review smell:
- the same logic appears inline in two modules
```

## Part 5 - Data And Drift Risks

List every file that persists between commands. If a command writes a file that
is not in this table, add it or treat the write as a design smell.

If this spec does not touch commands, workflow state, or persistent files, keep
this section short and write `Not touched in this spec` under the irrelevant
tables.

### Persistent Files

| File | Written By | Read By | Rule |
|---|---|---|---|
| `docs/specs/<spec>/tasks/WP*.md` | `src/tasks/` and humans | `src/tasks/`, `src/specs/`, humans, agents | Work-package frontmatter owns task state. |
| `docs/specs/<spec>/tasks.md` | `src/tasks/` and humans | humans and agents | Readable index. Refresh from work-package files; do not treat as authority. |
| Add every persistent file touched by this spec. | Name the module or command. | Name readers. | Say who wins on conflict or how drift is handled. |

### Drift Scenarios

#### Drift Scenario A - Name The Drift

**How it happens.** Explain the concrete way two files, generated views, or
modules can disagree.

**What breaks.** Explain the user-visible or implementation risk.

**Detection.** Name the function, test, command, or review check that catches
it.

**Resolution.** Say whether a command repairs it, refreshes a generated view, or
stops for human action.

> **Review flag.** Give reviewers one specific thing to look for in code.

## Part 6 - Code Review Checklist

Use this when reviewing implementation PRs. Each row should be concrete enough
to grep for or inspect directly.

| Area | Reject | Accept |
|---|---|---|
| Command layer | Business logic, file parsing, workflow decisions, or path construction. | Parse args, build typed request, call domain, print typed result. |
| Domain layer | `println!()` or direct clap types. | Typed request in, typed result out. |
| Template rendering | Two separate render implementations. | One named renderer with tests. |
| Write ordering | Validation and writes interleaved. | All validation first, then a clear write phase. |
| Error handling | Silent partial writes or vague recovery. | Clear message listing what changed and what to do next. |
| Data authority | Generated views or legacy projections treated as truth. | Authority comes from the file named in the data table. |
| Reuse | Same parser, renderer, normalizer, or resolver logic in two places. | One named helper with tests. |
| Legacy paths | Old folders read as fallback sources. | Old folders ignored, removed, or backlogged explicitly. |

## Appendix - Files Removed Or Moved

If this spec removes files, list them here with one reason each. If nothing is
removed, say so.

| File | Reason |
|---|---|
| Add removed path here. | Explain the former owner and the new owner, or say no current owner exists. |

## Appendix - Out-Of-Scope Residue

Record suspicious code, stale docs, or architecture smells found while planning
or implementing this spec. Do not fix them here unless they directly block the
work. Use `brief backlog add` for follow-up items that should survive the spec.
