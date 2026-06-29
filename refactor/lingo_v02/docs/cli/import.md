# `lingo import`

Merge an existing Lingo package directory into this library. Direct and
immediate — no model, no run (unlike [`extract`](./extract.md), which is the
model path for *raw* material).

See [`CLI.md`](../CLI.md) for the shared color/output legend.

## `--help`

```
Merge an existing lingo package into this library.

Usage: lingo import [OPTIONS] <PACKAGE>

Arguments:
  <PACKAGE>  A lingo.package directory (the output of `lingo publish --format package`)

Options:
      --force    Re-import duplicate sentences and replace existing audio
      --json     Machine-readable output
  -h, --help     Print help
```

Help colors: `import`/flags **green**, `<PACKAGE>` **yellow**, headers **bold cyan**.

## Behavior

A package may contain **one deck or many**; its `manifest.json` records each deck's
slug, title, and subtitle, and each sentence file names its deck. Import:

- **preserves deck slugs** from the package; on a slug collision it dedupes the
  *deck* (`ch01` → `ch01-2`) rather than merging into the wrong deck;
- **dedupes sentences** by normalized target text within a deck — new ones are
  added with their audio copied, duplicates are skipped;
- infers status from field completeness (a fully-filled sentence imports as
  `enriched`, otherwise `draft`);
- prints a **per-deck summary**.

`--force` re-imports duplicates and replaces existing audio. It never deletes
non-duplicate sentences.

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
      ♪ copied  audio/hb/sen-hb-01.mp3
  ~ sen-hb-03  skipped   duplicate of sen-ch01-02
      hb:  6 added · 1 skipped · 6 audio copied

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
