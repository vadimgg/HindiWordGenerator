---
id: picture-first-docs
display_name: Picture-First Docs
type: skill
version: 0.1.0
description: Use when writing or reviewing technical documentation that should be simple, complete, skimmable, architecture-aware, and rich with examples, file shapes, data shapes, diffs, and workflows.
activation:
  mode: selected_or_inferred
  examples:
    - writing a spec, recommendation, architecture note, or workflow guide
    - rewriting documentation so it is easier to skim
    - explaining folders, files, data stores, commands, reducers, projections, or domains
    - reviewing whether a document explains the full picture clearly
---

# Picture-First Docs

Use this skill when documentation should help a programmer quickly see the
system.

## Core Rule

Simple and complete.

Explain each concept in the simplest correct way. Use the simplest word,
shortest clear sentence, and smallest useful example first. If that is not
enough, add detail only until the idea is clear.

The goal is not short documentation. The goal is documentation that feels easy
to understand without leaving important gaps.

## Shape

Start broad, then zoom in.

Use the entry-point shape for overview documents. Use the picture-first shape
for technical documents that need system detail.

Use this order when it fits:

1. Short version
2. Mental model
3. Components involved
4. Files and data
5. Example workflow
6. Architecture
7. Edge cases
8. Deferred work

Skip sections that do not help. Add sections only when they clarify the picture.

## Document Roles

Give each document one clear job.

- `README.md`: orient the reader. Explain what is changing, why it matters, what
  changes in daily use, what stays the same, and where to read next.
- `spec.md`: define scope, goals, non-goals, and acceptance criteria.
- `architecture.md`: show ownership, data surfaces, authority boundaries,
  module boundaries, and drift risks.
- `testing.md`: map changes and risks to validation.
- `plan.md`: show implementation order, module responsibility, operation order,
  and risks.
- `tasks.md`: index work packages.
- `tasks/WP*.md`: give one agent an exact handoff.
- `review.md`: summarize closeout, validation, changed files, and follow-ups.

Do not make an overview file prove the whole design. Link to the deeper file.

## Entry-Point Docs

Use this for `README.md`, overview pages, and first-read documents.

Goal: help a reader who missed the conversation understand the situation
quickly.

Default shape:

1. What we're doing
2. Why now
3. What changes
4. What stays the same
5. Where to read more

Rules:

- Write like a teammate catching someone up after they missed the standup.
- Use 2-3 short paragraphs before any lists.
- Use before/after pairs for visible behavior changes.
- Avoid tables, diff blocks, acceptance criteria, and deep implementation
  detail.
- Link to detailed docs instead of repeating them.
- A reader should understand the change and why it matters without opening
  another file.

## What Each Section Should Do

- Short version: skim-friendly bullets for what changes, why it matters, and
  what is not changing.
- Mental model: the smallest useful phrase, diagram, or rule that explains the
  system.
- Components involved: files, folders, modules, commands, or documents, each
  with one simple sentence about what it owns.
- Files and data: folder trees, file snippets, JSON/YAML/Python shapes, or small
  diffs.
- Example workflow: user commands, what the system does, and what files change.
- Architecture: information flow and ownership boundaries.
- Edge cases: short list first, then details only where needed.
- Deferred work: what is intentionally not included, with links when useful.

## Writing Rules

- Prefer examples over long explanation.
- Prefer bullets over dense paragraphs.
- Prefer one concrete workflow over many abstract claims.
- Avoid redundant restatement. If a short model is already clear, do not add a
  second sentence that says the same thing.
- Use project words when they are already familiar, such as batch, stem,
  manifest, prompt, schema, output, audio, and viewer.
- Explain a project word simply the first time it matters.
- Do not repeat the same fact unless the second mention adds detail.
- Show authority boundaries: what is truth, what is a projection, and what is a
  human note.
- When explaining code behavior, show the command, the file change, and the data
  shape.
- When a file, event, or component appears often, create an index with links to
  the section that shows what it looks like.
- For event catalogs, include who or what triggers the event.
- Explain rules in bullets when that is easier to scan than prose.
- Keep representations consistent. If the doc says a field is structured, every
  example should use the structured shape.
- When a component owns behavior, say what it owns and what it does not own.
- Define operational categories. If the doc says "code task", explain how the
  system detects one.
- For new commands, include inputs, validation, writes, projection behavior,
  exit behavior, idempotency, and what the command must not change.
- If output can contain multiple items, define ordering.
- For edge cases, say whether the system rejects, warns, no-ops, or defers.
- Before finishing, scan examples for contradictions between human strings and
  structured fields, authority and projection, warnings and blockers, or current
  scope and deferred work.

## Good Patterns

Mental model:

```text
Input rows are source material.
Output JSON is completed card data.
Manifest data is processing metadata.
The viewer reads generated data.
```

Folder picture:

```text
input/sentences/
  complete_hindi_chapter_02_sentences.csv   # source rows
output/sentences/
  complete_hindi_chapter_02_sentences_batch_01.json
audio/sentences/
  complete_hindi_chapter_02_sentences_batch_01/
```

Data shape:

```json
{"title":"Complete Hindi","subtitle":"Chapter 02","sentences":[]}
```

Documentation-only comments can explain fields beside an example:

```text
$ cat output/sentences/complete_hindi_chapter_02_sentences_batch_01.json

{"title":"Complete Hindi","subtitle":"Chapter 02","sentences":[...]}
# title/subtitle: grouping labels copied from source headings
# sentences: learner-facing generated cards validated before write
```

Do not imply comments are valid in the real JSONL file. Use them only in docs.

Information flow:

```text
User command
  uv run main.py run --type sentences --max-batches 1
        |
        v
Pipeline plans pending batches
  - source CSV exists
  - output does not already contain the cards
  - limits are applied
        |
        v
Generation returns JSON
  - prompt is loaded
  - provider returns one batch
        |
        v
process.py validates and writes
  output/sentences/<stem>_batch_<nn>.json
```

Diff:

```diff
- "audio": "old/path.mp3"
+ "audio": "audio/sentences/stem_batch_01/01_sentence.mp3"
```

One-sentence explanation:

```text
`audio` is a relative browser/export path, not an absolute filesystem path.
```

Information flow:

```text
User action
  UI surface receives intent
        |
        v
Intent dispatched
  controller / view model handles intent
        |
        v
State updated
  owned state changes
        |
        v
Output refreshes
  view, CLI output, generated file, or cache updates
```

Edge case:

```text
If the commit does not exist:
- reject the command
- do not append an event
- leave status.json unchanged
- print a clear error
```

## Skim Test

Before finishing, skim only:

- headings
- diagrams
- code blocks
- bullet lists

If the core idea is not clear from those, improve the picture before adding more
prose.
