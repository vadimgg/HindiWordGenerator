# Lingo refactor — implementation package

This is a single, self-contained specification for the next iteration of Lingo:
a move from a batch/pipeline runtime to a **sentence-centric SQLite library**,
fed by multiple producers and consumed by multiple publishers. It is written to
be implemented end-to-end.

Read it in order. Each document is authoritative for its scope; where a decision
spans documents it is recorded once here in §Decisions and referenced elsewhere.

## The system in one picture

```
 PRODUCERS                    THE LIBRARY (SQLite)              PUBLISHERS
 ─────────                    ────────────────────              ──────────
 extract  raw → drafts ─▶                            ─▶  Anki     (.apkg / AnkiConnect)
 enrich   drafts → cards ─▶    sentences  +  words    ─▶  Package  (JSON like today,
 import   JSON / db      ─▶    (library.db, runtime)        │        or filtered db copy)
                                       │
                                  audio/<sentence-id>.mp3 (files referenced by the db)

 JSON stays the portable import/export format; SQLite is how we work at runtime.
```

## Documents

| # | Doc | Scope |
|---|-----|-------|
| 00 | [Requirements & workflows](./00-requirements-and-workflows.md) | What exists today, the workflows, requirements R1–R13, non-goals |
| 01 | [Target architecture](./01-architecture.md) | Crate boundaries, dependency arrows, source-of-truth rule, what to retire |
| 02 | [Crate & module design](./02-crate-and-module-design.md) | Per-crate module/file layout, key models and ports |
| 03 | [Data model & SQLite schema](./03-data-model-and-schema.md) | Sentence/word model, field authority, provenance, processing state, the hardened `lingo.library/v1` schema |
| 04 | [Workflows & file design](./04-workflows-and-files.md) | extract, enrich, organize, audio, words, package, export, status flows |
| 05 | [CLI](./05-cli.md) | Command surface, model transports, `--help`/output, exit codes |
| 06 | [UI](./06-ui.md) | DaVinci-style shell, tokens, pages, Settings/Help/CLI homes |
| 07 | [Prompts](./07-prompts.md) | Per-language prompt sets, contracts, location, customization |
| 08 | [Import & export](./08-import-export.md) | Interchange formats; JSON in/out, optional db export |
| 09 | [Reuse & patterns](./09-reuse-and-patterns.md) | Existing code to reuse/retire; patterns to use/avoid |
| 10 | [Implementation plan & tests](./10-implementation-plan-and-tests.md) | Phased migration (incl. walking skeleton) + test matrix |
| 11 | [Public API sketches](./11-public-api-sketches.md) | Core Rust contracts and DTO seams |

## Non-negotiable principles

1. **CLI-first.** Every operation is a clean, scriptable CLI command — the
   surface an agent (Codex / Claude Code) or a human drives. The UI is a helper
   over the same operations and is never the only way to do something.
2. **The UI mirrors the files.** Anything the UI changes is written to the same
   canonical artifacts the CLI reads: a settings change updates `config.toml`, a
   reorder updates `library.db`, an import merges into the library. No UI-only
   state.
3. **The UI shows the CLI.** Each screen exposes the equivalent CLI command in
   one organized place, so the UI teaches the CLI.
4. **One source of truth, two representations.** `library.db` is the canonical
   *runtime* store; **JSON stays the portable interchange** (import/export). The
   db can always be rebuilt from a JSON export. Audio bytes live as files
   referenced by the db. Generation scratch (`raw/`, run packets) is ephemeral.
5. **SQL is an adapter detail.** `lingo-domain` and `lingo-application` must not
   import `rusqlite`, SQL strings, filesystem layout, or viewer DTOs.
6. **Formats are versioned contracts** (`lingo.library/v1`, `lingo.sentence/v1`,
   `lingo.package/v2`) because Grasp and other tools consume them.
7. **Add a boundary only when removing it would spill real complexity into
   callers.** No crates/traits/registries for symmetry alone.

## Decisions (resolved; apply everywhere)

| # | Decision |
|---|----------|
| D1 | The sentence is the runtime atom; `library.db` is canonical for sentences, words, enrichment state, organization, and audio metadata. |
| D2 | Audio bytes are files keyed by sentence id: `audio/<sentence-id>.mp3`; metadata lives in a `sentence_audio` table (re-runnable audio service). |
| D3 | Word identity is the **normalized surface form**, scoped per collection (`UNIQUE(collection_id, key)`). Inflected/gendered variants are separate rows. No lemmatizer (R12). |
| D4 | `sentences.status ∈ {draft, enriching, enriched}`. "Ready to publish" is **derived** (`enriched` + has audio), not stored. |
| D5 | Enrichment is claimed in bounded batches (`--limit`); claimed rows move to `enriching` with an owning run id so they are never sent twice (R13). |
| D6 | Human-authored fields are preserved through enrichment (field authority); enrichment only fills `ai`/empty fields (R4). |
| D7 | Commands: `extract`/`enrich` replace `import`/`build`; `import` (package import) replaces the prototype `import-package`; `package [--as json\|db]`; `export` (Anki). |
| D8 | SQLite lives under `lingo-workspace-fs/src/library/`. No new `lingo-library-sqlite` crate yet (triggers for splitting are in doc 01). |
| D9 | The schema in doc 03 (`lingo.library/v1`, hardened: `STRICT`, FKs, CHECKs, `json_valid`, unique order, `library_metadata`) is authoritative. |

## Status

Design complete; not yet implemented. Two prototypes exist in the current code
and are explicitly **superseded** by this package: the per-sentence
`sentences/*.json` layer and `lingo import-package`. They become "the library"
and a proper package-import use case respectively (doc 09).
