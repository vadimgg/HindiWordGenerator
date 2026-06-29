# `lingo deck`

Inspect and manage decks. Decks are created automatically by [`extract`](./extract.md)
and [`import`](./import.md) — this command lists, inspects, renames, and removes
them. (Internally a deck is a "batch" row; the CLI only ever says "deck".)

See [`CLI.md`](../CLI.md) for the shared color/output legend.

## `--help`

```
Inspect and manage decks.

Usage: lingo deck <SUBCOMMAND>

Subcommands:
  list                List all decks with counts
  show   <SLUG>       Show one deck and its sentences
  set    <SLUG>       Update title / subtitle / slug
  delete <SLUG>       Remove a deck (and its audio + pending runs)

Run `lingo deck <subcommand> --help` for details.
```

## `lingo deck list`

```
Decks
  ch01   Complete Hindi · Chapter 01   12 sentences   8 enriched · 4 draft   ♪ 4 missing
  ch02   Complete Hindi · Chapter 02    9 sentences   9 enriching            ♪ 9 missing
  hb     Hindi Basics                   6 sentences   6 enriched             ♪ 0 missing

Next: lingo deck show ch01
```

Colors: `Decks` heading **bold cyan**; slugs **dim**; status words colored by
status; `♪ N missing` **red**; `Next:` label **yellow**, command **cyan**.

## `lingo deck show <SLUG>`

Prints the deck header (slug, title, subtitle, source, created) followed by its
sentences in the standard block, and a summary line. Ends with a `Next:` reflecting
the deck's state (enrich / qa / audio / publish).

## `lingo deck set <SLUG>`

```
Usage: lingo deck set [OPTIONS] <SLUG>

Options:
      --title <TITLE>      Set the collection title
      --subtitle <TEXT>    Set the chapter/lesson label
      --slug <SLUG>        Rename the deck slug (must not already exist)
      --clear-title        Remove the title
      --clear-subtitle     Remove the subtitle
```

Updates only the fields you pass; leaves sentences, audio, and status untouched.
`--title`/`--clear-title` (and the subtitle pair) conflict. Example:

```bash
lingo deck set ch01 --title "Complete Hindi" --subtitle "Chapter 01"
lingo deck set ch01 --slug ch01-final
```

## `lingo deck delete <SLUG>`

```
Usage: lingo deck delete [OPTIONS] <SLUG>

Options:
      --force    Delete even if the deck has applied sentences
```

Removes the deck, its `audio/<slug>/` folder, and its pending runs. **Refuses** to
delete a deck that has applied sentences unless `--force` — so cleaning up an
abandoned empty deck is safe, but dropping real work is deliberate. (For sweeping
abandoned empty decks in bulk, see [`runs clean --abandoned`](./runs.md).)

```
! Deck ch01 has 12 sentences. Re-run with --force to delete them.
```

## `Next:`

`list`/`show` point at the next useful action for the deck; `set` re-prints the
deck summary; `delete` points at `lingo status`.

## See also

[`status`](./status.md) · [`extract`](./extract.md) · [`runs`](./runs.md) ·
[`edit`](./edit.md)
