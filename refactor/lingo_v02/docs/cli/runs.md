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
  clean [--abandoned] Remove run directories

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
      --abandoned   Also remove pending/failed runs AND any empty deck they created
```

- **`lingo runs clean`** (safe tidy-up): removes only **applied** run directories.
  Their work is already committed to the library, so nothing is lost.
- **`lingo runs clean --abandoned`**: also removes runs whose reply was never
  applied, and **garbage-collects any deck left with zero applied sentences**
  (e.g. an `extract` you started and never finished). This is the cleanup path for
  abandoned empty decks.

```bash
lingo runs clean              # remove finished runs
lingo runs clean --abandoned  # also drop never-applied runs + their empty decks
```

Sample output:

```
✓ Removed 1 applied run
~ Removed 2 abandoned runs · dropped 1 empty deck (ch03)

Next: lingo status
```

Colors: `✓` **green**, `~` **yellow**; `Next:` label **yellow**, command **cyan**.

## `Next:`

`ls` points at the oldest pending run to apply; `clean` points at `lingo status`.

## See also

[`apply`](./apply.md) · [`extract`](./extract.md) · [`deck`](./deck.md)
(`delete`) · [`package-and-agents.md`](../package-and-agents.md)
