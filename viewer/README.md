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
npm run build
npm run preview
```

The dev server usually opens at:

```text
http://localhost:4321
```

## Data Contract

Word batches should have this shape:

```json
{
  "chapter": "Complete Hindi Chapter 01",
  "words": []
}
```

Sentence batches should have this shape:

```json
{
  "chapter": "Complete Hindi Chapter 02",
  "sentences": []
}
```

Cards with an `audio` field render a play button. Cards without audio still
render normally.

