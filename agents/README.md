# Project Agent Packs

Local agent packs for HindiWordGenerator. The layout is inspired by the
reference agents in `reference/agents`, but this project does not depend on
Brief or any external agent-pack runtime.

Each pack lives at:

```text
agents/packs/<id>/
  AGENT.md    # role instructions
  pack.json   # metadata for routing and validation
```

Shared standards and skills live at:

```text
agents/standards/<area>/README.md
agents/standards/<area>/rules/*.md
agents/skills/<skill>/SKILL.md
```

## Source Priority

When instructions overlap, use this priority order:

1. User request in the current thread
2. Specific task or review packet
3. Project docs such as `AGENTS.md`, `README.md`, and `ARCHITECTURE.md`
4. Selected project agent pack
5. Project standards in `agents/standards/`
6. Examples or previous generated output

The current task wins over reusable role behavior. Standards guide execution,
but they do not override explicit scope, protected files, validation commands,
or stop conditions.

## Packs

| Pack | Use For |
|---|---|
| `python-engineer` | Python implementation, refactors, tests, CLI/runtime behavior. |
| `python-reviewer` | Python architecture, data drift, schema, CLI, and regression review. |
| `project-reviewer` | Whole-project close-gate review across code, prompts, data, docs, and workflow. |
| `project-manager` | Planning, sequencing, scope, handoff notes, and completion review. |
| `doc-writer` | Clear docs, architecture notes, workflow docs, and documentation review. |
| `cli-ux-reviewer` | Command names, help, output, errors, and next-step guidance. |
| `pipeline-planner` | `main.py`, run planning, batching behavior, and operator confidence. |
| `schema-guardian` | `process.py`, validation, output writes, batch numbering, and manifest safety. |
| `audio-worker` | `audio_generator.py`, MP3 generation, and relative `audio` path enrichment. |
| `prompt-tuner` | Generation and review prompts. |
| `word-batch-generator` | Small script-assisted word output generation. |
| `sentence-batch-generator` | Small script-assisted sentence output generation. |
| `sentence-input-reviewer` | Source sentence CSV quality before generation. |
| `output-auditor` | Generated JSON quality audit without direct implementation changes. |
| `language-teacher-reviewer` | Delhi Hindi teaching-quality review. |
| `astro-viewer` | Astro viewer UI, TypeScript, CSS, browser behavior, and local build checks. |

## Standards

| Standard | Use For |
|---|---|
| `standards/python/README.md` | Python, CLI, schema, parsing, file/function size, data drift, and testing. |
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

## Script-Assisted Generation Workflow

For generation without external API keys, batch agents should use the
script-assisted workflow:

1. `python3 process.py check --type <words|sentences> --batch-size <n>`
2. take one returned batch object
3. enrich exactly that object's `csv` with the relevant prompt
4. save raw JSON to a temporary file
5. `python3 process.py write <type> <stem> <batch_num> <total_batches> <count> <json_file>`

Use `uv run main.py run ...` only when the user explicitly wants the
API-backed runtime pipeline and credentials are configured.

When the user says current output can be replaced for testing, batch agents may
overwrite a small, explicit set of output batch files through `process.py write`.
They should not edit source CSVs under that permission alone, and they should
report exactly which output/audio files changed.

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
Use agents/packs/schema-guardian/AGENT.md.
Task: tighten validation for words with weak sound_alikes.
Own these files: process.py, main.py.
Do not modify prompts.
Apply: agents/standards/python/README.md and agents/standards/hindi-generator/README.md.
Validate with: uv run main.py check --type words --max-batches 1.
Done when: invalid outputs fail fast and check reports quality gaps clearly.
```

## Stop Conditions

Agents should stop and ask for direction when:

- the required change falls outside the assigned write scope
- a protected file must change and no scope expansion was approved
- code and docs disagree in a way that changes behavior
- validation cannot be run or fails for reasons unrelated to the task
- a one-off data correction appears to reveal a repeated prompt or schema issue
