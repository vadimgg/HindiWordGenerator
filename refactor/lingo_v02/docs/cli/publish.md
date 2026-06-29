# `lingo publish`

Export sentences for use elsewhere. One verb, four formats: `package` (lossless
JSON backup / re-import), `study` (app-shaped SQLite for the iOS app), `anki`
(.apkg production cards), and `db` (a raw filtered copy of `library.db` for power
users). The first three serve two audiences — backup/interchange and study
targets; shapes are detailed in [`package-and-agents.md`](../package-and-agents.md).

See [`CLI.md`](../CLI.md) for the shared color/output legend.

## `--help`

```
Export a deck or the library: package | study | anki | db.

Usage: lingo publish [OPTIONS] [DECK]

Arguments:
  [DECK]  Publish only this deck (default scope depends on format; see below)

Options:
      --format <FORMAT>   package | study | anki | db  [default: package]
      --dest <PATH>       Output path (default: out/<deck> or the configured dest)
      --force               Overwrite the destination if it already exists
      --include-unapproved  Include enriched-but-unapproved rows in study/anki
      --allow-unqa          Publish even if some sentences have not been QA'd
      --json                Machine-readable output
  -h, --help                Print help
```

Help colors: `publish`/flags **green**, `[DECK]`/`<FORMAT>`/`<PATH>` **yellow**,
headers **bold cyan**.

## Formats

| `--format` | Audience | Default scope | Default selection | Missing audio | Round-trip |
|---|---|---|---|---|---|
| `package` *(default)* | backup, agents, re-import | the deck (or library) | **all rows** (lossless) | **included**, `"audio": null` | yes (`import`) |
| `study` | the custom iOS app | **whole library** | **approved only** | skipped + reported | one-way load |
| `anki` | Anki | the deck | **approved only** | skipped + reported | one-way |
| `db` | power users | the deck (or library) | all rows | included | n/a |

Key behaviors:

- **Approval is the gate for study targets.** `study` and `anki` export only
  **approved** enriched sentences by default — so only things you reviewed reach
  your phone / Anki. `--include-unapproved` additionally includes
  enriched-but-unapproved rows; `draft` rows are never studyable and are always
  excluded.
- **`package` and `db` are lossless** — they export *every* selected sentence and
  preserve approval, QA, field authority, tokens, audio metadata, and origin, so a
  backup→restore round-trips exactly. A sentence missing audio is exported with
  `"audio": null`, not dropped.
- **`study` defaults to the whole library**; `lingo publish <deck> --format study`
  exports one deck. It emits a loose folder: `study.sqlite` + an `audio/` sidecar,
  against a stable versioned schema.
- **`anki`** maps the deck to an Anki deck named `"<library title>::<subtitle>"`,
  one production card per sentence (English → target + audio + romanisation +
  breakdown), with the note GUID keyed to the sentence id so re-exports update
  cards instead of duplicating.
- **`db`** is a verbatim filtered copy of `library.db` for power users — *not* the
  app format; use `study` for the app.

## Scope is always shown

Because the default scope differs by format (`study` is the whole library, `anki`
and `package` are a single deck), every publish prints a `Scope` line so it's
never ambiguous what got exported:

```
Publishing   ch01 → anki
Scope        deck
Destination  out/ch01.apkg
```
```
Publishing   library → study
Scope        whole library
Destination  out/study/
```

## QA gate (warn-only)

QA is optional, so publish never hard-blocks. It does warn when study-facing
exports include sentences that were never QA'd:

- **`package`** and **`db`** — never gated (they're backups/interchange).
- **`study`** and **`anki`** — warn if any selected sentence has not been QA'd:

```
! 4 enriched sentences have not been QA'd.
  Continue with --allow-unqa, or run: lingo qa ch01
```

`--allow-unqa` proceeds anyway. (A stricter blocking mode may be added later, but
the default stays warn-only to match QA being optional.)

## Overwrite protection

If `--dest` exists, publish refuses unless `--force`:

```
! Destination already exists: out/study/
  Use --force to overwrite, or choose a different --dest.
```

## Examples

```bash
lingo publish ch01                                  # package → out/ch01/
lingo publish --format study --dest out/study       # whole library → app sqlite
lingo publish ch01 --format study --dest out/study  # one deck only
lingo publish ch01 --format anki --dest out/ch01.apkg
```

Sample output (`study`):

```
Publishing   library → study (sqlite)
Scope        whole library
Destination  out/study/

  ✓ ch01  Complete Hindi · Chapter 01    12 sentences · 12 audio
  ✓ ch02  Complete Hindi · Chapter 02     9 sentences ·  9 audio
  ~ ch02  skipped 1 sentence (missing audio)

21 sentences · 2 decks · 187 words
study.sqlite + audio/ written to out/study/

Next: lingo status
```

Colors: `Publishing`/`Scope`/`Destination` headings **bold cyan**; `✓` **green**;
`~` skip note **yellow**; deck slugs and counts **dim**; `Next:` label **yellow**,
command **cyan**.

## `Next:`

Usually `lingo status` (the pipeline is complete for this deck), or — if some
sentences were skipped for missing audio — `lingo audio` to fill the gap first.

## See also

[`import`](./import.md) · [`audio`](./audio.md) ·
[`package-and-agents.md`](../package-and-agents.md) · [`config`](./config.md)
(default destinations)
