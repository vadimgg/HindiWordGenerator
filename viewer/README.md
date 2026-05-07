# Hindi Generator Viewer

Astro viewer for the generated Hindi word and sentence batches.

The viewer reads live data from the generator project root:

```text
../output/words/*.json
../output/sentences/*.json
../audio/
```

That means the normal workflow is:

```bash
uv run main.py check
uv run main.py run --max-batches 1
cd viewer
npm run dev
```

When new JSON or MP3 files are generated, refresh the page. The Astro dev server
will read the latest `output/` files and serve audio through
`viewer/public/audio`, which is a symlink to the project-root `audio/` directory.

## Commands

```bash
npm install
npm run dev
npm run check
npm run check:ankiconnect
npm run build
npm run preview
```

Use `npm run check` before committing viewer changes. It runs sentence quality
checks, Anki preview/export parity checks, and a production Astro build.

`npm run check:ankiconnect` is an optional live smoke. Run it only when Anki is
open and the AnkiConnect add-on is enabled; it checks the connection, syncs the
word and sentence note types, and verifies the sentence note type uses the
`Topic` metadata field.

The dev server usually opens at:

```text
http://localhost:4321
```

## Data Contract

Word batches should have this shape:

```json
{
  "title": "Complete Hindi",
  "subtitle": "Chapter 01",
  "words": []
}
```

Sentence batches should have this shape:

```json
{
  "title": "Complete Hindi",
  "subtitle": "Chapter 02",
  "sentences": []
}
```

`title` and `subtitle` are required top-level keys. The viewer does not support
legacy `chapter`-only batches.

Cards with an `audio` field render a play button. Cards without audio still
render normally.

## Anki Export Workflow

Use the quick export buttons in Words or Sentences to send a whole source/topic
group to Anki immediately. Use Deliver when you want an advanced custom export
based on the current selected cards.
