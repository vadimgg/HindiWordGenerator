# Rust Local Model Design

This document owns implementation design. User-facing command flow and sample
CLI command names, options, and prompts live in `CLI_COMMANDS.md`. User-facing
sentence flow lives in `RUST_LOCAL_MODEL_WORKFLOW.md`.

## Workspace Layout

Use a Rust workspace instead of one large crate. The command surface should feel
like one app, but the implementation should keep project data, model providers,
generation, validation, audio, and export in separate crates.

Planned layout:

```text
crates/
  hindi-cli/          # `hindi` binary, argument parsing, terminal output
  hindi-config/       # project config, model roles, path discovery
  hindi-core/         # shared domain types, errors, IDs, run metadata
  hindi-source/       # YAML source parsing, normalization, source QA edits
  hindi-planner/      # pending/done/deferred planning from input + output
  hindi-schema/       # accepted JSON schemas, validators, token builder
  hindi-writer/       # atomic append-only writes, collision checks, temp files
  hindi-models/       # provider traits, Ollama implementation, model switching
  hindi-sentences/    # sentence generation orchestration
  hindi-audio/        # TTS/audio backfill boundary
  hindi-export/       # shared Anki/export builders for CLI and viewer
  hindi-transcribe/   # future Whisper/transcript pipeline
```

Dependency direction should stay boring and one-way:

```text
hindi-cli
  -> feature crates such as hindi-sentences, hindi-audio, hindi-export
feature crates
  -> hindi-core, hindi-config, hindi-source, hindi-planner, hindi-schema, hindi-models
foundation crates
  -> no dependency on hindi-cli
```

Rules:

- `hindi-cli` owns only command parsing, prompts, progress output, and exit
  codes.
- `hindi-core` owns shared structs and error types, but not file walking,
  provider calls, or command behavior.
- `hindi-source` reads and writes YAML source. No model calls.
- `hindi-planner` decides what is pending. No model calls and no accepted
  writes.
- `hindi-schema` validates candidate/accepted JSON and builds deterministic
  `tokens[]`. No provider calls.
- `hindi-writer` owns accepted-output writes, temp-file-and-rename behavior,
  collision checks, and append-only guarantees. Other crates do not write
  `output/` directly.
- `hindi-models` owns Ollama/provider details and model role switching. It does
  not know sentence-card schema details.
- `hindi-sentences` coordinates planning, model calls, validation, run reports,
  and write requests for sentence cards. It delegates accepted-output writes to
  `hindi-writer`.
- `hindi-audio`, `hindi-export`, and `hindi-transcribe` stay optional feature
  crates so they do not leak concerns into sentence generation.
- `hindi-export` should be shared by CLI export and viewer export so the web app
  does not grow a duplicate export format.

Start with the crates needed for M1/M2 only: `hindi-cli`, `hindi-config`,
`hindi-core`, `hindi-source`, `hindi-planner`, and `hindi-models`. Add
`hindi-schema`, `hindi-writer`, and `hindi-sentences` when generation starts.

## Generation Shape

Local models should receive structured YAML source context:

```text
hindi + romanisation + english
```

Sentence experiments showed that local models behave better with small, staged
tasks than with the old all-in-one API prompt.

Recommended staged path:

1. Parse and normalize one YAML source item.
2. Ask the model for sentence-level fields:
   - `hindi`
   - `romanisation`
   - `english`
   - `literal`
   - `register`
3. Ask the model for `words[]`.
4. Build `tokens[]` deterministically from the final `words[]` array.
5. Merge sentence-level fields, `words[]`, `tokens[]`, tags, and metadata into
   the candidate sentence-card batch.
6. Validate before any accepted write.
7. Optionally evaluate with a reviewer agent or evaluator model.

The first Rust spike may use one prompt if that gets us to validation faster,
but command/module boundaries should not assume generation is one model call
forever.

## Reviewable Run Folders

Every generation run should leave a durable run folder, even when it writes
validated output directly to `output/sentences/`. The run folder is where model
outputs, timing, validation results, and model identity live. Accepted card JSON
stays learner-facing and should not carry audit noise. The one exception is
durable source lineage: accepted sentence cards should carry `source_ref.file`,
`source_ref.item_id`, and `source_ref.fingerprint` so source/output audit does
not depend on run folders being kept forever.

Direct mode writes validated output immediately after validation. Review mode
creates the same run folder but does not write accepted output until `hindi
sentences accept`.

Planned command:

```bash
hindi sentences generate --model ollama:translategemma:12b --max-batches 1 --review
```

Planned layout:

```text
runs/sentences/<timestamp>_<model-slug>/
  plan.json
  model_outputs/
  validated/
  rejected/
  report.json
```

`model-slug` must be filesystem-safe. Replace provider separators, colons,
slashes, whitespace, and punctuation with underscores. For example:

```text
ollama:translategemma:12b -> ollama_translategemma_12b
```

Follow-up commands:

```bash
hindi sentences review runs/sentences/<run-id>
hindi sentences accept runs/sentences/<run-id>
```

`accept` must fail if a target output file already exists. A future explicit
repair/regenerate command can replace selected files, but normal `accept` is
append-only.

Run folders should preserve:

- raw model output
- staged partial output
- validation errors
- timing and model metadata
- accepted/rejected status

## Metadata

Generation metadata stays outside accepted card JSON unless the schema is
deliberately expanded.

Use `runs/.../report.json` for both direct and review-mode runs. CLI summaries
are useful, but they are not durable enough to be the only audit trail.

Capture:

- requested model
- loaded Ollama model
- prompt identifier or hash
- input file and item indexes
- planned output target
- model call timings
- validation result
- accepted/rejected status
- source QA decisions and applied source corrections, when source QA is enabled

## Failure Storage

The generator fails closed:

- malformed JSON goes to `runs/.../rejected/` when review mode exists
- schema validation failures go to `runs/.../rejected/`
- partial staged outputs stay in `runs/.../model_outputs/`
- learner-facing `output/sentences/` is touched only after validation succeeds

Before review mode exists, failures should print a clear error and write no
accepted output.

## Provider Boundary

Ollama sits behind a provider boundary. Generation code should not know about
HTTP details, OpenAI-compatible routes, or `ollama ps` parsing.

The provider exposes:

- loaded model discovery
- model mismatch check
- response test
- prompt call with timeout
- structured response and timing

## Model Role Routing

Generation should request models by role, not by hard-coded model names in the
business logic. Default role assignments live in `OLLAMA_MODEL_STRATEGY.md`.

Core roles:

- `sentence_source_qa`
- `sentence_generation`
- `word_draft`
- `word_quality`

Source QA is an optional separate command, not a hidden generation sub-step. It
may suggest YAML corrections, but it must not rewrite source files without an
explicit interactive confirmation or a clearly named non-interactive flag such
as `--apply`.

CLI flags:

```text
--require-source-qa                   require latest matching clean source-QA report
--word-mode draft|quality             choose word model role later
```

`--model <provider:model>` is intentionally not part of the normal user
workflow. Model choices live in config. A future expert/debug flag may override
a role, but the primary CLI should stay workflow-based.

The provider layer should resolve a role to a concrete model string from config,
check loaded models before every model-using stage, and return a clear mismatch
error before any prompt call.

The CLI should have explicit Ollama-management commands because the workflow may
need several specialist models and the target laptop has finite RAM:

```bash
hindi models status
hindi models prepare sentences
hindi models prepare source-qa
```

Normal generation should not silently stop or replace a loaded model. If the
needed role model is not loaded, or a different model is already loaded and
`max_loaded_models = 1`, fail with the next command:

```bash
hindi models prepare sentences
```

`hindi models prepare <workflow>` is the explicit public command where model
start/stop can happen. It should show the user-facing workflow being prepared,
whether another model will be stopped, and ask for confirmation unless
`--allow-model-switch` is set. Internal role names such as
`sentence_generation` may appear in config and run reports, but should not be
required in normal user commands.

When a command uses more than one model role, run stages sequentially and record
the loaded model before each stage:

```text
source QA       -> sentence_source_qa
generation      -> sentence_generation
validation      -> no model
write           -> no model
```

Run metadata should include requested model, loaded model, whether the model was
started by the CLI, timing, timeout, and mismatch failures for each role.

In the normal path, model switching happens only in `hindi models prepare ...`.
Generation and source-QA commands check readiness and fail with the next prepare
command if the wrong model is loaded.

Model tags such as `gemma4:latest` are floating tags. The experiment notes may
drift if Ollama updates what `latest` points to. Run metadata must record the
model name and Ollama model ID from `ollama ps` when available. If a stable
model tag or digest becomes available, prefer pinning it in config.

## Transcription Boundary

Transcription is optional and separate from YAML sentence generation. The
planned backend is local Whisper or a Whisper-compatible local model.

Owned paths are defined in `DATA_SURFACES.md`:

- `media/input/`
- `transcripts/raw/`
- `transcripts/reviewed/`
- `transcripts/references/`

The transcription module owns media discovery, transcript generation, optional
reference alignment, reviewed transcript storage, and stable segment IDs.

Sentence generation may later consume reviewed transcript segments. Accepted
cards still land in `output/sentences/` and keep only a lightweight transcript
reference when needed.

## TTS Boundary

The archived Python audio command used `gTTS`, the Google Text-to-Speech Python
package. Rust should hide audio generation behind a TTS boundary so the backend
can change later.

Initial Rust plan:

- keep Google/gTTS-style service-backed audio as the compatibility target
- define an interface that can later call a local TTS backend or HTTP service
- generate deterministic audio filenames
- write relative `audio` paths in accepted JSON
- support audio backfill without rerunning sentence generation
- write MP3s through a temporary path, validate the result exists, then update
  accepted JSON through a temp-file-and-rename flow
- skip cards that already have both an MP3 and a matching `audio` field

Local TTS model selection is deliberately not part of the first sentence
generation milestone.
