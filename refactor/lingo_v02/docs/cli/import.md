# `lingo import`

Merge an existing **new-format** Lingo package directory into this library.
Direct and immediate — no model, no run (unlike [`extract`](./extract.md), which
is the model path for *raw* material). This is not a prototype database migration
command.

See [`CLI.md`](../CLI.md) for the shared color/output legend.

## `--help`

```
Merge an existing lingo package into this library.

Usage: lingo import [OPTIONS] <PACKAGE>

Arguments:
  <PACKAGE>  A new-format lingo.package directory (from `lingo publish --format package`)

Options:
      --force            Re-import duplicate sentences and replace existing audio
      --trust-approval   Preserve approval/QA from the package (cross-library only)
      --json             Machine-readable output
  -h, --help             Print help
```

Help colors: `import`/flags **green**, `<PACKAGE>` **yellow**, headers **bold cyan**.

## Behavior

A package may contain **one deck or many**; its `manifest.json` records each deck's
slug, title, subtitle, and the source `library_id`, and each sentence file names
its deck. Import:

- **preserves deck slugs** from the package; on a slug collision it dedupes the
  *deck* (`ch01` → `ch01-2`) rather than merging into the wrong deck;
- **dedupes sentences** by normalized target identity within a deck — new ones are
  added with their audio copied to the flat internal path `audio/<sentence-id>.mp3`,
  duplicates are skipped;
- **records durable origin**: imported rows are marked `origin = imported` with the
  source library / package / sentence ids, so you can always tell imported
  sentences from ones you generated here (this survives `runs clean`);
- **allocates fresh local sentence ids** (the source id is kept only as provenance);
- prints a **per-deck summary**.

`--force` re-imports duplicates and replaces existing audio. It never deletes
non-duplicate sentences.

## Approval on import

Approval and QA do not blindly carry over from someone else's package —
otherwise an import could silently mark unreviewed content as "approved for study":

| Source | Approval / QA |
|---|---|
| **same library** (new-format package's `library_id` matches this library — i.e. a backup restore) | **preserved** |
| **different library** (a package from elsewhere) | **reset**: imported unapproved + QA `unchecked`, so you re-approve here |
| different library, with `--trust-approval` | preserved where the row is `enriched` |

A disaster-recovery restore into an empty library is a new-format restore flow
that seeds this library's `library_id` from the package first, so it counts as
"same library."

## Example

```bash
lingo import ./hindi-basics/
```

Sample output:

```
Package  ./hindi-basics/   (lingo.package/v2 · 2 decks)

Deck  hb   Hindi Basics
  + sen-hb-01  enriched  I was born in Lucknow.
      मैं लखनऊ में पैदा हुआ था।
      ♪ copied  audio/sen-hb-01.mp3
  ~ sen-hb-03  skipped   duplicate of sen-ch01-02
      hb:  6 added · 1 skipped · 6 audio copied · reset to unapproved

Deck  hb-proverbs   Hindi Basics · Proverbs
      hb-proverbs:  9 added · 0 skipped · 0 audio copied · 9 missing

15 sentences added · 1 skipped

Next: lingo audio
```

Colors: headings **bold cyan**; `+` **green**, `~` **yellow**; `♪ copied`
**green**, `♪ missing` **red**; ids and the "duplicate of …" trace **dim**;
`Next:` label **yellow**, command **cyan**.

## `Next:`

Points at the first gap in the imported material — usually `lingo audio` if any
sentence arrived without audio, otherwise `lingo status`.

## See also

[`publish`](./publish.md) (produces these packages) ·
[`package-and-agents.md`](../package-and-agents.md) · [`audio`](./audio.md)
