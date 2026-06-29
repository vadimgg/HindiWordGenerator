# `lingo runs`

Manage the prompt/reply handoff directory. A **run** is one model exchange — a
folder under `runs/` plus a DB row (see
[`package-and-agents.md`](../package-and-agents.md)). Use this to see what's open
and to tidy up.

See [`CLI.md`](../CLI.md) for the shared color/output legend.

## `--help`

```
Manage the prompt/reply handoff directory.

Usage: lingo runs <SUBCOMMAND>

Subcommands:
  ls                  List runs and their status
  clean [--abandoned] Tidy run directories

Run `lingo runs <subcommand> --help` for details.
```

## `lingo runs ls`

```
Runs
  ✓ ch01-extract-7f3a   extract  ch01   applied   2026-06-29 10:32
  ● ch02-enrich-3a7f     enrich   ch02   pending   2026-06-29 11:05   ← reply awaited
  ✗ ch01-enrich-9b2c     enrich   ch01   failed    2026-06-29 10:58   ← fix reply & re-apply

Next: lingo apply runs/ch02-enrich-3a7f/
```

Colors: `Runs` heading **bold cyan**; `✓` applied **green**, `●` pending
**yellow**, `✗` failed **red**; run ids and timestamps **dim**; trailing notes
**dim**; `Next:` label **yellow**, command **cyan**.

Status comes from the DB (authoritative); the folder's `run.json` mirrors it.

## `lingo runs clean`

```
Usage: lingo runs clean [OPTIONS]

Options:
      --abandoned   Mark abandoned runs and remove only safe folders
```

- **`lingo runs clean`** (safe tidy-up): removes only **applied** run directories.
  Their work is already committed to the library, so nothing is lost.
- **`lingo runs clean --abandoned`**: marks abandoned pending/failed runs in the
  DB, then removes only run folders that have no reply file. A pending run with a
  reply file is never deleted silently; the output tells you to apply or inspect
  it. If an abandoned `extract` left an empty deck, the empty deck is dropped only
  when it has zero applied sentences and the report names it explicitly.

```bash
lingo runs clean              # remove finished run folders
lingo runs clean --abandoned  # mark abandoned runs; remove only safe folders
```

Sample output:

```
✓ Removed 1 applied run
~ Marked 2 abandoned runs · removed 1 empty run folder · dropped 1 empty deck (ch03)

Next: lingo status
```

Colors: `✓` **green**, `~` **yellow**; `Next:` label **yellow**, command **cyan**.

## `Next:`

`ls` points at the oldest pending run to apply; `clean` points at `lingo status`.

## See also

[`apply`](./apply.md) · [`extract`](./extract.md) · [`deck`](./deck.md)
(`delete`) · [`package-and-agents.md`](../package-and-agents.md)
