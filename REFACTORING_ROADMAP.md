# Refactoring Roadmap

This document consolidates the first read-only review from the new local agent
packs. It is a planning document, not a record of completed work.

## Review Snapshot

- Python runtime review used `python-reviewer` standards for module ownership,
  file/function size, data drift, CLI behavior, and validation.
- Viewer review used `astro-viewer` standards for product UI, data loading,
  Anki export, audio handling, browser safety, and CSS organization.
- Project review used `project-reviewer` and `doc-writer` standards for docs,
  data-surface ownership, and workflow drift.

Current highest-risk theme: the project works, but several surfaces now know
too much about the same card data. The next refactors should reduce drift before
adding larger features such as transcription.

## P1: Fix Before Bigger Feature Work

### 1. Remove unsafe `innerHTML` rendering in Deliver rows

Affected files:

- `viewer/src/scripts/ui/exportPane.js`

Why it matters:

Generated card data is inserted into Deliver rows through HTML strings. Even in
a local tool, LLM-generated JSON should not be trusted as markup. A malformed or
hostile card could alter the page.

Refactor shape:

- Add small DOM row builders that use `createElement` and `textContent`.
- Keep any static badges as created elements, not interpolated strings.
- Add a regression check with a card containing `<script>`-like text.

Validation:

- `cd viewer && npm run check`
- Browser smoke: Deliver tab renders selected word and sentence rows normally.

### 2. Establish one canonical batch data contract

Affected files:

- `README.md`
- `AGENTS.md`
- `CLAUDE.md`
- `ARCHITECTURE.md`
- `viewer/README.md`
- `review_prompt_words.txt`
- `review_prompt_sentences.txt`
- `process.py`
- `viewer/src/pages/index.astro`

Why it matters:

Current project docs and prompts used both the new `title` / `subtitle` shape
and the older `chapter` shape. The project should not retain legacy
compatibility for generated batch contracts.

Refactor shape:

- Document `title` and `subtitle` as canonical and required.
- Reject `chapter` as an unexpected top-level key in newly written output.
- Add a short data contract section to `ARCHITECTURE.md` or a future
  `DATA_SURFACES.md`.
- Update reviewer prompts and viewer README to stop teaching `chapter`.
- Remove viewer/reviewer fallback behavior that reads `chapter`.

Validation:

- `uv run main.py check --type words --max-batches 1`
- `uv run main.py check --type sentences --max-batches 1`
- `cd viewer && npm run check`

### 3. Correct manifest and prompt-change documentation

Affected files:

- `README.md`
- `AGENTS.md`
- `CLAUDE.md`
- `ARCHITECTURE.md`
- `process.py`

Why it matters:

Some docs imply prompt-hash changes reprocess existing files. The current
append-only planner treats existing output JSON as the source of completed
cards, so prompt changes affect future pending work unless an explicit repair or
regeneration flow is used.

Refactor shape:

- Document the real rule: output JSON decides completed cards; `manifest.json`
  is audit metadata.
- Add a future repair/migration command for intentional prompt-driven backfills.
- Keep normal `run` append-only and non-destructive.

Validation:

- `uv run main.py check --type sentences --batch-size 5 --max-batches 1`
- `uv run main.py run --dry-run --max-batches 1`

## P2: Structural Refactors

### 4. Extract generated-data loading from `index.astro`

Affected files:

- `viewer/src/pages/index.astro`
- future `viewer/src/utils/loadGeneratedData.ts`

Why it matters:

`index.astro` currently owns filesystem loading, metadata parsing,
grouping, search payloads, data health, and QA issue construction. It also uses
loose `any` handling around generated JSON.

Refactor shape:

- Add a loader module with:
  - `readJsonFiles`
  - `normaliseWordBatch`
  - `normaliseSentenceBatch`
  - `buildViewerPayload`
- Return `{ data, warnings, qaIssues }`.
- Keep the Astro page focused on calling the loader and rendering components.

Validation:

- Fixture tests for malformed current-schema batches.
- `cd viewer && npm run check`

### 5. Split the Python runtime by ownership boundaries

Affected files:

- `process.py` reduced from 768 to 206 lines
- `generate.py` reduced from 856 to 512 lines
- `main.py` reduced from 380 to 149 lines

Why it matters:

The files still work, but they are over the project size thresholds. The risk is
not line count alone; planning, metadata, validation, writing, display,
provider orchestration, retries, progress, persistence, and audio attachment are
hard to review as separate behaviors.

Refactor shape:

- Keep `main.py` as the thin CLI router: done.
- Move check planning/rendering: done in `check_report.py`.
- Split `process.py` only along natural ownership:
  - shared path config: done in `pipeline_config.py`
  - input parsing and metadata: done in `batch_planner.py`
  - existing output scan and planning: done in `batch_planner.py`
  - schema validation: done in `schema_validator.py`
  - manifest persistence: done in `manifest_store.py`
  - validated writes and output placement: remains in `process.py`
- Split `generate.py` only along natural ownership:
  - provider/model creation: done in `llm_client.py`
  - prompt/message/JSON response handling: done in `llm_client.py`
  - batch execution and retry: retry done in `llm_client.py`; orchestration
    remains in `generate.py`
  - process/write/audio subprocess boundary: done in `generation_io.py`
  - progress/summary rendering
- Preserve public CLI behavior while moving internals.

Validation:

- Done: `python3 -m py_compile process.py batch_planner.py pipeline_config.py schema_validator.py manifest_store.py main.py generate.py generation_io.py llm_client.py generation_types.py`
- Done: `uv run main.py check --type sentences --batch-size 5 --max-batches 1`
- Done: `uv run main.py run --dry-run --type sentences --batch-size 5 --max-batches 1`
- Done: invalid audio path write probe still rejects before output write.
- Add focused tests before or during extraction when practical.

### 6. Finish the metadata object cleanup

Affected files:

- `batch_planner.py`
- `process.py`
- `main.py`

Why it matters:

The first cleanup pass renamed the main planner and viewer grouping paths away
from `chapter`. Remaining work should make metadata a clearer structured object
instead of passing a combined label between helper functions.

Refactor shape:

- Introduce a small batch metadata object with `title`, `subtitle`, and
  `display_label`: done as `BatchMetadata`.
- Replace ad hoc `split_title_subtitle()` calls with that object at boundaries:
  done inside pending batch planning and batch CSV construction.
- Move filename-derived metadata into one helper: done via
  `BatchMetadata.from_parts(..., fallback_label=label_from_stem(...))`.
- Add a migration/backfill command only after the data contract is documented:
  not needed for current data because existing input/output already uses the
  current `title`/`subtitle` contract.

Validation:

- Done: `python3 -m py_compile batch_planner.py process.py check_report.py main.py pipeline_config.py schema_validator.py manifest_store.py generate.py generation_io.py llm_client.py generation_types.py`
- Done: `uv run main.py check --type sentences --batch-size 5 --max-batches 1`
- Done: `uv run main.py run --dry-run --type sentences --batch-size 5 --max-batches 1`
- Done: `uv run process.py check --type sentences --batch-size 5`

### 7. Split viewer interaction and styling files

Affected files:

- `viewer/src/scripts/ui/pageInteractions.js` was split into a small composer
  plus focused interaction modules.
- `viewer/src/scripts/ui/exportPane.js` now delegates safe Deliver row
  rendering to `viewer/src/scripts/ui/deliverRows.js`.
- `viewer/src/styles/global.css` is now an import manifest for focused partials.

Why it matters:

The viewer is now a serious workbench. Its main interaction controller owns too
many behaviors, and the stylesheet mixes shell, cards, Deliver, QA, Anki
preview, and audio styling.

Refactor shape:

- Split `pageInteractions.js` into focused modules:
  - filter panel wiring
  - view mode controls
  - group collapse and selection
  - drag selection
  - group filter chips
- Split `exportPane.js` after the unsafe row rendering is fixed:
  - status polling
  - table rendering
  - export command wiring
  - feedback rendering
- Split CSS into stable surface files:
  - `base.css`
  - `card-primitives.css`
  - `app-shell.css`
  - `deliver.css`
  - `qa.css`
  - `anki-preview.css`

Current status:

- Done: page interaction split.
- Done: safe Deliver preview row rendering.
- Done: CSS split into `viewer/src/styles/partials/`.
- Done: `exportPane.js` split into AnkiConnect status, deck controls,
  selection preview, export action, and row-rendering modules.
- Remaining: add focused DOM/controller smoke tests for the new Deliver modules
  if these interactions keep changing.

Validation:

- `cd viewer && npm run check`
- Browser smoke: search, tabs, selection, group collapse, drag selection, audio,
  quick export, Deliver preview.

### 8. Make audio a single validated asset model

Affected files:

- `process.py`
- `viewer/src/utils/audioHelpers.ts`
- `viewer/src/utils/audioAssets.js`
- `viewer/src/scripts/anki/media.js`
- `viewer/src/scripts/anki/export.js`

Why it matters:

Browser playback reads JSON `audio`, while Anki media filenames are still partly
re-derived from card data. That can create preview/export drift when naming
rules change.

Refactor shape:

- Normalize audio once into an object such as:

```json
{
  "path": "audio/sentences/stem_batch_01/01_sentence.mp3",
  "browserSrc": "/audio/sentences/stem_batch_01/01_sentence.mp3",
  "mediaFilename": "sentences__stem_batch_01__01_sentence.mp3"
}
```

- Reject or warn on audio paths outside `audio/` or `/audio/`.
- Keep Anki export and browser playback reading the normalized shape.

Validation:

- Done: audio path unit tests via `viewer/scripts/check-audio-assets.js`.
- Done: browser playback and Anki media naming both use the normalized asset.
- Done: Anki preview/export parity check.
- Done: `process.py write` rejects unsafe audio paths before writing output.

### 9. Separate Anki export service from UI controllers

Affected files:

- `viewer/src/scripts/ui/ankiPreview.js`
- `viewer/src/scripts/ui/exportPane.js`
- `viewer/src/scripts/ui/quickExport.js`
- `viewer/src/scripts/anki/export.js`
- `viewer/src/scripts/anki/exportService.js`
- `viewer/src/scripts/anki/mediaUploader.js`
- `viewer/src/scripts/anki/deckNames.js`

Why it matters:

Preview and export now share field builders, which is good. The remaining issue
is orchestration: UI modules still coordinate deck names, AnkiConnect status,
media upload, selected card extraction, and send results.

Refactor shape:

- Keep field builders pure.
- Add service-style modules:
  - `anki/exportService.js`: done.
  - `anki/mediaUploader.js`: done.
  - `anki/deckNames.js`: done.
- Let UI modules call high-level commands and render returned results: done for
  Deliver export actions and quick export.
- Keep `anki/export.js` as a compatibility facade during the transition.

Validation:

- Done: `npm run check`.
- Done: `uv run main.py check --type sentences --batch-size 5 --max-batches 1`.
- Done: optional live smoke command added as `npm run check:ankiconnect`.
- Not live-verified yet: AnkiConnect was not reachable at `localhost:8765` in
  the current session, so rerun the command when Anki is open.

## P2/P3: Quality Gates And Docs

### 10. Add data contract tests

Affected files:

- `viewer/scripts/`
- future Python tests

Why it matters:

Current checks are useful, but they mostly cover smoke behavior. They do not yet
lock down malformed batch handling, unsafe HTML, audio path normalization, or
loader coercion.

Refactor shape:

- Add viewer fixtures for current batch shapes and malformed batches: done in
  `viewer/scripts/check-loader.js`.
- Add an escaping regression test for generated text in Deliver rows: done in
  `viewer/scripts/check-deliver-rows.js`.
- Add audio-path tests: done in `viewer/scripts/check-audio-assets.js`.
- Add Python tests around metadata parsing, pending planning, contiguous batch
  detection, and schema validation: done in
  `scripts/check-python-contracts.py`.

Validation:

- Done: `cd viewer && npm run check`
- Done: `python3 scripts/check-python-contracts.py`

### 11. Document data surfaces and cleanup rules

Affected files:

- `ARCHITECTURE.md`
- `BACKLOG.md`
- `DATA_SURFACES.md`

Why it matters:

`input/`, `output/`, `audio/`, `viewer/public/audio`, `demos-read-only/`, and
`reference/` have different authority levels. Cleanup or migration work is risky
until those rules are explicit.

Refactor shape:

- Document:
  - source-of-truth files: done.
  - generated but learner-facing files: done.
  - projections/symlinks: done.
  - read-only demos/reference material: done.
  - allowed writers: done.
  - safe cleanup rules: done.
- Promote the existing backlog cleanup item from low polish to architecture
  hygiene: done.

Validation:

- Done: `DATA_SURFACES.md` added and linked from `ARCHITECTURE.md`.
- Done: backlog cleanup item marked done.

### 12. Align romanisation policy

Affected files:

- `generation_prompt_words.txt`
- `generation_prompt_sentences.txt`
- `review_prompt_words.txt`
- `review_prompt_sentences.txt`
- generated output and audio filenames, if migrated later

Why it matters:

The current prompts appear to differ on nasalisation style between word and
sentence cards. That inconsistency leaks into learner-facing text and sometimes
filenames.

Refactor shape:

- Define one project-level romanisation policy: done in `ROMANISATION.md`.
- Align generation prompts and review prompts: done; both generation prompts use
  tilde nasalisation and review prompts flag only nasalisation-policy drift.
- Decide whether existing output should be left as-is, migrated, or normalized
  only on future generation: done; existing output/audio filenames are left as-is,
  and the policy applies to new generation and manual repairs.

Validation:

- Done: prompt scan for contradictory nasalisation instructions.
- Done: `python3 scripts/check-python-contracts.py`.
- Done: `uv run main.py check --type sentences --batch-size 5 --max-batches 1`.
- Remaining before broad regeneration: optional language-teacher review of tiny
  samples.

### 13. Sync runtime documentation after extraction

Affected files:

- `ARCHITECTURE.md`
- `README.md`

Why it matters:

The runtime was split into focused modules, but the user-facing docs still
described the earlier shape where `generate.py` and `process.py` owned most of
the behavior directly.

Refactor shape:

- Document `main.py` as a thin CLI router: done.
- Document `check_report.py`, `batch_planner.py`, `schema_validator.py`,
  `manifest_store.py`, `llm_client.py`, `generation_io.py`,
  `generation_types.py`, and `pipeline_config.py`: done.
- Update the project layout and generation workflow in `README.md`: done.
- Keep `DATA_SURFACES.md` and `ROMANISATION.md` linked from the architecture
  overview: done.

Validation:

- Done: `python3 scripts/check-python-contracts.py`.
- Done: `uv run main.py check --type sentences --batch-size 5 --max-batches 1`.

### 14. Standardize Python contract tests

Affected files:

- `tests/test_python_contracts.py`
- `scripts/check-python-contracts.py`
- `README.md`
- `REFACTORING_ROADMAP.md`

Why it matters:

The contract checks had the right behaviors, but lived as one script. Moving the
assertions into `tests/` makes the suite easier to grow while avoiding a new test
dependency before the project has Python packaging metadata.

Refactor shape:

- Use standard-library `unittest` for now: done.
- Move planner/schema contract assertions into `tests/test_python_contracts.py`:
  done.
- Keep `scripts/check-python-contracts.py` as the stable wrapper command: done.
- Document both the wrapper and direct unittest discovery command: done.

Validation:

- Done: `python3 scripts/check-python-contracts.py`.
- Done: `python3 -m unittest discover -s tests -p 'test_*.py'`.

### 15. Rename Anki sentence metadata field to Topic

Affected files:

- `viewer/src/scripts/anki/sentenceNoteType.js`
- `viewer/src/scripts/anki/fields/sentence.js`
- `viewer/src/scripts/anki/exportService.js`
- `viewer/src/scripts/anki/deckNames.js`
- `viewer/scripts/check-anki-preview.js`

Why it matters:

The project data contract now uses `title` and `subtitle`, and chapter is just
one kind of topic. Keeping the Anki field named `Chapter` made the export model
more specific than the source data.

Refactor shape:

- Rename the sentence Anki field from `Chapter` to `Topic`: done.
- Update the sentence card template to render `{{Topic}}`: done.
- Update sentence field assembly to emit `Topic`: done.
- Attempt a one-way `Chapter -> Topic` field rename during Anki note-type sync:
  done.
- Tighten the Anki preview/export smoke check so `Chapter` cannot reappear as a
  sentence field: done.

Validation:

- Done: `npm run check:anki`.
- Done: `npm run check:audio`.

### 16. Add optional AnkiConnect live smoke command

Affected files:

- `viewer/scripts/check-ankiconnect-live.js`
- `viewer/package.json`
- `viewer/README.md`
- `viewer/src/scripts/anki/exportService.js`

Why it matters:

The static Anki preview/export checks catch field/template drift, but they do
not prove that a local Anki + AnkiConnect instance can sync note types. That
check should be explicit and optional because it depends on a desktop app being
open.

Refactor shape:

- Add `npm run check:ankiconnect`: done.
- Export `ensureWordNoteType()` so the smoke can sync both note types: done.
- Verify word and sentence note fields after sync, including `Topic`: done.
- Keep this out of `npm run check` because it requires Anki to be running: done.

Validation:

- Done: `npm run check:anki`.
- Done: `node --check scripts/check-ankiconnect-live.js`.
- Expected local result while Anki is closed: command exits with a clear
  AnkiConnect-unreachable message.

### 17. Add Deliver deck presets

Affected files:

- `viewer/src/components/tabs/ExportTab.astro`
- `viewer/src/scripts/ui/deckControls.js`
- `viewer/src/styles/partials/deliver.css`
- `BACKLOG.md`

Why it matters:

Quick export now owns whole source/topic sends. Deliver should feel more like a
deliberate custom export surface, and deck presets make common custom targets
easy without hiding the editable deck inputs.

Refactor shape:

- Add word deck presets for vocabulary, review, and custom words: done.
- Add sentence deck presets for sentences, review, and custom sentences: done.
- Wire presets through `deckControls.js` so confirm lines and previews stay in
  sync: done.
- Keep the preset controls compact inside the existing deck config blocks: done.

Validation:

- Done: `npm run check`.
- Done: `node --check src/scripts/ui/deckControls.js`.
- Done: browser smoke on the Deliver tab at `http://127.0.0.1:4322/`.

### 18. Persist Deliver deck choices

Affected files:

- `viewer/src/scripts/ui/deckControls.js`
- `viewer/scripts/check-deck-controls.js`
- `viewer/package.json`
- `BACKLOG.md`

Why it matters:

Deliver is the advanced custom export surface, so custom deck choices should not
reset after every refresh. The controls should remember the last words and
sentences deck targets while still allowing one-click presets.

Refactor shape:

- Persist word and sentence deck inputs to localStorage: done.
- Restore saved deck inputs when Deliver controls are wired: done.
- Save preset choices the same way manual input changes are saved: done.
- Add a focused DOM/storage check for deck control persistence: done.
- Include the deck-control check in `npm run check`: done.

Validation:

- Done: `npm run check`.
- Done: `npm run check:deck-controls`.
- Done: `node --check src/scripts/ui/deckControls.js`.

### 19. Make QA a pre-export gate

Affected files:

- `viewer/src/utils/loadGeneratedData.js`
- `viewer/src/components/tabs/QATab.astro`
- `viewer/src/scripts/ui/qa.js`
- `viewer/src/scripts/ui/exportActions.js`
- `viewer/src/styles/partials/qa.css`
- `viewer/scripts/check-loader.js`
- `viewer/scripts/check-export-gate.js`
- `viewer/package.json`
- `BACKLOG.md`

Why it matters:

QA should be part of the export path, not just a passive report. The viewer
should surface issues, let the user inspect affected cards, track what has been
reviewed locally, and warn before exporting selected cards with unresolved
readiness problems.

Refactor shape:

- Include word audio issues in generated QA data: done.
- Keep sentence audio and exact-token issues: done.
- Add QA filters for all/audio/tokens/words/sentences: done.
- Add jump actions for both word and sentence cards: done.
- Add local reviewed markers without modifying generated data: done.
- Add Deliver export gating with a two-click override for selected issues: done.
- Add an export gate check to the normal viewer suite: done.

Validation:

- Done: `npm run check`.
- Done: `npm run check:loader`.
- Done: `npm run check:export-gate`.
- Done: `node --check src/scripts/ui/qa.js`.
- Done: `node --check src/scripts/ui/exportActions.js`.

### 20. Document and check no-API agent generation

Affected files:

- `NO_API_AGENT_WORKFLOW.md`
- `scripts/check-agent-workflows.py`
- `README.md`
- `BACKLOG.md`

Why it matters:

The project supports API-backed generation, but the local agent workflow should
also be clear when provider credentials are unavailable. Agents should use
project scripts for planning and writing, not manually split inputs or bypass
schema validation.

Refactor shape:

- Add one project-level no-API workflow document: done.
- Document `process.py check` as the planner and `process.py write` as the only
  writer: done.
- Document guardrails around `main.py run`, `--force`, direct output writes, and
  source CSV edits: done.
- Add a static check that keeps the workflow visible in both word and sentence
  generator agents: done.

Validation:

- Done: `python3 scripts/check-agent-workflows.py`.
- Done: `python3 scripts/check-python-contracts.py`.

### 21. Add repair audit and audio backfill commands

Affected files:

- `repair.py`
- `tests/test_python_contracts.py`
- `README.md`
- `BACKLOG.md`

Why it matters:

The viewer now surfaces readiness issues, but command-line repair discovery
should also exist. We need a safe read-only audit for legacy batches and input
quality problems, plus a direct audio backfill command for batches missing
audio paths.

Refactor shape:

- Add `repair.py audit` for output metadata/audio/token gaps: done.
- Add optional sentence input audit for phrase-like drills in sentence CSVs:
  done.
- Add `repair.py audio` wrapper for targeted audio backfill: done.
- Keep audit read-only by default: done.
- Cover audit detection in Python contract tests: done.

Validation:

- Done: `python3 scripts/check-python-contracts.py`.
- Done: `python3 repair.py audit --type sentences --inputs`.

## Recommended Sequence

1. Fix Deliver `innerHTML` row rendering.
2. Document the canonical batch data contract and manifest behavior.
3. Update viewer README, review prompts, and backlog items for the current
   contract.
4. Extract viewer data loading from `index.astro`.
5. Add data contract tests and audio-path tests.
6. Finish the metadata object cleanup.
7. Split Python runtime files along ownership boundaries.
8. Split viewer interaction modules and CSS.
9. Normalize audio asset handling.
10. Separate Anki export services from UI controllers.
11. Decide romanisation policy and migration strategy. Done.
12. Sync architecture and README docs with the extracted runtime. Done.
13. Standardize Python contract tests. Done.
14. Rename Anki sentence metadata field from `Chapter` to `Topic`. Done.
15. Add optional AnkiConnect live smoke command. Done.
16. Add Deliver deck presets. Done.
17. Persist Deliver deck choices. Done.
18. Make QA a pre-export gate. Done.
19. Document and check no-API agent generation. Done.
20. Add repair audit and audio backfill commands. Done.

## Open Decisions

- Whether to create a dedicated `docs/` directory now, or keep roadmap/data
  surface docs at the project root.
- Anki sentence note field label renamed from `Chapter` to `Topic`; sync tries
  a one-way field rename for existing note types and otherwise adds `Topic`.
- Romanisation normalization applies only to future generation and manual repair;
  old output/audio filenames are not migrated just for policy alignment.
- Python contract tests use standard-library `unittest` until the project adds
  Python packaging metadata or needs richer fixtures.
