# 02 — Crate and File Structure

## Start with five crates

Do not begin with a crate for every future concern. Start with real boundaries and split later when dependencies prove the need.

```text
Cargo.toml
crates/
  lingo-domain/
  lingo-service/
  lingo-sqlite/
  lingo-workspace/
  lingo-cli/
```

Potential future splits:

```text
lingo-handoff   when prompt templates + reply codecs would pollute service/workspace
lingo-audio     when TTS provider dependencies become non-trivial
lingo-publish   when zip/Anki/study-writer dependencies become non-trivial
```

## `lingo-domain`

Pure business model and invariants. No SQL, no filesystem, no CLI, no network.

```text
crates/lingo-domain/src/
  lib.rs
  ids.rs
  clock.rs
  profile/
    mod.rs
    hindi.rs
    catalog.rs
  deck.rs
  sentence/
    mod.rs
    lifecycle.rs
    text.rs
    authority.rs
    tokens.rs
    edit.rs
    visible_status.rs
  run.rs
  audio.rs
  word.rs
  publish.rs
  reports.rs
  error.rs
```

Owns:

- `DeckSlug`, `SentenceId`, `RunId`, `ProfileId`;
- sentence lifecycle, approval state, QA state, sentence origin;
- field authority rules;
- target edit impact classification;
- token and word-key invariants;
- run stage/status closed sets;
- audio fingerprint value objects;
- publish format closed set.

Does not own:

- row mapping;
- path layout;
- terminal command strings;
- prompt template text;
- `serde_json::Value` as behavior model.

## `lingo-service`

Application use cases and ports. It orchestrates domain decisions through contracts.

```text
crates/lingo-service/src/
  lib.rs
  deps.rs
  ports/
    mod.rs
    repository.rs
    workspace.rs
    prompts.rs
    audio.rs
    publish.rs
    clock.rs
    ids.rs
  requests/
    mod.rs
    status.rs
    extract.rs
    enrich.rs
    qa.rs
    apply.rs
    edit.rs
    audio.rs
    publish.rs
    import.rs
    deck.rs
    runs.rs
  reports/
    mod.rs
    next.rs
    status.rs
    apply.rs
    edit.rs
    audio.rs
    publish.rs
    doctor.rs
  use_cases/
    mod.rs
    init.rs
    status.rs
    extract.rs
    enrich.rs
    qa.rs
    apply.rs
    edit.rs
    audio.rs
    publish.rs
    import.rs
    deck.rs
    runs.rs
    words.rs
    doctor.rs
  errors.rs
  test_support/
    fake_repo.rs
    fake_workspace.rs
    fixtures.rs
```

Owns:

- use-case sequencing;
- `Next` / `Done` / `Blocked` decisions as typed reports;
- repository/workspace/prompt/audio/publish port traits;
- service-level validation that spans multiple aggregates;
- stable error categories and exit-code mapping.

Does not own:

- SQL syntax;
- path string construction;
- terminal formatting;
- provider-specific network details.

## `lingo-sqlite`

SQLite repository adapter. Owns schema, row codecs, transactions, and connection pragmas.

```text
crates/lingo-sqlite/src/
  lib.rs
  connection.rs
  migrations.rs
  schema.sql
  row/
    mod.rs
    deck.rs
    sentence.rs
    run.rs
    token.rs
    audio.rs
  codec/
    mod.rs
    closed_sets.rs
    timestamps.rs
  queries/
    mod.rs
    status.rs
    sentences.rs
    runs.rs
    words.rs
  tx/
    mod.rs
    apply_run.rs
    claim_run.rs
    edit_sentence.rs
    set_audio.rs
    import_package.rs
  repository.rs
  test_support.rs
```

Owns:

- `PRAGMA` setup;
- migrations/schema initialization;
- `BEGIN IMMEDIATE` write transactions;
- strict row decoding from DB wire names to domain enums;
- `run_sentences` claim queries;
- idempotent apply commit mechanics.

Does not own:

- language normalization rules;
- human-field overwrite policy;
- CLI output.

## `lingo-workspace`

Filesystem and config adapter. Owns local layout and safe file writes.

```text
crates/lingo-workspace/src/
  lib.rs
  layout.rs
  paths.rs
  atomic_file.rs
  config/
    mod.rs
    keys.rs
    parse.rs
    defaults.rs
  init.rs
  run_files.rs
  audio_files.rs
  package_files.rs
  prompt_overrides.rs
```

Owns:

- `WorkspaceRoot`, `WorkspaceRelativePath`, resolved paths;
- `library.db`, `config.toml`, `raw/`, `runs/`, `audio/`, `out/`, `prompts/` paths;
- atomic write + read-back verification;
- `run.json` mirror read/write/repair;
- audio file writes to deterministic flat path;
- typed config loading and writing.

Does not own:

- sentence state rules;
- run validation;
- SQL;
- model reply parsing beyond reading bytes.

## `lingo-cli`

Thin edge. Parses arguments, wires adapters, calls services, renders reports.

```text
crates/lingo-cli/src/
  main.rs
  runtime.rs
  args/
    mod.rs
    status.rs
    extract.rs
    enrich.rs
    qa.rs
    apply.rs
    edit.rs
    audio.rs
    publish.rs
    import.rs
    deck.rs
    runs.rs
    doctor.rs
  commands/
    mod.rs
    status.rs
    extract.rs
    enrich.rs
    qa.rs
    apply.rs
    edit.rs
    audio.rs
    publish.rs
    import.rs
    deck.rs
    runs.rs
    doctor.rs
  render/
    mod.rs
    theme.rs
    json.rs
    text.rs
    sentence_block.rs
    next.rs
    errors.rs
```

Owns:

- clap definitions;
- converting raw CLI strings into typed requests;
- terminal formatting and ANSI/ASCII glyph policy;
- JSON DTO mapping for agents;
- process exit code.

Does not own:

- state transitions;
- SQL;
- file paths except as arguments parsed into workspace-relative path values;
- model reply validation.

## When to split future crates

### `lingo-handoff`

Split when prompt templates and reply codecs grow beyond simple service modules.

```text
lingo-handoff/src/
  task.rs
  manifest.rs
  render.rs
  fence.rs
  codecs/extract.rs
  codecs/enrich.rs
  codecs/qa.rs
  templates/*.md.hbs
```

Boundary proof: service tests can pass a fake handoff engine; CLI does not know parser internals.

### `lingo-audio`

Split when gTTS or future providers add external dependencies.

```text
lingo-audio/src/
  catalog.rs
  backend.rs
  gtts.rs
  deterministic_fake.rs
```

Boundary proof: service tests use deterministic fake audio; CLI help uses backend metadata from typed catalog.

### `lingo-publish`

Split when package/study/Anki exporters pull in zip, SQLite writer, or APKG dependencies.

```text
lingo-publish/src/
  package/
  study/
  anki/
  db_copy.rs
```

Boundary proof: service passes typed export snapshots to publisher; publisher does not query authoring DB directly.

## Module comments

Each grown module should start with a short ownership comment:

```rust
//! @domain run-handoff
//! @intent Owns portable run files. DB run rows remain authoritative.
//! @do-not Decide whether a reply is valid here; validation belongs to apply.
```

Use comments for boundary intent, not narration.
