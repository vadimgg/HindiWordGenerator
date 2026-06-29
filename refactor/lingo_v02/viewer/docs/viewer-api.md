# Viewer API contract

The refactored viewer is intentionally presentation-only. It does not open
SQLite, mutate files, call TTS providers, or build packages itself. The Rust
composition root should serve static files and expose two optional JSON routes.

## `GET /api/view/state`

Returns a presentation DTO projected from typed application reports and the
canonical library store.

```ts
type ViewState = {
  workspace: {
    name: string;
    language: string;
    languageCode?: string;
    libraryPath: string;
  };
  config: {
    display: { lead: 'romanisation' | 'target'; showSecondary: boolean };
    audio: { backend: 'gtts' | 'elevenlabs'; voice?: string; model?: string };
    anki: { deck: string; replace?: boolean };
    package: { destination: string; format: 'json' | 'db' };
  };
  sentences: ViewSentence[];
  words?: ViewWord[];
};
```

`words` is optional. If omitted, the browser projects a lightweight lexicon from
sentence breakdowns for display only.

## `POST /api/view/action`

```jsonc
{ "kind": "audio.missing", "payload": { "ids": [], "backend": "gtts" } }
```

Supported action names used by the UI:

- `extract.prepare`
- `enrich.prepare`
- `enrich.reset`
- `import.package`
- `organize.reorder`
- `audio.missing`
- `audio.force`
- `anki.export`
- `package.export`
- `config.save`

The route may return `{ "message": "...", "state": <ViewState> }`. When it is
not implemented, the UI still works as a CLI companion and displays commands in
the `⌘ CLI` drawer.

## Mapping rule

Handlers should parse JSON into typed DTOs, call application use cases, then map
typed reports back to this wire shape. Do not place SQL or business validation in
viewer handlers.
