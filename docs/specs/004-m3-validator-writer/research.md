# Research

## Files

### `docs/DESIGN.md`

#### R001 - M3 Contract Is Infrastructure Before Generation

Status: confirmed  
Kind: scope  
Backlog: none  
Confidence: high

What we saw:
- M3 is "Validator And Writer"; M4 is "Direct Local Sentence Generation".
- Design says default generation writes accepted output only after validation.

Why it matters:
- M3 should not call Ollama or expose generation. It should give M4 a safe
  boundary to call.

Recommended action:
- Keep M3 focused on schema, validator, writer, and viewer compatibility.

#### R002 - Viewer Compatibility Is A Blocking M3 Requirement

Status: confirmed  
Kind: compatibility  
Backlog: none  
Confidence: high

What we saw:
- Roadmap says viewer compatibility for `word_id` must exist before real Rust
  output is accepted.

Why it matters:
- The first valid Rust-generated card would otherwise pass validation but fail
  in preview/export.

Recommended action:
- Include viewer `word_id` support in this spec rather than deferring it to M6.

### `src/sentence_plan.rs`

#### R003 - Source Fingerprint Logic Exists In Planner

Status: confirmed  
Kind: reuse  
Backlog: none  
Confidence: high

What we saw:
- M2 planner computes source fingerprints internally.

Why it matters:
- M3 validator must use the same fingerprint semantics to avoid source drift.

Recommended action:
- Extract shared source identity/fingerprint code or otherwise guarantee one
  tested implementation.

### `src/cli.rs`

#### R004 - No Generate Command Exists Yet

Status: confirmed  
Kind: scope  
Backlog: none  
Confidence: high

What we saw:
- Current CLI exposes `doctor`, `source ids`, and `sentences plan`.

Why it matters:
- M3 should not silently introduce `sentences generate`; M4 owns that command.

Recommended action:
- Keep help output unchanged except if small wording updates are necessary.

## Data Drift Themes Caught

- Planner and validator must not compute source fingerprints independently.
- Validator and viewer have different legacy responsibilities: validator rejects
  `word_index`, viewer tolerates it.

## Research Decisions

- M3 is infrastructure-only; M4 will be the first production path that writes
  accepted output.
- Viewer `word_id` support belongs in M3 because it blocks safe acceptance of
  real Rust output later.
