# Project Agent Packs

Active local agent packs for the Rust migration of HindiWordGenerator.

Archived Python-specific packs and standards live under `archive/agents/`.

## Active Packs

| Pack | Use For |
|---|---|
| `rust-engineer` | Rust implementation, local-model CLI, migration parity, schema validation, and tests. |
| `rust-reviewer` | Rust architecture, CLI, data-safety, provider-boundary, and Python-parity review. |
| `doc-writer` | Clear docs, architecture notes, workflow docs, and documentation review. |
| `cli-ux-reviewer` | Command names, help, output, errors, and next-step guidance. |
| `plan-reviewer` | Skeptical implementation-gate review for plans, migration phases, acceptance criteria, and architecture risks. |
| `usability-reviewer` | Mental-model, first-read, CLI-output, and documentation usability review. |
| `project-manager` | Planning, sequencing, scope, handoff notes, and completion review. |
| `prompt-tuner` | Generation and review prompts. |
| `language-teacher-reviewer` | Delhi Hindi teaching-quality review. |
| `astro-viewer` | Astro viewer UI, TypeScript, CSS, browser behavior, and local build checks. |

## Active Standards

| Standard | Use For |
|---|---|
| `standards/rust/README.md` | Rust CLI, module boundaries, local-model provider seams, schema safety, and migration parity. |
| `standards/commenting/README.md` | Comment tags and comment quality for code and docs. |
| `standards/command-design/README.md` | Intent-level command design and user-facing CLI review. |
| `standards/hindi-generator/README.md` | Project workflow, prompt/schema ownership, output safety, and fix routing. |
| `standards/astro-viewer/README.md` | Viewer architecture, product UI, responsive design, and browser validation. |

## Skills

| Skill | Use For |
|---|---|
| `architecture-seam-planning` | Planning changes that touch ownership, persistence, commands, UI flows, or data surfaces. |
| `code-reuse-review` | Checking existing helpers and ownership before adding new abstractions. |
| `escaped-defect-handling` | Fixing bugs that escaped implementation, review, or manual QA. |
| `picture-first-docs` | Writing docs that start with mental models, concrete examples, and visible data flow. |

## Source Priority

When instructions overlap, use this priority order:

1. User request in the current thread
2. Specific task or review packet
3. Project docs such as `AGENTS.md`, `README.md`, and `docs/`
4. Selected project agent pack
5. Project standards in `agents/standards/`
6. Archived Python behavior, when explicitly used for parity

## Delegation Shape

When delegating work, give the sub-agent:

- pack id
- concrete task
- files it owns
- protected files or directories
- relevant standards and skills
- validation commands
- success condition

Example:

```text
Use agents/packs/rust-engineer/AGENT.md.
Task: implement `hindi sentences check` planning preview.
Own these files: future Rust CLI/planner files.
Do not modify output data.
Apply: agents/standards/rust/README.md and docs/RUST_LOCAL_MODEL_WORKFLOW.md.
Validate with: cargo test and a small parity comparison against archive/python.
Done when: the Rust preview reports pending/skipped sentence rows without writes.
```

## Parallel Review Panels

Use this review panel before implementing non-trivial docs, CLI, workflow, data
surface, or migration changes. Spawn each reviewer in fresh context so they do
not inherit the drafting conversation.

### Docs And Workflow Panel

Run these agents in parallel:

| Agent | Focus |
|---|---|
| `plan-reviewer` | Source of truth, architecture risks, data drift, write authority, sequencing, acceptance criteria, and testability. |
| `usability-reviewer` | First-read clarity, normal workflow, mental model, stale terms, examples, file ownership, and concept budget. |
| `cli-ux-reviewer` | Command grammar, flags, help/output, prompts, non-interactive behavior, exit codes, JSON output, and side-effect honesty. |

Shared input:

- `docs/CLI_COMMANDS.md`
- `docs/RUST_LOCAL_MODEL_WORKFLOW.md`
- `docs/RUST_LOCAL_MODEL_DESIGN.md`
- `docs/RUST_MIGRATION_PLAN.md`
- `docs/DATA_SURFACES.md`
- `docs/README.md`
- `README.md`
- relevant agent standards

Prompt shape:

```text
Review this docs/workflow package from fresh context.
Do not assume prior conversation.
Prioritize blockers, contradictions, source-of-truth drift, command UX,
unowned writes, and unclear recovery paths.
Return findings first, then suggested edits.
```

Use the panel result as an implementation gate: blockers should be fixed before
coding starts; needs-work items can become acceptance criteria for the first
work package.

## Stop Conditions

Agents should stop and ask for direction when:

- the required change falls outside the assigned write scope
- a protected file must change and no scope expansion was approved
- code and docs disagree in a way that changes behavior
- validation cannot be run or fails for reasons unrelated to the task
- generated learner data would become less safe
