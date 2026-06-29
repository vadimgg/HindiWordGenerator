# `lingo extract`

Turn raw learning material into draft sentences, via a model prompt. Creates the
deck (empty, pending) and writes a prompt **task**; sentences appear only after
you [`apply`](./apply.md) the reply.

See [`CLI.md`](../CLI.md) for the shared color/output legend.

## `--help`

```
Start a deck from raw material → writes a prompt task.

Usage: lingo extract [OPTIONS] <RAW>

Arguments:
  <RAW>  Raw text file to segment into sentences (.md, .txt)

Options:
      --deck <SLUG>       Deck slug to file these under (default: derived from title/filename)
      --title <TITLE>     Collection title, e.g. "Complete Hindi"
      --subtitle <TEXT>   Chapter / lesson label, e.g. "Chapter 01"
      --out <FILE>        Write the task to FILE (default: runs/<deck>-extract-<id>/task.md)
      --print             Print the task to stdout and exit (writes nothing)
      --watch             After writing the task, wait for the reply file and apply it
      --json              Machine-readable output
  -h, --help              Print help
```

Help colors: `extract` and flags **green**, `<RAW>`/placeholders **yellow**,
headers **bold cyan**.

## Options

| Flag | Effect |
|---|---|
| `--deck` | Pin the slug. Omitted → derived from subtitle→title→filename, deduped `-2`. |
| `--title` / `--subtitle` | Deck display name. Optional; set later with [`deck set`](./deck.md). |
| `--out` | Custom task path. Default is under `runs/`. |
| `--print` | Dump the packet, write nothing (for piping / inspection). |
| `--watch` | Poll for the reply file and auto-`apply` (agent convenience). |

## Example

```bash
lingo extract raw/chapter01.md --deck ch01 \
  --title "Complete Hindi" --subtitle "Chapter 01"
```

Sample output:

```
Deck     ch01   Complete Hindi · Chapter 01   (new, empty — 0 sentences)
Run      ch01-extract-7f3a

  task   runs/ch01-extract-7f3a/task.md
  reply  runs/ch01-extract-7f3a/reply.yaml   (save the model's answer here)

Paste task.md into ChatGPT or Claude, save the reply, then:

Next: lingo apply runs/ch01-extract-7f3a/
```

Colors: `Deck`/`Run` headings **bold cyan**; slug, run id and paths **dim**;
`Next:` label **yellow**, command **cyan**.

### What the task asks the model

To segment raw text into clean sentences, strip page chrome, and — crucially —
**keep any translation/romanisation you already wrote, verbatim**, marking those
fields `authority: human` so later stages never overwrite them (the "uncle-ji"
guarantee). The reply is a single ` ```yaml ` fence with `format` + `sentences`.

## `Next:`

Always points at applying this run:

```
Next: lingo apply runs/ch01-extract-7f3a/
```

If you abandon the run without applying, clean it up with
`lingo runs clean --abandoned` (also removes the empty deck).

## See also

[`apply`](./apply.md) · [`enrich`](./enrich.md) · [`deck`](./deck.md) ·
[`import`](./import.md) (for already-built packages) · [`workflows.md`](../workflows.md)
