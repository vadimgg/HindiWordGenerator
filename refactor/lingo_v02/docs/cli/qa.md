# `lingo qa`

Ask the model to review enriched sentences for mistakes and return corrections.
Writes a focused review **task**; applying its reply patches only the fields the
model flagged — and refuses to touch `authority: human` fields. Use it before
publishing, especially when an agent did the enrichment.

See [`CLI.md`](../CLI.md) for the shared color/output legend.

## `--help`

```
Ask the model to review enriched sentences → writes a task.

Usage: lingo qa [OPTIONS] [DECK]

Arguments:
  [DECK]  Review only this deck (default: all enriched sentences not yet QA'd)

Options:
      --limit <N>     Max sentences to review per task [default: 20]
      --recheck       Re-review sentences already marked QA'd
      --out <FILE>    Write the task to FILE (default: runs/<deck>-qa-<id>/task.md)
      --print         Print the task to stdout and exit
      --watch         After writing the task, wait for the reply file and apply it
      --json          Machine-readable output
  -h, --help          Print help
```

Help colors: `qa`/flags **green**, `[DECK]`/`<N>` **yellow**, headers **bold cyan**.

## What QA checks

The task gives the model a checklist: romanisation matches the target script; the
breakdown tokens cover the sentence in order; the literal gloss aligns with the
tokens; `register` is exactly one of `informal | standard | formal`; honorifics
and particles (e.g. "ji") are preserved; and human-authored fields are left alone.
The reply is **corrections keyed by sentence id** — only changed fields.

## QA state

QA is tracked by a nullable `qa_checked_at` field, separate from the lifecycle
status:

- `enriched` + not yet QA'd → this is what [`status`](./status.md) calls "needs QA".
- Applying a QA reply stamps `qa_checked_at` on every sentence in the run (a
  sentence with no corrections still counts as checked-and-clean).
- A later `enrich --force` clears `qa_checked_at`, sending rows back through QA.
- QA is **optional**: [`audio`](./audio.md) and [`publish`](./publish.md) work on
  `enriched` regardless; `status` just nudges you to QA first.

## Example

```bash
lingo qa ch01
lingo apply runs/ch01-qa-1d4e/
```

Sample output of the apply step (a before/after diff so you can sanity-check):

```
Deck   ch01   Complete Hindi · Chapter 01
Run    ch01-qa-1d4e

  ✓ sen-ch01-03  register  formal → standard
  ✓ sen-ch01-07  romanisation  "maĩ ne" → "maĩne"
  ~ sen-ch01-09  english  (rejected: field is human-authored)

2 corrections applied · 1 rejected · 12 checked

Next: lingo audio ch01
```

Colors: `✓` corrections **green**; `~` rejected **yellow**; field names and old
values **dim**; new values **cyan**; `Next:` label **yellow**, command **cyan**.

## `Next:`

After applying corrections, points at the next pipeline gap (usually
`lingo audio <deck>`, or `lingo publish` if audio is already done).

## See also

[`enrich`](./enrich.md) · [`apply`](./apply.md) · [`status`](./status.md) ·
[`workflows.md`](../workflows.md) (see "QA as a separate agent pass")
