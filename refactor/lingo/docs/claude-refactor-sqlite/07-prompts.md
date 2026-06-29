# 07 · Prompts

## 1. A set per language, one per stage

| Prompt | Stage | Input → Output |
|---|---|---|
| `extract` | extract | raw text → sentences (foreign required; roman/English optional, preserved if supplied) |
| `enrich` | enrich | sentences → translation (if missing) + romanisation + literal + word-by-word breakdown |

Today these exist as `import.md.hbs` / `build.md.hbs`; they are renamed to
`extract` / `enrich` to match the commands (doc 05). Future optional prompts slot
into the same set without architectural change (e.g. `audio-hints`).

Per-language because romanisation conventions, script handling, and "what counts
as a word" differ. The Hindi `extract` prompt pins tilde nasalisation (`maĩ`,
`yahā̃`) and Delhi register.

## 2. The "respect the human" contract (critical, R4)

Both prompts are written around field authority (doc 03 §3):

- **extract** captures whatever the learner supplied and marks it `human`. It
  must not rewrite or "improve" provided translations/wording. Foreign-only input
  leaves English empty for enrich to fill.
- **enrich** receives each sentence *with its authority map* and is instructed:
  > Preserve every human-authored field exactly, including honorifics and
  > particles (e.g. Hindi *ji* → "uncle-ji"). Only generate empty fields. Never
  > restate or alter human text.

This is the whole reason for the two-stage split (doc 00 §2).

## 3. Where prompts live (resolution order)

Handlebars templates (`*.md.hbs`), most-specific-first:

```
1. Deck-local override     my-deck/prompts/<stage>.md.hbs
2. User global override     ~/.config/lingo/profiles/<id>/prompts/<stage>.md.hbs
3. Built-in profile         (embedded) crates/lingo-workspace-fs/assets/profiles/<id>/prompts/<stage>.md.hbs
```

Defaults ship in the binary; a user can override a language globally; a single
deck can override just for itself. This mechanism already exists and is kept.

## 4. Customizing

```
$ lingo lang edit hindi extract     # opens the effective template; writes a deck/global override on save
```

Or copy a built-in and edit by hand:

```
my-deck/prompts/extract.md.hbs   # overrides only this deck's extract prompt
my-deck/prompts/enrich.md.hbs
```

Overrides are plain files: agents can edit them and they diff cleanly in git
(unlike the binary db).

## 5. Template context

| Variable | Example | From |
|---|---|---|
| `{{target.language}}` | Hindi | profile |
| `{{target.script}}` | Devanagari | profile |
| `{{romanisation.convention}}` | iast-tilde | profile |
| `{{learner.goal}}` | practical fluency | config |
| `{{learner.native_languages}}` | ["English"] | config |
| `{{#each sentences}}` | (enrich) sentences + authority | library |

## 6. Example: `extract.md.hbs` (abridged)

```handlebars
You are preparing {{target.language}} sentence material for a learner whose
native language is {{#each learner.native_languages}}{{this}}{{/each}}.

Segment the raw material into complete, learnable sentences. Remove page chrome,
duplicates, page numbers, and fragments.

If the raw material ALREADY contains the learner's own translations or notes,
KEEP them verbatim — do not rewrite, "correct", or re-translate them, and
preserve honorifics/particles (e.g. {{target.language}} "ji": uncle-ji).
Mark any field the learner supplied as human-authored.

For each sentence output ONLY the extract fields:
  - target       (the {{target.script}} sentence; required)
  - romanisation ({{romanisation.convention}}; include only if supplied or trivial)
  - english      (include only if the learner supplied it)
  - tags         (lowercase, optional)

Do NOT add literal glosses, word breakdowns, audio, or source metadata — that is
the enrich stage.
```

## 7. Reply contracts (validated before commit)

`extract` reply (YAML):

```yaml
format: lingo.extract/v1
sentences:
  - target: "अध्यापक जी, यहाँ कितने विद्यार्थी हैं?"
    english: "Teacher ji, how many students are here?"   # human-supplied, preserved
    authority: { english: human }
    tags: [classroom]
  - target: "मैं लड़का हूँ।"                                # foreign-only; english omitted
    tags: [identity]
```

`enrich` reply (JSON):

```jsonc
{
  "format": "lingo.enrich/v1",
  "sentences": [
    {
      "id": "01J8ZQ…",
      "english": "Teacher ji, how many students are here?",  // unchanged (human)
      "romanisation": "adhyāpak jī, yahā̃ kitne vidyārthī haĩ?",
      "literal": "teacher ji here how-many students are",
      "register": "standard",
      "breakdown": [
        { "surface": "अध्यापक", "roman": "adhyāpak", "gloss": "teacher" },
        { "surface": "जी",      "roman": "jī",       "gloss": "honorific (ji)" }
      ]
    }
  ]
}
```

The validator checks: ids match the claimed run, `human` fields are byte-identical
to what was sent, required fields present, breakdown covers the sentence. On
failure the CLI returns exit code `2` and writes nothing.

### Partial batches are normal

`enrich` works on a learner-chosen slice per prompt (`--limit N`, doc 05 §4).
Each prompt contains only that run's sentences (each carrying its `id`); the reply
must return exactly those ids. The CLI tracks processed sentences via `status`
(doc 03 §4), so later prompts cover only the rest and nothing is enriched twice.

## 8. Adding a new language

1. `assets/profiles/<id>/profile.toml` (language, script, romanisation);
2. `assets/profiles/<id>/prompts/extract.md.hbs` and `enrich.md.hbs`;
3. `lingo init --lang <id>` works, and any deck can override the prompts locally.
