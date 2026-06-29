# 08 · Import & export (interchange)

## 1. Model

- **SQLite (`library.db`) is the internal/runtime store** (doc 03) — organize,
  words, and audio status need queries and cheap mutation.
- **JSON is the portable interchange** — what we import from and export to, like
  today. Git-friendly, human-readable, consumable without linking SQLite.
- **Importing is "read JSON sentence data into the library."** Exporting is the
  inverse. They are symmetric over the same shapes.
- Exporting the **SQLite db itself** is just one optional export target (for a
  consumer like Grasp that prefers to link SQLite), not "the package."

```
              import (JSON/db → db)
 JSON  ──────────────────────────────▶  library.db (runtime)
 files ◀──────────────────────────────
              export (db → JSON)        ─┬─▶ JSON package   (default, like today)
                                         └─▶ library.db copy (optional, for Grasp)
```

JSON in, JSON out (like today); SQLite is how we work in between.

## 2. Canonical vs derived

At runtime the db is canonical (you can't reorder 238 rows or aggregate a lexicon
across JSON files cheaply). JSON is a first-class projection you can produce
anytime and re-import losslessly: `lingo import --from <json-export>` into a fresh
deck rebuilds the db. A `lingo dump --json` snapshot keeps history reviewable in
git. Derived JSON must not become a second runtime truth (doc 01 §4).

## 3. JSON export shape (kept, like today)

```
hindi_export/
  manifest.json                       lingo.package/v2 (identity, counts, integrity)
  sentences/<sentence-id>.json        one file per sentence (lingo.sentence/v1)
  audio/<sentence-id>.mp3
  README.txt
```

`sentences/<sentence-id>.json` (self-contained, browsable/greppable/editable):

```jsonc
{
  "format": "lingo.sentence/v1",
  "id": "01J8ZQ…",
  "collection": "Complete Hindi",
  "section": "Chapter 02",
  "order": 1,
  "target": "अध्यापक जी, यहाँ कितने विद्यार्थी हैं?",
  "romanisation": "adhyāpak jī, yahā̃ kitne vidyārthī haĩ?",
  "english": "Teacher ji, how many students are here?",
  "authority": { "english": "human" },
  "breakdown": [ { "surface": "अध्यापक", "roman": "adhyāpak", "gloss": "teacher" } ],
  "audio": "audio/01J8ZQ….mp3",
  "provenance": { "kind": "imported", "package": "sentence_package_01_lingo" }
}
```

`manifest.json`:

```jsonc
{
  "format": "lingo.package/v2",
  "language": { "name": "Hindi", "code": "hi", "script": "Devanagari", "romanisation": "iast-tilde" },
  "collection": { "title": "Complete Hindi" },
  "counts": { "sentences": 238, "words": 612, "audio_files": 238 },
  "integrity": { "algorithm": "sha256", "files": { "sentences/01J8ZQ….json": "sha256:…", "audio/01J8ZQ….mp3": "sha256:…" } }
}
```

(A single consolidated `library.json` is also offered via `lingo dump --json`,
doc 03 §9.)

## 4. Optional SQLite export (for Grasp)

```
$ lingo package --as db --dest ~/exports/hindi --filter "section:Chapter 02"
~/exports/hindi/
  manifest.json     lingo.package/v2 (library = library.db)
  library.db        lingo.library/v1 (filtered copy)
  audio/…
```

A filtered **copy** of the db + audio. Grasp bundles it read-only (never shares
the live deck db). The db schema (`lingo.library/v1`, doc 03 §6) is the contract
to validate against Grasp.

## 5. Import = read into the library

`lingo import --from <dir>` reads an export (JSON `sentences/<id>.json`, a
`library.json`, or a `library.db`) and **merges** it:

- new sentences inserted with fresh ids; `provenance.kind = "imported"`;
- audio copied into the deck's `audio/`;
- words/meanings folded into the lexicon by `key`;
- `order` appended after existing content (organize later).

It accepts only the formats we **export**. A foreign/old package is converted to
our JSON by a throwaway script first, then imported — the CLI never grows readers
for arbitrary shapes. (This is exactly how the current Grasp `v1` package was
brought in; that path is replaced by a proper package-import use case, doc 09.)

```
$ lingo import --from ~/grasp_converted
✓ 238 sentences imported · 238 audio · 612 words   (provenance: imported)
```

## 6. Anki is a different publisher

Anki export is separate — its own format (`.apkg` = its own SQLite + media),
produced by `lingo export`, not by import/export of our JSON.

```
library.db ──▶ lingo package [--as json|db] ──▶ JSON (default) or db   (Grasp & friends)
library.db ──▶ lingo export                 ──▶ deck.apkg              (Anki)
```

## 7. Grasp consumption (contract notes)

- Grasp bundles an exported copy (JSON or db) as a read-only resource; never the
  live deck db. No concurrency concern.
- Grasp pins the schema/format version it understands; changes bump the version
  and ship a migration; Grasp upgrades deliberately.
- Audio resolves via the sentence's `audio` path relative to the export root.

> Open implementation question: confirm Grasp's current loader expectations so
> `lingo.library/v1` (doc 03) / `lingo.sentence/v1` match what Grasp queries.
