# Hindi Word Generator

This project is moving from the archived Python implementation toward a small
Rust-first local-model workflow for Hindi sentence flashcards.

For current implementation direction, start with:

- `docs/DESIGN.md`
- `docs/ROADMAP.md`
- `docs/ROMANISATION.md`

Older detailed planning drafts live under `archive/docs/rust-planning/` and are
reference material only. Do not treat them as active contracts.

## Active Priorities

1. Build the Rust CLI around the direct sentence path first.
2. Keep the first happy path small and explicit: `hindi doctor`,
   `hindi sentences plan`, `hindi sentences generate`,
   `hindi sentences audio`, `hindi viewer`.
3. Preserve append-only output safety.
4. Keep accepted output compatible with the viewer/export workflow.
5. Use local Ollama models, but do not build CLI-managed model switching in the
   first implementation.
6. Keep Python behavior available for parity checks during migration.

## Agent Packs

Active packs are selected in `.agents/config.toml`, sourced from
`common_agents/`, and rendered with `agents render --target codex`.

Use the rendered pack paths in the managed section below. Do not edit
`.agents/rendered/**` directly; update `common_agents/**` or
`.agents/config.toml`, then render again.

Project-specific specialists included for this repo:

- `astro-viewer` for the Astro preview/export app.
- `hindi-language-teacher-reviewer` for Hindi teaching quality, romanisation,
  register, and Delhi/practical naturalness.
- `hindi-prompt-tuner` for generation/evaluation prompt quality and prompt/schema
  alignment.

Archived Python-era packs are under `archive/agents/` and `archive/agents_1/`.

## Hindi Display Rule

Whenever docs, CLI output, reports, or review messages display Hindi text, also
display the romanisation directly under it. The user cannot read Devanagari
comfortably yet.

## Source Format

Source files use YAML:

```yaml
title: Complete Hindi
subtitle: Chapter 02
items:
  - id: "0001"
    hindi: क्या आप कमला जी हैं?
    romanisation: kyā āp Kamalā jī haĩ?
    english: Are you Kamala?
```

## Safety Rules

- Do not overwrite existing `output/` batch files during normal generation.
- Validate generated JSON before writing accepted output.
- Keep `output/` as the completed-card authority.
- Keep model/run metadata outside accepted card JSON.
- Keep source lineage in accepted sentence JSON once Rust generation starts.
- Do not move or delete `input/`, `output/`, or `audio/` without explicit user
  approval.

<!-- agents:codex:start -->
# Codex Agent Routing

This managed section is generated from `.agents/config.toml` and the configured agent source catalog.
Edit the selection config or catalog source, then run `agents render --target codex`.

## Agent Packs

- `agent-skill-reviewer`: `.agents/rendered/packs/agent-skill-reviewer/AGENT.md` (source `common_agents/packs/agent-skill-reviewer/agent.md`)
- `astro-viewer`: `.agents/rendered/packs/astro-viewer/AGENT.md` (source `common_agents/packs/astro-viewer/agent.md`)
- `brief-manager`: `.agents/rendered/packs/brief-manager/AGENT.md` (source `common_agents/packs/brief-manager/agent.md`)
- `cli-ux-reviewer`: `.agents/rendered/packs/cli-ux-reviewer/AGENT.md` (source `common_agents/packs/cli-ux-reviewer/agent.md`)
- `doc-writer`: `.agents/rendered/packs/doc-writer/AGENT.md` (source `common_agents/packs/doc-writer/agent.md`)
- `ios-designer`: `.agents/rendered/packs/ios-designer/AGENT.md` (source `common_agents/packs/ios-designer/agent.md`)
- `hindi-language-teacher-reviewer`: `.agents/rendered/packs/hindi-language-teacher-reviewer/AGENT.md` (source `common_agents/packs/hindi-language-teacher-reviewer/agent.md`)
- `plan-reviewer`: `.agents/rendered/packs/plan-reviewer/AGENT.md` (source `common_agents/packs/plan-reviewer/agent.md`)
- `hindi-prompt-tuner`: `.agents/rendered/packs/hindi-prompt-tuner/AGENT.md` (source `common_agents/packs/hindi-prompt-tuner/agent.md`)
- `project-manager`: `.agents/rendered/packs/project-manager/AGENT.md` (source `common_agents/packs/project-manager/agent.md`)
- `reader-experience-reviewer`: `.agents/rendered/packs/reader-experience-reviewer/AGENT.md` (source `common_agents/packs/reader-experience-reviewer/agent.md`)
- `rust-engineer`: `.agents/rendered/packs/rust-engineer/AGENT.md` (source `common_agents/packs/rust-engineer/agent.md`)
- `rust-reviewer`: `.agents/rendered/packs/rust-reviewer/AGENT.md` (source `common_agents/packs/rust-reviewer/agent.md`)
- `swift-engineer`: `.agents/rendered/packs/swift-engineer/AGENT.md` (source `common_agents/packs/swift-engineer/agent.md`)
- `swift-reviewer`: `.agents/rendered/packs/swift-reviewer/AGENT.md` (source `common_agents/packs/swift-reviewer/agent.md`)
- `usability-reviewer`: `.agents/rendered/packs/usability-reviewer/AGENT.md` (source `common_agents/packs/usability-reviewer/agent.md`)

## Context To Read

- none

## Protected Paths

- none

## Rules

- Read the matching pack `AGENT.md` before acting in that role.
- Treat the pack paths above as project-relative paths.
- Do not edit inside generated or protected paths unless the user explicitly expands scope.
- Use brief specs and task packets for implementation work when they are provided.
<!-- agents:codex:end -->
