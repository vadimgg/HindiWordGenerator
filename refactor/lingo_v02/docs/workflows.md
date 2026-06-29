# Lingo Workflows

How the tool is actually used, end to end. For the command reference see
[`CLI.md`](./CLI.md); for the on-disk layout see
[`package-and-agents.md`](./package-and-agents.md).

The pipeline is always the same progression:

```
raw material → draft sentences → enriched → QA checked → approved → audio → published
   extract       apply             enrich      qa           approve    audio    publish
```

Every model-facing stage is the same loop: **lingo writes a task → you/an agent
fills a reply → `lingo apply` commits it.** You never have to remember the next
step — each command prints it as a `Next:` line.

---

## Workflow A — by hand, with ChatGPT/Claude in the browser

This is the original workflow: you paste prompts into a chat model and save its
replies. Lingo never touches the network.

```bash
# 0. one-time: create the library
lingo init my-hindi-library
cd my-hindi-library

# 1. extract: turn a chapter into sentences
lingo extract raw/chapter01.md --deck ch01 \
  --title "Complete Hindi" --subtitle "Chapter 01"
#   → writes runs/ch01-extract-7f3a/task.md
#   → prints:  Next: lingo apply runs/ch01-extract-7f3a/

#   Paste task.md into ChatGPT/Claude. Save the reply as the reply file the
#   command named (runs/ch01-extract-7f3a/reply.yaml).

lingo apply runs/ch01-extract-7f3a/
#   → 12 draft sentences created
#   → Next: lingo enrich ch01

# 2. enrich: fill romanisation, gloss, word-by-word breakdown
lingo enrich ch01
#   → writes a task, prints Next: lingo apply runs/ch01-enrich-9b2c/
#   Paste, save reply.json, then:
lingo apply
#   (no argument needed when there's exactly one pending run)
#   → 12 enriched · Next: lingo qa ch01

# 3. qa (optional but recommended): catch model mistakes
lingo qa ch01
lingo apply                  # applies the corrections; shows a before/after diff
#   → Next: lingo ls --deck ch01 --unapproved

# 4. approve: curate what is ready for study
lingo ls --deck ch01 --unapproved
lingo approve sen-ch01-01
#   Repeat for rows you approve, or approve the whole deck when appropriate.
#   → Next: lingo audio ch01

# 5. audio: generate speech for approved sentences missing it
lingo audio ch01
#   → Next: lingo publish ch01 --format study --dest out/study

# 6. publish: export to your study target
lingo publish --format study --dest out/study     # whole library → iOS app
lingo publish ch01 --format anki --dest out/ch01.apkg
```

The golden rule by hand: **read the `Next:` line, run it.** When in doubt, run
`lingo status` — it always tells you the single most useful next command.

### Why keep the raw text around

If your source already contains *your own* translation or romanisation (common in
textbooks, or when you want "uncle-ji" instead of "uncle"), paste it in the raw
file as-is. `extract` preserves learner-supplied fields verbatim and marks them
`authority: human`; `enrich` and `qa` then **never overwrite them**. This is the
whole reason to feed raw text rather than just a word list.

---

## Workflow B — a coding agent drives the CLI (Codex / Claude Code)

Here a coding agent does the pasting for you. It does **not** call a model API on
Lingo's behalf — it reads the task file, produces the answer, writes the reply
file, and runs `apply`. The contract lives in `AGENTS.md` at the library root:

```
You drive Lingo by exchanging files. Never call a model API; Lingo doesn't either.

Loop:
  1. Run: lingo status --json
  2. If a run is pending, read runs/<run>/task.md, do the task exactly per the
     output contract inside it, write the reply file named in run.json, then
     run: lingo apply runs/<run>/
  3. Otherwise run the command in status.next, then go to 1.
  4. Stop when status.next reports "done".

Rules:
  - Obey the output contract inside each task.md exactly.
  - Never edit fields marked authority:human.
  - If apply reports a validation error, FIX THE REPLY FILE and re-apply.
    Do not start a new run.
  - Run `lingo qa <deck>` before publishing; apply its corrections.
  - Approve the enriched rows you want to study before audio/publish.
```

Why this is safe even when the agent's context gets large:

- Each `task.md` carries its own strict output contract; `apply` validates the
  reply against it and **rejects** malformed output with a precise, re-tryable
  error — so drift produces a clear failure, not silent corruption.
- `apply` refuses to overwrite `authority: human` fields and reports any the model
  tried to touch.
- `lingo qa` is the agent's self-check: it asks the model to review its own
  enrichment against a checklist before anything is published.

A typical agent session is just the same three steps repeating:

```bash
lingo status --json            # {"next": "lingo enrich ch01", "pending_runs": []}
lingo enrich ch01              # writes the task
# agent reads runs/.../task.md, writes runs/.../reply.json
lingo apply runs/ch01-enrich-9b2c/
# → status.next becomes "lingo qa ch01", and so on
```

### QA as a separate agent pass

Because agents degrade as context fills, run QA in a **fresh** agent (or a fresh
context) pointed only at `lingo qa <deck>`. It claims the enriched sentences,
writes a focused review task, and its reply is corrections keyed by sentence id.
`apply` patches only those fields. This isolates "review" from "produce" and keeps
each pass small.

---

## Workflow C — the web viewer (deferred)

`lingo viewer` will serve a local UI over the same library after the CLI-first
refactor has the application use cases in place. The future "Generate" buttons
must create the **same runs**; a textarea is just another place to paste a reply;
"Apply" calls the **same** use case. There is no second code path — the UI is a
third actor on the same loop, so anything done in the UI is visible to the CLI
and vice versa.

For now, use the CLI for the full extract→enrich→QA→approve→audio→publish
pipeline.

---

## Recovering when something goes wrong

| Situation | What to do |
|---|---|
| Don't know what to do next | `lingo status` |
| A reply failed validation | Fix the reply file, `lingo apply runs/<run>/` again (the run stays pending) |
| Sentences stuck in `enriching` (abandoned run) | `lingo enrich --reset` |
| Started an extract you don't want | `lingo runs clean --abandoned` (marks it abandoned; reports any empty deck it drops) |
| Don't remember which runs are open | `lingo runs ls` |
| Setup seems broken (missing audio backend, dirs) | `lingo doctor` |
| A whole deck is wrong | `lingo deck delete <slug>` |

The DB is the truth. You can delete `out/` and `audio/` and regenerate them;
`runs/` holds in-flight work, so only clean it once its replies are applied.

---

## The whole pipeline on one screen

```bash
lingo extract raw/ch01.md --deck ch01 --title "Complete Hindi" --subtitle "Chapter 01"
lingo apply
lingo enrich ch01
lingo apply
lingo qa ch01
lingo apply
lingo ls --deck ch01 --unapproved
lingo approve sen-ch01-01
lingo audio ch01
lingo publish --format study --dest out/study
lingo publish ch01 --format anki --dest out/ch01.apkg
```

Every line after the first was printed to you as the previous command's `Next:`.
