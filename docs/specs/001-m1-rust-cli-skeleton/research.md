# Research

## Status

Used lightly. M1 is a greenfield Rust slice.

## Active Inputs Reviewed

- `docs/DESIGN.md`
- `docs/ROADMAP.md`
- `README.md`
- `docs/README.md`
- `.agents/rendered/packs/brief-manager/AGENT.md`
- `.agents/rendered/packs/brief-manager/skills/brief-workflow/SKILL.md`

## Findings

### R001 - Rust crate does not exist yet

Status: confirmed
Kind: implementation
Backlog: none

What we saw:
- No active `Cargo.toml` or `src/` exists at project root.

Why it matters:
- M1 must create the initial crate rather than modify an existing one.

Recommended action:
- Create a single binary crate at project root.

### R002 - Brief init is blocked by local GitHub auth

Status: confirmed
Kind: workflow
Backlog: none

What we saw:
- `brief init` failed because `gh` is installed but the local token is invalid.
- A minimal `.brief/config.json` was added manually so `brief spec new` could
  create this spec packet.

Why it matters:
- Local spec authoring works, but future `brief spec complete` / PR creation
  may still require fixing `gh auth`.

Recommended action:
- Fix GitHub auth before spec closeout or PR creation.

## Deferred

None.
