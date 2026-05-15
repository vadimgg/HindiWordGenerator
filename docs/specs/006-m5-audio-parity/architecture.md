# Architecture

## Boundary

M5 adds audio as a separate sentence enrichment boundary. It does not belong in
generation. `hindi sentences generate` produces accepted JSON without audio;
`hindi sentences audio` later adds missing media and missing metadata.

## Data Flow

```text
output/sentences/*.json
  -> scan missing sentence audio
  -> synthesize missing MP3s
  -> audio/sentences/<batch-stem>/<nn>_<slug>.mp3
  -> patch missing sentence.audio fields
  -> output/sentences/*.json
```

## Trust Boundary

Accepted sentence JSON is already learner data. Audio may add only this field:

```json
"audio": "audio/sentences/<batch-stem>/<nn>_<slug>.mp3"
```

It must not alter:

- `title`
- `subtitle`
- `hindi`
- `romanisation`
- `english`
- `literal`
- `register`
- `source_ref`
- `tokens`
- `words`
- `anki_tags`

## Path Policy

Audio paths are project-relative strings and must:

- start with `audio/sentences/`;
- end with `.mp3`;
- contain no `..`;
- contain no absolute path prefix;
- use a filesystem-safe ASCII filename.

Planned path shape:

```text
audio/sentences/<batch-stem>/<card-index>_<roman-or-english-slug>.mp3
```

Prefer romanisation for the slug when available, then English, then Hindi, then
the numeric index. The slug is only a filename hint; the card identity is still
the JSON entry position inside the batch.

## TTS Backend

Define a small backend boundary, for example:

```rust
trait TtsBackend {
    fn synthesize_hindi(&self, text: &str, target: &Path) -> Result<(), TtsError>;
}
```

Automated tests use a fake backend that writes deterministic bytes. The first
real backend may call a Google/gTTS-style service, but that dependency should be
replaceable without changing scanner or JSON patching logic.

## Atomicity

MP3 files:

1. Write to a temp path in the target directory.
2. Ask the backend to synthesize into the temp path.
3. Rename temp path to final MP3 path.
4. Remove temp path on failure when possible.

Accepted JSON:

1. Read and parse the batch.
2. Apply missing `audio` fields in memory only after required MP3s exist.
3. Write patched JSON to temp path.
4. Rename temp path over the accepted batch.

If audio generation succeeds but JSON patching fails, rerunning the command is
the recovery path. Existing MP3s are skipped and metadata is patched.

## Existing Compatibility

The viewer already prefers explicit `audio` fields and serves project-root
`audio/` through `viewer/public/audio`. M5 should preserve that contract rather
than introduce a sidecar manifest.
