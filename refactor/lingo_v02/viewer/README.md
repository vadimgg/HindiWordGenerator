# Lingo Viewer

A CLI-first Astro viewer for the sentence-centric Lingo refactor.

This replaces the old batch-file viewer. The runtime truth is now the local deck's
`library.db`; JSON packages, Anki exports, and prompt files are derived or
scratch artifacts. The viewer is deliberately a helper: each page shows the exact
CLI command that performs the same operation.

## Pages

- **Generate**: raw text to draft sentences with `lingo extract`, then bounded
  enrichment with `lingo enrich --limit N`.
- **Import**: merge a package with `lingo import --from <DIR>`.
- **Organize**: library order, section assignment, tags, edits, and deletes.
- **Words**: lexicon projected from committed sentence breakdowns.
- **Audio**: generate or regenerate sentence audio independently of enrichment.
- **Anki**: export selected/ready sentences to `.apkg` or AnkiConnect-backed flows.
- **Package**: export a JSON package or SQLite db copy.

## Commands

```bash
npm install
npm run dev
npm run check
npm run build
npm run preview
```

`npm run check` intentionally uses only Node's built-in test runner plus the
included static build script, so the core viewer can be smoke-tested without a
full Astro install. `npm run dev` and `npm run build` use Astro.

## Local server contract

At runtime the browser tries to load `GET /api/view/state`. If the endpoint is
unavailable, the viewer runs in offline demo mode with fixture data and still
shows the correct CLI commands.

Expected response shape:

```jsonc
{
  "workspace": {
    "name": "Complete Hindi",
    "language": "Hindi",
    "languageCode": "hi",
    "libraryPath": "library.db"
  },
  "config": {
    "display": { "lead": "romanisation", "showSecondary": true },
    "audio": { "backend": "gtts", "voice": "", "model": "" },
    "anki": { "deck": "Hindi::Sentences", "replace": false },
    "package": { "destination": "packages/sentences", "format": "json" }
  },
  "sentences": [
    {
      "id": "01J...",
      "collection": "Complete Hindi",
      "section": "Chapter 02",
      "order": 1,
      "target": "...",
      "romanisation": "...",
      "english": "...",
      "literal": "...",
      "register": "standard",
      "status": "enriched",
      "tags": ["classroom"],
      "authority": { "english": "human" },
      "breakdown": [{ "surface": "...", "roman": "...", "gloss": "...", "kind": "noun" }],
      "audio": { "path": "audio/01J....mp3", "backend": "gtts", "voice": null, "hash": "sha256:..." }
    }
  ],
  "words": [
    {
      "key": "जी",
      "form": "जी",
      "roman": "jī",
      "kind": "particle",
      "count": 14,
      "meanings": ["honorific (ji)"],
      "sentenceIds": ["01J..."]
    }
  ]
}
```

Optional mutations are sent to `POST /api/view/action` with `{ "kind": "...",
"payload": { ... } }`. The UI remains useful without mutation support because it
surfaces the equivalent CLI command in the `⌘ CLI` drawer.

## Drop-in layout

This zip is rooted as:

```text
lingo/apps/viewer/
```

Copy or merge that folder into the Rust workspace. The viewer has no direct SQL,
filesystem, prompt, audio-provider, or artifact logic; those remain behind the
CLI/local server composition root.
