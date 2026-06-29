# `lingo audio`

Generate spoken audio for approved sentences that don't have it yet. Internal files land at
`audio/<sentence-id>.mp3` — a flat, deterministic path keyed to the permanent
sentence id, so audio survives reordering, re-enrichment, and deck renames untouched
(no files to move). Exports may organize audio into per-deck folders for
readability; that's an export-local concern, not the authoring path.

See [`CLI.md`](../CLI.md) for the shared color/output legend.

## `--help`

```
Synthesize speech for approved sentences missing it.

Usage: lingo audio [OPTIONS] [DECK]

Arguments:
  [DECK]  Generate only for this deck (default: approved rows across the library)

Options:
      --force            Regenerate audio that already exists
      --backend <NAME>   TTS backend [default: gtts] [possible values: gtts]
      --include-unapproved
                         Also synthesize enriched-but-unapproved rows
      --json             Machine-readable output
  -h, --help             Print help
```

Help colors: `audio`/flags **green**, `[DECK]`/`<NAME>` **yellow**, headers **bold cyan**.

> **`elevenlabs` is a future backend.** It will appear in `--help` (with
> `--voice` / `--select-voice` / `--save-voice` options) only once it can list
> voices, select one, and synthesize. Until then `gtts` is the only value.

## What gets generated

By default `audio` synthesizes for **approved enriched** sentences that are
missing audio or whose audio is stale. Draft rows are never synthesized.
`--include-unapproved` additionally includes enriched-but-unapproved rows when
you intentionally want audio before curation.

Audio is stale when the spoken text (or backend/voice) no longer matches what
produced the existing file — e.g. after you edit a sentence's target. Staleness is
computed from a stored fingerprint, not a flag, so it's always accurate. `--force`
regenerates everything in scope regardless.

## Options

| Flag | Effect |
|---|---|
| `--force` | Re-synthesize every selected sentence, even fresh ones. |
| `--backend` | TTS engine. `gtts` is free and simple, but **networked** — see below. |
| `--include-unapproved` | Include enriched-but-unapproved rows. Draft rows are still excluded. |

> **`gtts` is free but not offline.** It uses Google Translate's text-to-speech
> endpoint, so it requires a network connection — unlike the rest of Lingo, which
> is fully local. A fully-offline backend may be added later; until then, audio is
> the one stage that reaches the network. (`elevenlabs`, when added, is also a
> networked API and needs a key.)

## Example

```bash
lingo audio ch01
```

Sample output:

```
Generating audio   (3 missing · 1 stale)

  ✓ sen-ch01-02  audio/sen-ch01-02.mp3
  ✓ sen-ch01-03  audio/sen-ch01-03.mp3
  ✓ sen-ch01-07  audio/sen-ch01-07.mp3   (was stale)
  ✓ sen-ch01-09  audio/sen-ch01-09.mp3

4 generated · 0 failed

Next: lingo publish ch01 --format study --dest out/study
```

Colors: heading **bold cyan**; `✓` **green**; ids and paths **dim**; failures
would show `!` **red**; `Next:` label **yellow**, command **cyan**.

## `Next:`

Once a deck has audio, points at publishing it. If other decks still lack audio,
may instead suggest `lingo audio` (all decks).

## See also

[`publish`](./publish.md) · [`status`](./status.md) · [`ls`](./ls.md)
(`--missing-audio`)
