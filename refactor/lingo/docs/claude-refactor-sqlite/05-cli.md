# 05 · CLI

## 1. The CLI is the product

The UI is a helper. **Every operation is doable from the CLI**, because an agent
(Codex / Claude Code) drives it, a human drives it, and it keeps the system
scriptable and testable. The CLI must stay clean and complete — no UI-only
operations. The UI displays the equivalent command (organized under `⌘`, doc 06).

## 2. Command surface (Producers → Library → Publishers)

```
lingo init   --lang <profile> [DIR]          create / repair a deck

# ── produce ───────────────────────────────────────────────
lingo extract  [RAW] [--collection T] [--section S]          raw text → draft sentences
                 (--print | --apply <reply> | --out <file> --watch | --api)
lingo enrich   [--filter …] [--limit N] [--force] [--reset]  drafts → translation+breakdown
                 (--print | --apply <reply> | --out <file> --watch | --api)
lingo import   --from <DIR>                                  merge a package (JSON/db) into the library

# ── library (organize) ────────────────────────────────────
lingo ls       [--section S] [--missing-audio] [--json]      list sentences
lingo show     <sentence-id>                                 one sentence (+ words)
lingo organize move <id> --to <ord> | --section S            reorder / re-section
lingo organize set  <id> --english … --section …            edit fields (marks human)
lingo organize tag  <id> --add t --remove t
lingo organize rm   <id>
lingo words    [--min-count N] [--json]                      lexicon view
lingo check    [--filter …]                                  deterministic validation

# ── audio (service) ───────────────────────────────────────
lingo audio    [--filter …] [--backend …] [--voice V] [--force]   (re)synthesize
lingo audio voices                                          list ElevenLabs voices

# ── publish ───────────────────────────────────────────────
lingo export   [--filter …] [--deck D] --dest <file.apkg>   → Anki
lingo package  [--filter …] [--as json|db] --dest <folder>  → JSON (default, like today) or db copy

# ── deck plumbing ─────────────────────────────────────────
lingo config   get [key] | set <key> <value>               read/write config.toml
lingo dump     --json [--out library.json]                 derived snapshot
lingo migrate                                              fold old files into library.db (one-time)
lingo status
lingo lang     list | show | which | edit
lingo doctor
lingo viewer   [--port 4321]
```

Renames from today (D7): `import`→`extract`, `build`→`enrich`; the package
importer is `import` (replacing the prototype `import-package`); `package [--as
json|db]`; `export` is Anki. `--filter` is a shared selector: `--section "Chapter
02"`, `--tag verbs`, `--id <id>`, `--missing-audio`, `--all` (default).

## 3. Model transport (one mechanism, three modes)

`extract` and `enrich` produce a prompt and consume a validated reply. The
transport is pluggable; all modes pass through the **same validation gate** —
nothing is written until the reply validates.

| Mode | How | Flags |
|---|---|---|
| Manual / web | print packet, paste reply back | `--print` then `--apply <reply.md>` |
| File handoff | write `prompt.md`, agent writes `reply.md`, CLI reads it | `--out runs/…/prompt.md --watch` |
| Direct API | CLI calls the model itself | `--api` |

File-handoff (Codex / Claude Code):

```
$ lingo extract raw/chapter-02.md --collection "Complete Hindi" --section "Chapter 02" \
      --out runs/extract/ch02/prompt.md --watch
prompt → runs/extract/ch02/prompt.md
watching runs/extract/ch02/reply.md …
# agent writes reply.md
✓ 14 sentences extracted → library (status: draft)
```

## 4. Bounded enrichment & "already processed" tracking (R13)

`enrich` is context-limited, so you choose the batch size and the CLI walks the
backlog without re-sending a sentence (doc 03 §4):

- default selection is `status = 'draft'`; `--limit N` caps one prompt;
- emitting a prompt claims those rows (`enriching` + run id), so the next call
  picks the following N — fan out in parallel safely;
- applying a reply flips that run's rows to `enriched`;
- `--reset` reclaims abandoned `enriching` rows; `--force` re-enriches finished
  rows (still preserving human fields).

```
$ lingo status
  sentences  238   (draft 238 · enriching 0 · enriched 0)

$ lingo enrich --limit 20 --out runs/enrich/a/prompt.md     # claims 20
$ lingo enrich --limit 20 --out runs/enrich/b/prompt.md     # claims the NEXT 20
$ lingo status
  sentences  238   (draft 198 · enriching 40 · enriched 0)

$ lingo enrich --apply runs/enrich/a/reply.md
✓ 20 enriched   (draft 198 · enriching 20 · enriched 20)
```

## 5. `--help` examples

```
$ lingo enrich --help
Enrich draft sentences: translation (if missing), romanisation, literal, and
word-by-word breakdown. Human-authored fields are preserved (never overwritten).

Usage: lingo enrich [OPTIONS]

Options:
      --filter <SEL>     which sentences (default: status=draft)
      --limit <N>        max sentences per prompt (context-window control)
      --force            re-enrich already-enriched rows (keeps human fields)
      --reset [<RUN>]    return abandoned 'enriching' rows to 'draft'
      --print            print the prompt packet to stdout
      --apply <FILE>     apply a pasted reply file
      --out <FILE>       write the prompt to FILE (file-handoff mode)
      --watch            after --out, watch for the sibling reply.md
      --api              call the configured model directly
  -h, --help
```

```
$ lingo package --help
Export a portable package: a filtered selection of the library + audio + manifest.
Default output is JSON (one file per sentence, like today); use --as db to emit a
SQLite copy instead. Consumed by Grasp and other tools.

Usage: lingo package [OPTIONS] --dest <FOLDER>

Options:
      --dest <FOLDER>    destination folder
      --as <FORMAT>      json (default) | db
      --filter <SEL>     which sentences (default: all)
  -h, --help
```

## 6. Output examples (clean, scriptable)

```
$ lingo status
Complete Hindi · hi · library.db
  sentences  238   (draft 0 · enriching 0 · enriched 238)
  words      612
  audio      238/238
  next       nothing pending — try `lingo package --dest …`
```

```
$ lingo ls --section "Chapter 02" --json | jq '.[0]'
{ "id": "01J8ZQ…", "section": "Chapter 02", "ord": 1,
  "target": "अध्यापक जी, यहाँ कितने विद्यार्थी हैं?",
  "english": "Teacher ji, how many students are here?",
  "status": "enriched", "audio": true }
```

```
$ lingo words --min-count 3
जी       honorific (ji)        used in 14 sentences
यहाँ      here                  used in  9 sentences
हूँ       am (to be, 1sg)       used in  7 sentences

$ lingo config set display.lead target
display.lead = target   (config.toml updated)
```

## 7. UI ↔ files parity

Every UI mutation has a CLI twin writing the **same** artifact (R7/R9):

| Operation | Artifact | CLI |
|---|---|---|
| reorder / re-section | `library.db` | `lingo organize move` |
| edit field | `library.db` (flips authority to human) | `lingo organize set` |
| change setting | `config.toml` | `lingo config set` |
| (re)generate audio | `audio/*.mp3` + `library.db` | `lingo audio` |
| export Anki | `*.apkg` | `lingo export` |
| export package | package folder | `lingo package` |

## 8. Exit codes & scripting

- `0` success, `1` operational error, `2` validation failure (e.g. a reply that
  failed the gate) — stable enough for agents to branch on.
- `--json` on read commands (`ls`, `show`, `words`, `status`).
- No interactive prompts in non-`viewer` commands (agents never hang).
