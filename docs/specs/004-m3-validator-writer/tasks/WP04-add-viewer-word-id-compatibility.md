---
id: WP04
title: Add viewer word id compatibility
agent_type: astro-viewer
status: done
dependencies: ["WP03"]
acceptance_refs: ["AC19", "AC20"]
extra_skills: []
read_scope: ["viewer/**", "output/sentences/*.json", "docs/DESIGN.md", "docs/ROADMAP.md", "docs/specs/004-m3-validator-writer/**"]
write_scope: ["viewer/**", "docs/ROADMAP.md", "docs/specs/004-m3-validator-writer/**"]
protected_scope: ["input/**", "output/**", "audio/**", "runs/**"]
validation: ["npm --prefix viewer run build", "cargo test", "git diff --name-only -- input output audio runs", "git diff --check"]
manual_validation_reason: null
created_at: 2026-05-15T03:01:05.805554+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP04 - Add Viewer Word ID Compatibility

## Goal

Update the Astro viewer/export path so new Rust sentence cards that use
`token.word_id` render correctly while legacy Python cards that use
`token.word_index` continue to render.

## Done When

- Viewer token-to-word lookup resolves `word_id` first.
- Viewer falls back to legacy `word_index` for existing output.
- A focused fixture/test or build-time check covers both shapes.
- Active roadmap wording no longer says viewer `word_id` support is pending.
- Validation commands in frontmatter pass, or `npm run build` failure is
  documented if viewer dependencies are unavailable.

## Must Not

- Modify accepted output JSON to add compatibility fields.
- Convert Rust `word_id` back to `word_index`.
- Change viewer UX beyond the compatibility path.
- Modify protected paths.

## Handoff Notes

This work is intentionally before M4 generation. The first Rust-generated card
must render without requiring a post-generation viewer patch.
