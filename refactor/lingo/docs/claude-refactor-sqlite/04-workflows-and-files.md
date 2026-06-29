# 04 · Workflows & file design

## 1. Target workspace layout

```text
my-deck/
  config.toml
  library.db                    # canonical runtime store (doc 03)
  audio/<sentence-id>.mp3        # path stored in sentence_audio.path
  raw/<source>.md                # input scratch for extract workflows
  runs/<stage>/<run-id>/         # prompt.md, reply.md, run.json, applied.json
  packages/                      # derived package exports
  exports/                       # derived Anki exports
  prompts/                       # optional deck-local prompt overrides
  profile.toml                   # optional deck-local profile override
```

Not runtime authorities (may exist only during one-time migration or as derived
export formats): `input/sentences/<batch>.yaml`, `output/sentences/<batch>.json`,
`sentences/<batch>__<item>.json`.

## 2. Extract: raw text → draft sentences

```text
CLI/Studio request
  -> application::prepare_extract  (load raw, resolve context, render packet, record PreparedRun)
  -> PromptEngine::render_extract
  -> [transport: manual | file handoff | api]  (doc 05)
  -> PromptEngine::parse_extract_reply         (strict parse only)
  -> application::apply_extract                (require prepared run, check stage/profile)
  -> application::extract::accept              (untrusted draft DTOs -> domain SentenceDraft)
  -> LibraryStore::insert_drafts              (one transaction)
  -> ExtractReport -> CLI/Studio DTO
```

Invariants:
- A raw reply cannot overwrite existing human fields unless it is an explicit
  edit/import merge with typed rules (doc 03 §3).
- Draft insert is all-or-nothing.
- Provenance records the producer kind and run/package source.

## 3. Enrich: draft sentences → translation + breakdown (bounded)

```text
prepare_enrich(selection, limit)
  -> LibraryStore::claim_for_enrichment   (draft -> enriching + run id, one tx)
  -> PromptEngine::render_enrich          (only the claimed sentence ids)
  -> RunJournal::record_prepared
apply_enrich(run, reply)
  -> require prepared run
  -> parse reply
  -> validate exact id set for the claimed run
  -> validate human fields are byte-identical (doc 03 §3)
  -> validate required generated fields + breakdown coverage
  -> LibraryStore::apply_enrichment       (enriching -> enriched; project words/meanings/occurrences; one tx)
```

Invariants (doc 03 §4):
- claiming changes `draft → enriching` and stores the run id in the same tx;
- a second prepare cannot claim the same rows;
- apply only touches rows claimed by that run;
- `--force` re-enriches `enriched` rows but still preserves human fields;
- `reset_enrichment_claim` recovers abandoned `enriching` rows.

## 4. Organize: reorder / section / retitle / tag / edit / delete

```text
application::library::{list, organize, edit}
  -> LibraryStore::{reorder, update_sentence, delete_sentences}
  -> CLI commands/library.rs (ls, get, edit, move, reorder, section, delete)
  -> Studio handlers mirror the same use cases
```

Invariants:
- reorder is one transaction preserving the unique-order invariant (doc 03 §6);
- editing a field from CLI/UI marks that field `human`;
- deleting a sentence cascades occurrences and its `sentence_audio` row; audio
  bytes are deleted by explicit policy or reported as orphaned for cleanup.

## 5. Audio: a service over the library (not a gate)

```text
application::audio::synthesize_audio(selection, mode, backend, voice)
  -> AudioCatalog (backend + fallback)
  -> AudioFileStore::write_sentence_audio  (stage temp -> hash -> verify -> install audio/<sentence-id>.mp3)
  -> LibraryStore::set_audio               (upsert sentence_audio AFTER the file exists and matches hash)
```

Invariants:
- bytes are written to a staged temp file, hashed, verified, then atomically
  installed;
- the `sentence_audio` row is written only after the file exists and matches the
  recorded hash;
- missing audio is status/actionable, not a deterministic validation failure
  (unless a publisher requires audio).

## 6. Words / lexicon

```text
application::words::{list_words, get_word}
  -> LibraryStore::list_words (reads words / word_meanings / sentence_words)
  -> CLI commands/words.rs (scriptable view)
```

Invariants: word tables are derived from committed sentence breakdowns at apply
time; if projection drifts, rebuild from canonical enriched sentences (doc 03 §5).

## 7. Package & export

```text
application::package(selection, destination, format)  -> lingo-artifacts package/{json,db}
application::export_anki(selection, deck, destination) -> lingo-artifacts anki/*
```

Invariants: outputs are derived and verified after write; a db package is a
filtered **copy**, never the live deck db; Anki schema is separate from the Lingo
schema (doc 08).

## 8. Status

```text
application::status -> typed facts + NextAction
  reads canonical library.db state + file health (workspace-fs library/health.rs)
```

Invariants: status reads canonical db + file health; it never infers state from
exported JSON; pending/incomplete/corrupt audio or db states are classified
distinctly.
