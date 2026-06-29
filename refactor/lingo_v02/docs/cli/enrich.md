# `lingo enrich`

Fill the study fields for `draft` sentences — romanisation, natural English,
literal gloss, register, and a word-by-word breakdown — via a model prompt.
Claims the sentences (status → `enriching`) and writes a task; the enrichment
lands when you [`apply`](./apply.md) the reply. Human-authored fields are never
overwritten.

See [`CLI.md`](../CLI.md) for the shared color/output legend.

## `--help`

```
Claim draft sentences for word-by-word enrichment → writes a task.

Usage: lingo enrich [OPTIONS] [DECK]

Arguments:
  [DECK]  Enrich only this deck (default: drafts across all decks)

Options:
      --limit <N>     Max sentences to claim for this prompt [default: 20]
      --force         Re-enrich already-enriched rows (still preserves human fields)
      --reset         Return stuck 'enriching' sentences to 'draft'
      --out <FILE>    Write the task to FILE (default: runs/<deck>-enrich-<id>/task.md)
      --print         Print the task to stdout and exit
      --watch         After writing the task, wait for the reply file and apply it
      --json          Machine-readable output
  -h, --help          Print help
```

Help colors: `enrich`/flags **green**, `[DECK]`/`<N>` **yellow**, headers **bold cyan**.

## Options

| Flag | Effect |
|---|---|
| `--limit` | Cap how many drafts are claimed — keeps the prompt inside the model's context window. |
| `--force` | Re-run enrichment on `enriched` rows (e.g. after improving the prompt). `authority: human` fields still untouched; clears `qa_checked_at`; if study-facing fields or tokens change, clears approval too. |
| `--reset` | Recovery: a crashed/abandoned run can leave rows in `enriching`. This returns them to `draft`. |
| `--print` / `--watch` | As in [`extract`](./extract.md). |

## Examples

```bash
lingo enrich ch01            # one deck
lingo enrich                 # up to --limit drafts across all decks
lingo enrich ch01 --force    # redo enrichment for ch01
lingo enrich --reset         # un-stick abandoned 'enriching' rows
```

Sample output (claiming a task):

```
Deck     ch01   Complete Hindi · Chapter 01
Run      ch01-enrich-9b2c
Claimed  12 draft sentences

  task   runs/ch01-enrich-9b2c/task.md
  reply  runs/ch01-enrich-9b2c/reply.json

Paste task.md into ChatGPT or Claude, save the reply, then:

Next: lingo apply runs/ch01-enrich-9b2c/
```

Sample output (`--reset`):

```
✓ Reset 9 sentences  enriching → draft

Next: lingo enrich ch01
```

Colors: headings **bold cyan**; `Claimed`/counts **green**; `✓` **green**; ids and
paths **dim**; `Next:` label **yellow**, command **cyan**.

## `Next:`

- After claiming: `Next: lingo apply runs/<run>/`.
- After `--reset`: points back at `lingo enrich <deck>`.

## See also

[`apply`](./apply.md) · [`qa`](./qa.md) · [`extract`](./extract.md) ·
[`status`](./status.md) · [`workflows.md`](../workflows.md)
