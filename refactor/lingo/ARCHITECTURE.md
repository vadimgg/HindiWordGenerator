# Lingo architecture

> **Status:** clean-slate implementation contract.  
> This document describes the system to build. It is not a migration plan, and
> it does not preserve prototype commands, formats, names, aliases, or internal APIs.

## 1. Product boundary

`lingo` is a local, language-agnostic CLI for one directional workflow:

```text
raw text -> reviewed source YAML -> enriched card JSON -> checked cards
         -> audio -> portable package and/or Anki export
```

ChatGPT or Claude is a manual external collaborator. `lingo` prepares a complete
prompt packet, the user runs it in the model UI of their choice, and `lingo`
parses and validates the pasted reply. There is no model-server abstraction,
Ollama integration, engine flag, generic provider payload, or autonomous LLM call.

The first shipped card type is `sentences`. Language-specific facts live in a
profile; durable Rust models use language-neutral names such as `target`,
`romanisation`, and `english`.

## 2. Non-negotiable rules for implementation agents

1. **No migration or compatibility code.** Do not add old `hindi` commands,
   `sentences <verb>` nesting, legacy fields, dual readers, aliases, or shims.
2. **One owner per concept.** Domain invariants live in `lingo-domain`; use-case
   policy lives in `lingo-application`; filesystem, prompt, provider, artifact,
   and presentation mechanics live in their adapters.
3. **Dependency arrows are enforced, not aspirational.** No lower/inward crate
   imports `lingo-cli`, and `lingo-application` imports no concrete adapter.
4. **Templates are guidance, not schema authority.** Editable prompt templates
   contain language-specific instructions. Rust appends canonical output
   contracts and validates replies.
5. **Canonical data is written only after construction and validation.** A bad
   model reply may produce diagnostics and run files, never partial source/card data.
6. **Use typed values at boundaries.** IDs, stages, modes, backend names,
   fingerprints, display lead, and diagnostic codes are not raw strings inside
   behavior code.
7. **No speculative abstraction.** No event bus, plugin system, service locator,
   hidden self-registration, generic repository framework, or generic JSON operation API.
8. **Public functions read as intent.** Keep orchestration visible; extract only
   helpers that own a concept or difficult mechanics.
9. **Reports carry facts; edges format them.** Services do not emit ANSI text or
   transport DTOs.
10. **Generated artifacts are derived.** Runs, status, viewer DTOs, packages,
    exports, and manifests never become the authority for source/card data.

## 3. Source-of-truth map

| Fact | Canonical authority | Derived/read-only surfaces |
| --- | --- | --- |
| Learner defaults | global config | resolved context and doctor output |
| Language facts and prompt overrides | selected profile layers | rendered packet |
| Reviewed source items | `input/sentences/<batch>.yaml` (`lingo.source/v1`) | build packet, status |
| Enriched cards and source lineage | `output/sentences/<batch>.json` (`lingo.cards/v1`) | viewer DTO, package, Anki |
| Audio relationship | `Card.audio` reference in canonical card JSON | package/Anki media map |
| Audio bytes | `audio/sentences/.../*.mp3` derived artifact | viewer/package/export |
| Prompt/reply history | advisory `runs/` journal | debugging/reproduction only |
| Package/export | generated publication artifact | external consumers |

When a derived copy disagrees with canonical source/card files, canonical files win
and the derived surface is rebuilt.

## 4. Crate boundaries

| Crate | Owner | Why it is a crate |
| --- | --- | --- |
| `lingo-domain` | Canonical values, aggregates, fingerprints, validation | Enforces a dependency-free semantic core |
| `lingo-application` | Use cases, ports, typed reports, next-action policy | Keeps workflow policy independent of mechanisms and testable with meaningful fakes |
| `lingo-workspace-fs` | File-backed workspace/config/profile/run adapter | Isolates path, TOML/YAML/JSON, XDG, and atomic-file dependencies |
| `lingo-prompt` | Packet rendering and untrusted reply parsing | Isolates Handlebars and lenient boundary DTOs from canonical models |
| `lingo-audio` | gTTS/ElevenLabs providers, explicit catalog, fallback | Two real implementations justify a backend contract and isolate HTTP/process dependencies |
| `lingo-artifacts` | Portable package and Anki publication formats | Owns generated format vocabulary, checksums, zip/APKG, and directory staging |
| `lingo-cli` | Arguments, composition, interaction, output, viewer server | The only executable edge and the only place concrete adapters are wired together |

There is deliberately no `core`, `common`, `utils`, `plugins`, `models`, or
`services` junk-drawer crate.

## 5. Allowed dependency graph

```text
                         lingo-cli
           +----------------+----------------+
           |                |                |
           v                v                v
 lingo-workspace-fs   lingo-prompt      lingo-audio      lingo-artifacts
           \                |                /                 /
            \               |               /                 /
             +--------------+--------------+-----------------+
                            v
                    lingo-application
                            v
                       lingo-domain
```

All adapter crates implement ports owned by `lingo-application`. Adapters do not
import one another. `lingo-cli` is the composition root.

### Forbidden arrows

```text
lingo-domain        -X-> any workspace crate
lingo-application   -X-> lingo-workspace-fs | lingo-prompt | lingo-audio | lingo-artifacts | lingo-cli
lingo-workspace-fs  -X-> lingo-prompt | lingo-audio | lingo-artifacts | lingo-cli
lingo-prompt        -X-> lingo-workspace-fs | lingo-audio | lingo-artifacts | lingo-cli
lingo-audio         -X-> lingo-workspace-fs | lingo-prompt | lingo-artifacts | lingo-cli
lingo-artifacts     -X-> lingo-workspace-fs | lingo-prompt | lingo-audio | lingo-cli
apps/viewer         -X-> filesystem paths or canonical file formats
```

Do not hide a forbidden edge behind a re-export, feature flag, macro, build
script, generated code, or compatibility module.

## 6. Runtime composition

```rust
fn compose(env: ProcessEnvironment) -> Result<AppContext, StartupError> {
    let workspace = FsWorkspace::open_or_target(env.cwd, env.xdg_config_home)?;
    let prompts = HandlebarsPromptEngine::strict();
    let audio = AudioCatalogBuilder::new()
        .add_gtts(env.uv_path)
        .add_elevenlabs_if_configured(env.http_client, env.elevenlabs_key)
        .build()?;
    let artifacts = ArtifactPublishers::new();

    Ok(AppContext { workspace, prompts, audio, artifacts, environment: env })
}
```

The catalog is explicit and duplicate-checked. Profiles are explicit data assets,
not executable plugins. There is no linker-time registration or runtime code scan.

## 7. Command-to-use-case map

| CLI command | Application owner | State change |
| --- | --- | --- |
| `lingo init` | `application::init` | create missing workspace files only |
| `lingo import` | `application::import` | prepare run; on apply, create source YAML |
| `lingo build` | `application::build` | prepare run; on apply, create card JSON |
| `lingo check` | `application::check` | none |
| `lingo audio` | `application::audio` | write audio, then replace card reference |
| `lingo package` | `application::package` | publish derived package |
| `lingo export` | `application::export` | publish derived Anki artifact |
| `lingo status` | `application::status` | none |
| `lingo lang` | `application::lang` | list/show/which, or create explicit override |
| `lingo doctor` | `application::doctor` | none |
| `lingo viewer` | `application::viewer` + CLI server | none |

Command modules parse, call one use case, and render. They do not call other
commands, recursively invoke the binary, or implement workflow policy.

## 8. Prompt packet boundary

`import` and `build` use two explicit application phases:

```text
prepare -> render and journal packet -> user/model interaction -> apply
```

- `prepare_*` may write advisory run files, but never canonical input/output.
- `--print` returns after preparation and writes no canonical data.
- `--apply <file>` skips editor/clipboard and passes exact bytes to `apply_*`.
- Interactive editor/clipboard behavior belongs to `lingo-cli::interaction`.
- Reply adapters may remove one optional code fence; they do not invent fields,
  repair semantics, or accept prose surrounding multiple documents.
- Application acceptance constructs typed domain values, runs deterministic
  validation, then calls atomic create/replace storage operations.

Editable templates contain language guidance only. The Rust packet builder owns
format tags, field vocabulary, lineage rules, and worked examples.

## 9. Canonical models

```rust
pub struct SourceBatch {
    format: SourceFormat,          // lingo.source/v1
    batch: BatchId,
    title: SourceTitle,
    subtitle: Option<SourceSubtitle>,
    items: Vec<SourceItem>,
}

pub struct Card {
    id: CardId,
    target: TargetText,
    romanisation: Option<Romanisation>,
    english: Gloss,
    literal: Gloss,
    register: Register,
    tokens: Vec<CardToken>,
    words: Vec<Word>,
    tags: CardTags,
    audio: Option<AudioRef>,
    source: SourceRef,             // batch + item + fingerprint, never a raw path
}
```

Aggregates have private fields and named constructors. Boundary DTOs live in
adapter crates and must call these constructors. Empty strings are not used to
represent absence.

### Stable identity

- `BatchId` is explicit and validated.
- `SourceItemId` is derived deterministically from normalized source content plus
  a duplicate ordinal, so reordering does not churn unchanged items.
- `CardId` composes `BatchId + SourceItemId`.
- Source lineage fingerprints normalize Unicode to NFC and collapse whitespace,
  then use the maintained `sha2` implementation.

## 10. Atomicity and failure sequencing

### Canonical file acceptance

```text
parse untrusted bytes
-> construct typed aggregate
-> deterministic validation
-> serialize canonical format
-> write unique sibling temp with create_new
-> flush + sync
-> rename
```

Create and replace are separate named operations. Create never silently
replaces; replace keeps the old file until the new file is durable.

### Audio

```text
synthesize bytes -> write audio atomically -> attach AudioRef -> replace card JSON
```

If card replacement fails, an orphan audio file is harmless derived data and can
be detected later; a card must never reference bytes that were not written.
Fallback runs only for retryable provider failures and at most once.

### Package/export

Build the complete plan, write a sibling staging directory, verify all paths and
checksums, write the manifest last, read it back, then swap the destination.
Never expose a half-built publication artifact.

## 11. Configuration and profile rules

Resolution order, last defined value wins:

```text
built-in defaults
-> global config
-> selected built-in/global profile
-> deck config
-> deck profile/prompt override
-> typed command override
```

Every resolved field carries provenance for `doctor` and `lang which`.
Configuration may select known Rust-owned choices; it may not define arbitrary
validation code. Secrets are never stored in TOML. Config stores an environment
variable name; the composition edge reads the value into a redacted secret type.

## 12. Testing and architecture evidence

Required CI sequence:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo deny check
npm --prefix apps/viewer ci
npm --prefix apps/viewer run check
npm --prefix apps/viewer run build
```

Evidence expectations:

- value objects: valid, invalid, and round-trip tests;
- domain validation: structured code/location assertions;
- application use cases: meaningful fake ports and typed report assertions;
- adapters: actual codec/filesystem/provider contract tests;
- CLI: help/output/exit-code smoke tests;
- architecture: Cargo graph forbidden-edge test;
- package/export: read-back and integrity verification;
- no tests or fixtures for prototype compatibility.

## 13. Reusable ideas from the attached implementation

The attached code is a reference library of mechanics, not an API or format to
preserve. The following ideas are worth re-implementing under the owners above:

- sibling-temp atomic writes with collision and missing-parent tests;
- Unicode normalization before source fingerprints;
- typed command reports rather than printing deep inside behavior;
- a real TTS backend seam, because gTTS and ElevenLabs differ materially;
- staged package publication with explicit missing-audio detection;
- deterministic source/card lineage validation.

Do **not** carry forward the giant manual CLI parser, Hindi-specific field names,
Ollama/eval surface, hand-written SHA-256, mixed orchestration/formatting modules,
or compatibility aliases.

## 14. Construction order (not migration)

1. Bootstrap manifests, lints, and architecture test.
2. Implement domain IDs, values, source/card aggregates, fingerprints, diagnostics,
   and validation.
3. Define application ports/reports and implement `init`, `import`, `build`, and
   `check` with fakes.
4. Implement filesystem config/profile/codecs/storage and atomic writes.
5. Implement prompt rendering/reply parsing and CLI packet interaction.
6. Implement audio catalog/providers and audio use case.
7. Implement package/Anki artifact publishers.
8. Implement status, lang, doctor, viewer server, and static viewer.
9. Add full CLI and pipeline integration tests.

Every step targets only the clean-slate v1 contracts in this document.

## 15. Complete linked file structure

- [ `Cargo.toml` ](docs/architecture/files/Cargo.toml.md)
- [ `rust-toolchain.toml` ](docs/architecture/files/rust-toolchain.toml.md)
- [ `deny.toml` ](docs/architecture/files/deny.toml.md)
- [ `clippy.toml` ](docs/architecture/files/clippy.toml.md)
- `crates/lingo-domain/`
  - [ `crates/lingo-domain/Cargo.toml` ](docs/architecture/files/crates/lingo-domain/Cargo.toml.md) — `Cargo.toml`
  - [ `crates/lingo-domain/src/audio.rs` ](docs/architecture/files/crates/lingo-domain/src/audio.rs.md) — `src/audio.rs`
  - [ `crates/lingo-domain/src/card.rs` ](docs/architecture/files/crates/lingo-domain/src/card.rs.md) — `src/card.rs`
  - [ `crates/lingo-domain/src/diagnostic.rs` ](docs/architecture/files/crates/lingo-domain/src/diagnostic.rs.md) — `src/diagnostic.rs`
  - [ `crates/lingo-domain/src/fingerprint.rs` ](docs/architecture/files/crates/lingo-domain/src/fingerprint.rs.md) — `src/fingerprint.rs`
  - [ `crates/lingo-domain/src/ids.rs` ](docs/architecture/files/crates/lingo-domain/src/ids.rs.md) — `src/ids.rs`
  - [ `crates/lingo-domain/src/language.rs` ](docs/architecture/files/crates/lingo-domain/src/language.rs.md) — `src/language.rs`
  - [ `crates/lingo-domain/src/lib.rs` ](docs/architecture/files/crates/lingo-domain/src/lib.rs.md) — `src/lib.rs`
  - [ `crates/lingo-domain/src/pipeline.rs` ](docs/architecture/files/crates/lingo-domain/src/pipeline.rs.md) — `src/pipeline.rs`
  - [ `crates/lingo-domain/src/source.rs` ](docs/architecture/files/crates/lingo-domain/src/source.rs.md) — `src/source.rs`
  - [ `crates/lingo-domain/src/validation.rs` ](docs/architecture/files/crates/lingo-domain/src/validation.rs.md) — `src/validation.rs`
- `crates/lingo-application/`
  - [ `crates/lingo-application/Cargo.toml` ](docs/architecture/files/crates/lingo-application/Cargo.toml.md) — `Cargo.toml`
  - [ `crates/lingo-application/src/audio.rs` ](docs/architecture/files/crates/lingo-application/src/audio.rs.md) — `src/audio.rs`
  - [ `crates/lingo-application/src/build.rs` ](docs/architecture/files/crates/lingo-application/src/build.rs.md) — `src/build.rs`
  - [ `crates/lingo-application/src/check.rs` ](docs/architecture/files/crates/lingo-application/src/check.rs.md) — `src/check.rs`
  - [ `crates/lingo-application/src/doctor.rs` ](docs/architecture/files/crates/lingo-application/src/doctor.rs.md) — `src/doctor.rs`
  - [ `crates/lingo-application/src/export.rs` ](docs/architecture/files/crates/lingo-application/src/export.rs.md) — `src/export.rs`
  - [ `crates/lingo-application/src/import.rs` ](docs/architecture/files/crates/lingo-application/src/import.rs.md) — `src/import.rs`
  - [ `crates/lingo-application/src/init.rs` ](docs/architecture/files/crates/lingo-application/src/init.rs.md) — `src/init.rs`
  - [ `crates/lingo-application/src/lang.rs` ](docs/architecture/files/crates/lingo-application/src/lang.rs.md) — `src/lang.rs`
  - [ `crates/lingo-application/src/lib.rs` ](docs/architecture/files/crates/lingo-application/src/lib.rs.md) — `src/lib.rs`
  - [ `crates/lingo-application/src/package.rs` ](docs/architecture/files/crates/lingo-application/src/package.rs.md) — `src/package.rs`
  - [ `crates/lingo-application/src/ports.rs` ](docs/architecture/files/crates/lingo-application/src/ports.rs.md) — `src/ports.rs`
  - [ `crates/lingo-application/src/report.rs` ](docs/architecture/files/crates/lingo-application/src/report.rs.md) — `src/report.rs`
  - [ `crates/lingo-application/src/status.rs` ](docs/architecture/files/crates/lingo-application/src/status.rs.md) — `src/status.rs`
  - [ `crates/lingo-application/src/viewer.rs` ](docs/architecture/files/crates/lingo-application/src/viewer.rs.md) — `src/viewer.rs`
- `crates/lingo-workspace-fs/`
  - [ `crates/lingo-workspace-fs/Cargo.toml` ](docs/architecture/files/crates/lingo-workspace-fs/Cargo.toml.md) — `Cargo.toml`
  - [ `crates/lingo-workspace-fs/assets/profiles/hindi/profile.toml` ](docs/architecture/files/crates/lingo-workspace-fs/assets/profiles/hindi/profile.toml.md) — `assets/profiles/hindi/profile.toml`
  - [ `crates/lingo-workspace-fs/assets/profiles/hindi/prompts/build.md.hbs` ](docs/architecture/files/crates/lingo-workspace-fs/assets/profiles/hindi/prompts/build.md.hbs.md) — `assets/profiles/hindi/prompts/build.md.hbs`
  - [ `crates/lingo-workspace-fs/assets/profiles/hindi/prompts/import.md.hbs` ](docs/architecture/files/crates/lingo-workspace-fs/assets/profiles/hindi/prompts/import.md.hbs.md) — `assets/profiles/hindi/prompts/import.md.hbs`
  - [ `crates/lingo-workspace-fs/src/atomic_file.rs` ](docs/architecture/files/crates/lingo-workspace-fs/src/atomic_file.rs.md) — `src/atomic_file.rs`
  - [ `crates/lingo-workspace-fs/src/codecs.rs` ](docs/architecture/files/crates/lingo-workspace-fs/src/codecs.rs.md) — `src/codecs.rs`
  - [ `crates/lingo-workspace-fs/src/config.rs` ](docs/architecture/files/crates/lingo-workspace-fs/src/config.rs.md) — `src/config.rs`
  - [ `crates/lingo-workspace-fs/src/error.rs` ](docs/architecture/files/crates/lingo-workspace-fs/src/error.rs.md) — `src/error.rs`
  - [ `crates/lingo-workspace-fs/src/layout.rs` ](docs/architecture/files/crates/lingo-workspace-fs/src/layout.rs.md) — `src/layout.rs`
  - [ `crates/lingo-workspace-fs/src/lib.rs` ](docs/architecture/files/crates/lingo-workspace-fs/src/lib.rs.md) — `src/lib.rs`
  - [ `crates/lingo-workspace-fs/src/profiles.rs` ](docs/architecture/files/crates/lingo-workspace-fs/src/profiles.rs.md) — `src/profiles.rs`
  - [ `crates/lingo-workspace-fs/src/root.rs` ](docs/architecture/files/crates/lingo-workspace-fs/src/root.rs.md) — `src/root.rs`
  - [ `crates/lingo-workspace-fs/src/runs.rs` ](docs/architecture/files/crates/lingo-workspace-fs/src/runs.rs.md) — `src/runs.rs`
  - [ `crates/lingo-workspace-fs/src/scan.rs` ](docs/architecture/files/crates/lingo-workspace-fs/src/scan.rs.md) — `src/scan.rs`
  - [ `crates/lingo-workspace-fs/src/store.rs` ](docs/architecture/files/crates/lingo-workspace-fs/src/store.rs.md) — `src/store.rs`
- `crates/lingo-prompt/`
  - [ `crates/lingo-prompt/Cargo.toml` ](docs/architecture/files/crates/lingo-prompt/Cargo.toml.md) — `Cargo.toml`
  - [ `crates/lingo-prompt/src/build_reply.rs` ](docs/architecture/files/crates/lingo-prompt/src/build_reply.rs.md) — `src/build_reply.rs`
  - [ `crates/lingo-prompt/src/error.rs` ](docs/architecture/files/crates/lingo-prompt/src/error.rs.md) — `src/error.rs`
  - [ `crates/lingo-prompt/src/import_reply.rs` ](docs/architecture/files/crates/lingo-prompt/src/import_reply.rs.md) — `src/import_reply.rs`
  - [ `crates/lingo-prompt/src/lib.rs` ](docs/architecture/files/crates/lingo-prompt/src/lib.rs.md) — `src/lib.rs`
  - [ `crates/lingo-prompt/src/packet.rs` ](docs/architecture/files/crates/lingo-prompt/src/packet.rs.md) — `src/packet.rs`
  - [ `crates/lingo-prompt/src/render.rs` ](docs/architecture/files/crates/lingo-prompt/src/render.rs.md) — `src/render.rs`
- `crates/lingo-audio/`
  - [ `crates/lingo-audio/Cargo.toml` ](docs/architecture/files/crates/lingo-audio/Cargo.toml.md) — `Cargo.toml`
  - [ `crates/lingo-audio/src/backend.rs` ](docs/architecture/files/crates/lingo-audio/src/backend.rs.md) — `src/backend.rs`
  - [ `crates/lingo-audio/src/catalog.rs` ](docs/architecture/files/crates/lingo-audio/src/catalog.rs.md) — `src/catalog.rs`
  - [ `crates/lingo-audio/src/elevenlabs.rs` ](docs/architecture/files/crates/lingo-audio/src/elevenlabs.rs.md) — `src/elevenlabs.rs`
  - [ `crates/lingo-audio/src/error.rs` ](docs/architecture/files/crates/lingo-audio/src/error.rs.md) — `src/error.rs`
  - [ `crates/lingo-audio/src/fallback.rs` ](docs/architecture/files/crates/lingo-audio/src/fallback.rs.md) — `src/fallback.rs`
  - [ `crates/lingo-audio/src/gtts.rs` ](docs/architecture/files/crates/lingo-audio/src/gtts.rs.md) — `src/gtts.rs`
  - [ `crates/lingo-audio/src/lib.rs` ](docs/architecture/files/crates/lingo-audio/src/lib.rs.md) — `src/lib.rs`
  - [ `crates/lingo-audio/src/model.rs` ](docs/architecture/files/crates/lingo-audio/src/model.rs.md) — `src/model.rs`
- `crates/lingo-artifacts/`
  - [ `crates/lingo-artifacts/Cargo.toml` ](docs/architecture/files/crates/lingo-artifacts/Cargo.toml.md) — `Cargo.toml`
  - [ `crates/lingo-artifacts/src/anki.rs` ](docs/architecture/files/crates/lingo-artifacts/src/anki.rs.md) — `src/anki.rs`
  - [ `crates/lingo-artifacts/src/checksum.rs` ](docs/architecture/files/crates/lingo-artifacts/src/checksum.rs.md) — `src/checksum.rs`
  - [ `crates/lingo-artifacts/src/error.rs` ](docs/architecture/files/crates/lingo-artifacts/src/error.rs.md) — `src/error.rs`
  - [ `crates/lingo-artifacts/src/lib.rs` ](docs/architecture/files/crates/lingo-artifacts/src/lib.rs.md) — `src/lib.rs`
  - [ `crates/lingo-artifacts/src/manifest.rs` ](docs/architecture/files/crates/lingo-artifacts/src/manifest.rs.md) — `src/manifest.rs`
  - [ `crates/lingo-artifacts/src/model.rs` ](docs/architecture/files/crates/lingo-artifacts/src/model.rs.md) — `src/model.rs`
  - [ `crates/lingo-artifacts/src/package.rs` ](docs/architecture/files/crates/lingo-artifacts/src/package.rs.md) — `src/package.rs`
  - [ `crates/lingo-artifacts/src/staging.rs` ](docs/architecture/files/crates/lingo-artifacts/src/staging.rs.md) — `src/staging.rs`
- `crates/lingo-cli/`
  - [ `crates/lingo-cli/Cargo.toml` ](docs/architecture/files/crates/lingo-cli/Cargo.toml.md) — `Cargo.toml`
  - [ `crates/lingo-cli/src/cli.rs` ](docs/architecture/files/crates/lingo-cli/src/cli.rs.md) — `src/cli.rs`
  - [ `crates/lingo-cli/src/commands/audio.rs` ](docs/architecture/files/crates/lingo-cli/src/commands/audio.rs.md) — `src/commands/audio.rs`
  - [ `crates/lingo-cli/src/commands/build.rs` ](docs/architecture/files/crates/lingo-cli/src/commands/build.rs.md) — `src/commands/build.rs`
  - [ `crates/lingo-cli/src/commands/check.rs` ](docs/architecture/files/crates/lingo-cli/src/commands/check.rs.md) — `src/commands/check.rs`
  - [ `crates/lingo-cli/src/commands/doctor.rs` ](docs/architecture/files/crates/lingo-cli/src/commands/doctor.rs.md) — `src/commands/doctor.rs`
  - [ `crates/lingo-cli/src/commands/export.rs` ](docs/architecture/files/crates/lingo-cli/src/commands/export.rs.md) — `src/commands/export.rs`
  - [ `crates/lingo-cli/src/commands/import.rs` ](docs/architecture/files/crates/lingo-cli/src/commands/import.rs.md) — `src/commands/import.rs`
  - [ `crates/lingo-cli/src/commands/init.rs` ](docs/architecture/files/crates/lingo-cli/src/commands/init.rs.md) — `src/commands/init.rs`
  - [ `crates/lingo-cli/src/commands/lang.rs` ](docs/architecture/files/crates/lingo-cli/src/commands/lang.rs.md) — `src/commands/lang.rs`
  - [ `crates/lingo-cli/src/commands/mod.rs` ](docs/architecture/files/crates/lingo-cli/src/commands/mod.rs.md) — `src/commands/mod.rs`
  - [ `crates/lingo-cli/src/commands/package.rs` ](docs/architecture/files/crates/lingo-cli/src/commands/package.rs.md) — `src/commands/package.rs`
  - [ `crates/lingo-cli/src/commands/status.rs` ](docs/architecture/files/crates/lingo-cli/src/commands/status.rs.md) — `src/commands/status.rs`
  - [ `crates/lingo-cli/src/commands/viewer.rs` ](docs/architecture/files/crates/lingo-cli/src/commands/viewer.rs.md) — `src/commands/viewer.rs`
  - [ `crates/lingo-cli/src/composition.rs` ](docs/architecture/files/crates/lingo-cli/src/composition.rs.md) — `src/composition.rs`
  - [ `crates/lingo-cli/src/exit.rs` ](docs/architecture/files/crates/lingo-cli/src/exit.rs.md) — `src/exit.rs`
  - [ `crates/lingo-cli/src/interaction.rs` ](docs/architecture/files/crates/lingo-cli/src/interaction.rs.md) — `src/interaction.rs`
  - [ `crates/lingo-cli/src/main.rs` ](docs/architecture/files/crates/lingo-cli/src/main.rs.md) — `src/main.rs`
  - [ `crates/lingo-cli/src/output.rs` ](docs/architecture/files/crates/lingo-cli/src/output.rs.md) — `src/output.rs`
  - [ `crates/lingo-cli/src/viewer_server.rs` ](docs/architecture/files/crates/lingo-cli/src/viewer_server.rs.md) — `src/viewer_server.rs`
  - [ `crates/lingo-cli/tests/architecture.rs` ](docs/architecture/files/crates/lingo-cli/tests/architecture.rs.md) — `tests/architecture.rs`
  - [ `crates/lingo-cli/tests/cli_smoke.rs` ](docs/architecture/files/crates/lingo-cli/tests/cli_smoke.rs.md) — `tests/cli_smoke.rs`
  - [ `crates/lingo-cli/tests/pipeline_e2e.rs` ](docs/architecture/files/crates/lingo-cli/tests/pipeline_e2e.rs.md) — `tests/pipeline_e2e.rs`
  - [ `crates/lingo-cli/tests/support/mod.rs` ](docs/architecture/files/crates/lingo-cli/tests/support/mod.rs.md) — `tests/support/mod.rs`
- `apps/viewer/`
  - [ `apps/viewer/astro.config.mjs` ](docs/architecture/files/apps/viewer/astro.config.mjs.md) — `astro.config.mjs`
  - [ `apps/viewer/package.json` ](docs/architecture/files/apps/viewer/package.json.md) — `package.json`
  - [ `apps/viewer/src/components/CardView.astro` ](docs/architecture/files/apps/viewer/src/components/CardView.astro.md) — `src/components/CardView.astro`
  - [ `apps/viewer/src/lib/api.ts` ](docs/architecture/files/apps/viewer/src/lib/api.ts.md) — `src/lib/api.ts`
  - [ `apps/viewer/src/pages/index.astro` ](docs/architecture/files/apps/viewer/src/pages/index.astro.md) — `src/pages/index.astro`
  - [ `apps/viewer/src/styles/global.css` ](docs/architecture/files/apps/viewer/src/styles/global.css.md) — `src/styles/global.css`
