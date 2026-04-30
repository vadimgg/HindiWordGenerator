# Project Agents

These are lightweight custom agent briefs for the Hindi generator project.
They adapt the pack-style role design from `/Users/vadim/Projects/brief/docs/agents`
without requiring this repo to become a full agent-pack repository.

Each role file should define:

- role and ownership
- primary files or data it may change
- protected files it should not change without explicit scope expansion
- standards it must apply
- stop conditions

## Recommended Agents

- `pipeline-planner.md`
- `word-batch-generator.md`
- `sentence-batch-generator.md`
- `sentence-input-reviewer.md`
- `prompt-tuner.md`
- `schema-guardian.md`
- `audio-worker.md`
- `output-auditor.md`
- `language-teacher-reviewer.md`
- `astro-viewer.md`
- `reviewer.md`

## Source Priority

When instructions overlap, use this priority order:

1. User request in the current thread
2. Specific task or review packet
3. Project docs such as `AGENTS.md`, `README.md`, and `ARCHITECTURE.md`
4. Selected project agent role
5. Project standards in `agents/standards/`
6. Examples or previous generated output

The current task wins over reusable role behavior. Standards can guide execution,
but they do not override explicit scope, protected files, validation commands, or
stop conditions.

## How To Use Them

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

When delegating work, give the sub-agent:
- the role file name
- the concrete task
- the files it owns
- protected files or directories
- relevant standards
- validation commands
- the success condition

Example:

```text
Use agents/schema-guardian.md.
Task: tighten validation for words with weak sound_alikes.
Own these files: process.py, main.py.
Do not modify prompts.
Apply: agents/standards/coding.md and agents/standards/hindi-generator.md.
Validate with: uv run main.py check --type words --max-batches 1.
Done when: invalid outputs fail fast and check reports quality gaps clearly.
```

## Ownership Rules

To avoid overlap, assign agents by responsibility:

- Planner: `main.py`, planning logic, CLI ergonomics
- Sentence input reviewer: source sentence CSV quality, translation accuracy,
  romanisation, duplicates, and input structure before generation
- Word batch generator: small bounded word conversion runs from `input/words/`
  to `output/words/` using the current word prompt
- Sentence batch generator: small bounded sentence enrichment runs from
  `input/sentences/` to `output/sentences/` using the current sentence prompt
- Prompt tuner: generation prompt files
- Schema guardian: `process.py`, validation logic
- Audio worker: `audio_generator.py`, audio paths, batch enrichment
- Output auditor: review generated JSON and report issues, not implementation
- Language teacher reviewer: review output quality from a Delhi learner-teacher perspective
- Astro viewer engineer: `viewer/`, live card UI, browser interactions, viewer build
- Reviewer: close-gate review for code, prompt, data, and workflow changes

## Shared Standards

- `standards/hindi-generator.md`: project-specific workflow, data, prompt, and
  fix-routing rules.
- `standards/coding.md`: Python/CLI coding standards adapted from the Brief
  agent standards, including file/function size, CLI output, error handling,
  testing, comments, and reuse.
- `standards/astro-viewer.md`: Astro, TypeScript, CSS, browser interaction, and
  live generated-data viewing standards for `viewer/`.

## Suggested Workflow

1. Start with `pipeline-planner.md` to scope the task.
2. Use `sentence-input-reviewer.md` before sentence generation when source rows
   were produced by an AI or look risky.
3. Use `word-batch-generator.md` for small word conversion runs.
4. Use `sentence-batch-generator.md` for small sentence enrichment runs.
5. Hand schema changes to `schema-guardian.md`.
6. Hand prompt quality changes to `prompt-tuner.md`.
7. Hand audio work to `audio-worker.md`.
8. Use `output-auditor.md` to inspect a sample output set before large runs.
9. Use `language-teacher-reviewer.md` to decide whether prompt changes are needed for teaching quality.
10. Use `astro-viewer.md` for viewer UI, Astro, TypeScript, browser behavior, and
   local web validation.
11. Use `reviewer.md` before considering a task complete when behavior, prompts,
   generated data, or workflow docs changed.

## Stop Conditions

Agents should stop and ask for direction when:

- the required change falls outside the assigned write scope
- a protected file must change and no scope expansion was approved
- code and docs disagree in a way that changes behavior
- validation cannot be run or fails for reasons unrelated to the task
- a one-off data correction appears to reveal a repeated prompt or schema issue
