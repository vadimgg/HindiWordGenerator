# `crates/lingo-cli/src/composition.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Is the sole composition root: discovers the workspace, constructs concrete adapters, and wires command dependencies explicitly.

## Scope: this file owns

- adapter construction
- explicit audio catalog
- clock/environment process helpers
- app context

## Out of scope: this file must not own

- workflow policy
- hidden service locator
- runtime plugin scanning

## Allowed dependencies

- all adapter crates
- application ports

## Forbidden dependencies and shortcuts

- lower crates importing composition

## Key implementation shape

```rust
pub fn compose(env: ProcessEnvironment) -> Result<AppContext, StartupError> {
    let workspace = FsWorkspace::open_or_target(env.cwd.clone(), env.xdg_config_home)?;
    let prompts = HandlebarsPromptEngine::strict();
    let audio = AudioCatalogBuilder::new()
        .add_gtts(env.uv_path)
        .add_elevenlabs(env.http_client, env.elevenlabs_key)
        .build()?;
    let artifacts = ArtifactPublishers::new();
    Ok(AppContext { workspace, prompts, audio, artifacts, environment: env })
}
```

## Required tests / evidence

- built-in catalog is explicit
- missing optional ElevenLabs key does not hide gTTS
- no global mutable singleton

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
