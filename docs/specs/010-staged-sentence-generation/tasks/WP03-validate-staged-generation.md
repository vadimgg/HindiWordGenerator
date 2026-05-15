---
id: WP03
title: Validate staged generation and update docs
agent_type: rust-engineer
status: planned
dependencies: ["WP01", "WP02"]
acceptance_refs: ["AC12", "AC13"]
extra_skills: ["doc-writer", "hindi-prompt-tuner"]
read_scope:
  - "src/**"
  - "docs/DESIGN.md"
  - "docs/ROADMAP.md"
  - "docs/specs/010-staged-sentence-generation/**"
  - "README.md"
write_scope:
  - "docs/DESIGN.md"
  - "docs/ROADMAP.md"
  - "docs/specs/010-staged-sentence-generation/**"
  - "README.md"
protected_scope:
  - "input/**"
  - "output/**"
  - "audio/**"
  - ".agents/rendered/**"
validation:
  - "cargo fmt --check"
  - "cargo test"
  - "make check"
manual_validation_reason: null
created_at: 2026-05-15T14:43:59.077166+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP03 - Validate staged generation and update docs

## Goal

Prove the staged generation implementation is coherent and update the active
docs so they describe staged generation as the normal path. This work package
owns final drift checks, end-to-end tests, doc cleanup, and a safe live-smoke
decision.

## Done When

- Active docs no longer describe normal generation as one full-enrichment
  prompt.
- Active docs mention stage-level run report metadata where generation behavior
  is described.
- Drift greps for old single-prompt generation wording are reviewed and any
  remaining matches are intentional.
- Full test suite passes.
- `make check` passes.
- A live Ollama smoke run is either completed safely or explicitly skipped with
  a reason in the handoff.
- If a live smoke run writes accepted output, the target path is confirmed safe
  before running.

## Must Not

- Do not edit `.agents/rendered/**`.
- Do not remove the full-enrichment eval prompt; it remains useful for
  comparison.
- Do not perform a live generation smoke run if the planned target would collide
  with real accepted output.
- Do not move or delete `input/`, `output/`, or `audio/`.

## Handoff Notes

Use the Hindi display rule in any new docs or CLI examples that include Hindi:
romanisation must appear directly under Devanagari. This package may touch docs
and tests, but it should not redesign the command surface.
