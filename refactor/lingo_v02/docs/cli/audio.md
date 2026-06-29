# `lingo audio`

Generate spoken audio for sentences that don't have it yet. Files land at
`audio/<deck-slug>/<sentence-id>.mp3` — id-based, so they survive reordering and
re-enrichment, and match the stable ids exports depend on.

See [`CLI.md`](../CLI.md) for the shared color/output legend.

## `--help`

```
Synthesize speech for sentences missing it.

Usage: lingo audio [OPTIONS] [DECK]

Arguments:
  [DECK]  Generate only for this deck (default: every sentence missing audio)

Options:
      --force            Regenerate audio that already exists
      --backend <NAME>   TTS backend [default: gtts] [possible values: gtts]
      --json             Machine-readable output
  -h, --help             Print help
```

Help colors: `audio`/flags **green**, `[DECK]`/`<NAME>` **yellow**, headers **bold cyan**.

> **`elevenlabs` is a future backend.** It will appear in `--help` (with
> `--voice` / `--select-voice` / `--save-voice` options) only once it can list
> voices, select one, and synthesize. Until then `gtts` is the only value.

## Options

| Flag | Effect |
|---|---|
| `--force` | Re-synthesize even where an mp3 already exists (overwrites in place). |
| `--backend` | TTS engine. `gtts` is free and simple, but **networked** — see below. |

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
Generating audio   (4 sentences missing)

  ✓ sen-ch01-02  audio/ch01/sen-ch01-02.mp3
  ✓ sen-ch01-03  audio/ch01/sen-ch01-03.mp3
  ✓ sen-ch01-07  audio/ch01/sen-ch01-07.mp3
  ✓ sen-ch01-09  audio/ch01/sen-ch01-09.mp3

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
