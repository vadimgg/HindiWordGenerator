# `crates/lingo-cli/tests/pipeline_e2e.rs`

> **Target kind:** Integration test  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Exercises the clean-slate pipeline with fake prompt/audio adapters and real filesystem/artifact adapters.

## Scope: this file owns

- init → import prepare/apply → build prepare/apply → check → audio → package/export flow

## Out of scope: this file must not own

- real ChatGPT/Claude calls
- real ElevenLabs/gTTS calls
- old-format fixtures

## Allowed dependencies

- test composition root
- temporary directories

## Forbidden dependencies and shortcuts

- network and user editor

## Key implementation shape

```rust
#[test]
fn complete_pipeline_produces_verified_package() {
    let app = TestApp::new();
    app.init("hindi");
    app.import_reply(valid_source_reply());
    app.build_reply(valid_card_reply());
    app.audio_with(FakeAudio::mp3());
    let package = app.package();
    package.verify_manifest_and_checksums();
}
```

## Required tests / evidence

- canonical v1 round-trips
- package manifest/checksums
- no compatibility path is exercised

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
