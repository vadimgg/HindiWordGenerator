# Commenting Standard

Shared commenting rules for code-writing and review agents.

Load this file first, then load the focused rule files that match the task.

## High-Priority Rules

| Rule | Use when |
|---|---|
| [Intent Tags](rules/intent-tags.md) | creating modules, structs, or non-trivial functions |
| [Affects Tags](rules/affects-tags.md) | changing external state or writing files |
| [Watch-Out And Do-Not Tags](rules/watch-out-do-not-tags.md) | documenting non-obvious traps or hard local rules |
| [Contract Tags](rules/contract-tags.md) | documenting caller guarantees or state preconditions |
| [Invariant Tags](rules/invariant-tags.md) | documenting state that must always hold |
| [Design And Why-Not Tags](rules/design-why-not-tags.md) | preserving architectural choices and rejected alternatives |
| [Ownership Tags](rules/ownership-tags.md) | documenting ownership/lifetime/resource responsibility |
| [Data Surface Tags](rules/data-surface-tags.md) | documenting file reads/writes, generated views, and in-memory caches |
| [Behavior And Error-Handling Tags](rules/behavior-error-handling-tags.md) | documenting dynamic-language behavior or failure handling |
| [Tag Scope And Syntax](rules/tag-scope-syntax.md) | choosing tags by module/type/function scope or language syntax |

## Default Guidance

Comments explain what code cannot. Do not restate names, types, or obvious
control flow.

## Tag Set

Required for typed languages when applicable:

- `@intent`
- `@affects`
- `@watch-out`
- `@do-not`

Recommended for typed languages when they add information the type system cannot
express:

- `@contract`
- `@invariant`
- `@design`
- `@why-not`
- `@ownership`
- `@reads-file`
- `@writes-file`
- `@refreshes-file`
- `@cache`
- `@cache-source`
- `@cache-invalidation`

Required or strongly recommended for untyped and loosely typed languages:

- `@contract`
- `@behavior`
- `@error-handling`
