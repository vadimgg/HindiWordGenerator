# ollama generation acceptance

## What We're Doing

Turn the user's short explanation into two or three plain-English paragraphs.
Start with the practical problem, then describe the change, then say why it
matters now.

Write this for someone who missed the conversation. They should understand the
shape of the change before opening the deeper docs. Avoid internal module names
unless they are necessary to understand the work.

## Why Now

Explain why this is worth doing now instead of later. Name the user pain,
workflow friction, data drift, or maintenance risk that makes the timing
important.

## What Changes

**Before:** Synthesize the current behavior in one sentence.

**After:** Synthesize the intended behavior in one sentence.

## What Stays The Same

- List only the most important boundaries that should not move.
- Mention state, command, data, or ownership rules that stay unchanged.
- Keep this short; [architecture.md](architecture.md) owns the details.

## What To Watch During Review

- Does the implementation put behavior in the module named in
  [architecture.md](architecture.md)?
- Does each command write only the files it is supposed to write?
- Does anything treat a generated view as the source of state?
- Did any old command, old template, or hidden fallback survive?

## Workflow Example

```text
brief spec new --name "ollama generation acceptance"
brief task new --name "First task"
brief task done WP01
brief spec complete
```

## Where To Read More

| If you want to understand... | Read... |
|---|---|
| Exact scope and what is intentionally excluded | [spec.md](spec.md) |
| Module ownership, command flow, and data drift risks | [architecture.md](architecture.md) |
| How this change will be proven safe | [testing.md](testing.md) |
| CLI commands, prompts, and user-facing output | [cli.md](cli.md) |
| Research findings and file-by-file audit notes | [research.md](research.md) |
| Implementation order and handoff strategy | [plan.md](plan.md) |
| Work packages | [tasks.md](tasks.md) |
