# Architecture

## Viewer Boundary

The viewer is already an Astro app. Rust should not reimplement it. `hindi
viewer` is a convenience wrapper that runs the viewer from the correct working
directory and prints the expected URL.

```text
hindi viewer
  -> cd viewer
  -> npm run dev
  -> Astro reads ../output and ../audio
```

The viewer's existing npm lifecycle runs `scripts/sync-audio.js`, which links
`viewer/public/audio` to the project-root `audio/`.

## Export Boundary

M6 export writes a rebuildable artifact. It does not send to live Anki. The
viewer remains the interactive export path.

```text
output/sentences/*.json
  -> filter by title/subtitle
  -> map sentence fields to Anki fields
  -> exports/<source>_<topic>_sentences.tsv
```

## Field Contract

Sentence export fields:

- `English`
- `Hindi`
- `Audio`
- `Romanisation`
- `Literal`
- `Register`
- `WordBreakdown`
- `Topic`
- `Tags`

The artifact should include Anki import headers:

```text
#separator:tab
#html:true
#notetype:Hindi Sentence
#deck:Hindi::Sentences
#columns:English	Hindi	Audio	Romanisation	Literal	Register	WordBreakdown	Topic	Tags
```

Exact deck naming can be simple in M6. Deck customization can come later.

## Audio Contract

The viewer converts explicit relative paths like:

```text
audio/sentences/example_batch_01/01_sentence.mp3
```

into flat Anki media filenames by stripping `audio/` and replacing `/` with
`__`. Rust should mirror that:

```text
sentences__example_batch_01__01_sentence.mp3
```

The Anki field becomes:

```text
[sound:sentences__example_batch_01__01_sentence.mp3]
```

## Word Breakdown Contract

Use a simple HTML list/table representation from `words[]`. It must be stable
and readable, not necessarily pixel-identical to the viewer preview.

At minimum include each word's Hindi, roman, and meaning.

## Data Safety

- `hindi viewer` writes no project data directly.
- `hindi export` writes only under `exports/`.
- Neither command modifies `input/`, `output/`, or `audio/`.
