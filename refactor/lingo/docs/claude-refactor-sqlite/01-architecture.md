# 01 · Target architecture

## 1. Preserve the hexagonal split

The existing project has a useful hexagonal split that we keep:

```text
lingo-domain        pure value objects and valid-by-construction aggregates
lingo-application   use cases, typed reports, and ports
lingo-workspace-fs  local workspace/config/profile/run/library adapters
lingo-prompt        prompt rendering and reply parsing adapters
lingo-audio         audio backend adapter catalog
lingo-artifacts     portable package and Anki publishers
lingo-cli           CLI, composition root, local viewer server, Studio edge
apps/viewer         static UI
```

The refactor changes **what flows through** the architecture, not the layers.
The old runtime center is batch files (`raw → source YAML → card JSON → audio →
package`). The new runtime center is a **sentence library** (`library.db`).

## 2. Boundary map

```text
Owner: lingo-domain
Canonical truth: typed concepts and invariants for Sentence, Word, AudioAttachment,
  FieldAuthority, Status, Provenance, selection IDs.
Derived data: none. Domain does not know storage, SQL, files, CLI, Studio, or provider APIs.
Allowed deps: serde, hashing/normalization helpers, thiserror.
Forbidden deps: rusqlite, std::fs layout, prompt engines, audio providers, CLI/viewer DTOs.
Public contracts: value-object constructors, aggregate methods, validation helpers, closed-set wire names.
Drift checks: valid/invalid value-object tests; serialization tests for persisted/wire names;
  validation tests for enrichment invariants.
```

```text
Owner: lingo-application
Canonical truth: workflow policy and typed reports.
Derived data: CLI hints, status summaries, publish material assembled from the library.
Allowed deps: lingo-domain and the port traits this crate defines.
Forbidden deps: rusqlite, serde_json::Value as an internal model, filesystem layout,
  provider SDKs, viewer DTOs.
Public contracts: use-case functions (prepare_extract, apply_extract, claim_for_enrichment,
  apply_enrichment, list_library, reorder_sentences, synthesize_audio, package, export_anki).
Drift checks: service tests with fake stores; typed report tests; architecture dependency test.
```

```text
Owner: lingo-workspace-fs
Canonical truth: local persistence mechanics for config, profiles, run journal,
  the SQLite library, and audio file bytes.
Derived data: scan/health classification built from library.db + files.
Allowed deps: lingo-application ports, lingo-domain types, rusqlite, serde codecs, fs APIs.
Forbidden deps: CLI output, viewer DTOs, prompt templates, audio providers, Anki/package formatting.
Public contracts: FsWorkspace, SqliteLibraryStore, FsRunJournal, FsProfileCatalog, AudioFileStore.
Drift checks: migration tests, round-trip tests through the port, FK/CHECK tests, partial-file/audio health tests.
```

```text
Owner: lingo-cli / apps/viewer
Canonical truth: none, except local UI state while a screen is open.
Derived data: DTOs and terminal rendering mapped from typed application reports.
Allowed deps: all concrete adapter crates, at the composition root.
Forbidden deps: direct SQL for business operations, direct writes to derived exports,
  business validation hidden in JSON handlers.
Public contracts: CLI commands, Studio HTTP routes, output formats.
Drift checks: CLI smoke tests; Studio DTO mapping tests; route tests proving JSON is parsed
  into typed requests before use cases run.
```

## 3. Dependency arrows (one-way)

```text
apps/viewer -> lingo-cli -> concrete adapters + lingo-application -> lingo-domain
                           \-> lingo-workspace-fs -> lingo-application + lingo-domain
                           \-> lingo-prompt       -> lingo-application + lingo-domain
                           \-> lingo-audio        -> lingo-application + lingo-domain
                           \-> lingo-artifacts    -> lingo-application + lingo-domain
```

`lingo-application` owns the port traits. Concrete crates implement them.
`lingo-cli` is the composition root that chooses the concrete store, prompts,
audio backends, publishers, and environment probe.

## 4. Source-of-truth rule

`library.db` is the only runtime authority for:

- sentence identity, order, section, collection membership, and status;
- target text, romanisation, English, literal gloss, register, tags, field authority;
- word identities, meanings, and sentence-word occurrences;
- audio metadata: current path, hash, backend, voice/model, format;
- enrichment claims and run ownership.

Derived surfaces (rebuild/republish; never merge back as truth unless an explicit
import command validates and commits through the store):

- JSON package exports and `lingo dump --json` snapshots;
- the Anki `.apkg` SQLite database;
- manifests and checksums;
- CLI/Studio JSON rows;
- generated prompt packets and reply journals;
- any in-memory indexes.

## 5. Boundary decision: no new SQLite crate yet (D8)

Implement SQLite under `lingo-workspace-fs/src/library/`. That crate already owns
local workspace layout and persistence adapters; adding `rusqlite` there does not
force SQL into policy crates. A separate `lingo-library-sqlite` crate becomes
justified only when one of these is true:

- `lingo-workspace-fs` grows unrelated persistence responsibilities with a
  separate release/test cadence;
- another composition root wants the SQLite library without the fs workspace;
- a second production library store appears and needs a conformance-test boundary;
- compile-time evidence shows the boundary prevents real accidental imports.

Until then, a module boundary plus conformance tests is simpler than a crate.

## 6. What to retire as runtime authority

- `input/sentences/<batch>.yaml` as canonical source;
- `output/sentences/<batch>.json` as canonical cards;
- the prototype `sentences/<batch>__<item>.json` per-sentence layer;
- status derived from scanning file layers.

Keep their codecs only as import/export or one-time migration tools (see
[08](./08-import-export.md), [10](./10-implementation-plan-and-tests.md)). If
there is no production data to preserve, remove the compatibility paths rather
than maintaining dual truth.

Two existing prototypes are explicitly superseded:

- the `sentences/*.json` layer → becomes rows in `library.db`;
- `lingo import-package` → becomes a package-import use case committing through
  `LibraryStore` (see [09](./09-reuse-and-patterns.md)).
