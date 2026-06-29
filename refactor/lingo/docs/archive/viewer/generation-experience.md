# Generation Studio — UI/UX specification

> A guided, in-browser flow for turning **raw text** into finished sentence
> cards, living inside the existing `lingo viewer`. It mirrors the CLI pipeline
> `raw → import → build → check → audio` exactly, surfaces every CLI option, and
> preserves Lingo's bring-your-own-model loop.

---

## 1. Principles

These are non-negotiable and explain most of the decisions below.

1. **The model loop stays visible and manual.** Lingo never calls a model. The
   Studio's job is to make *copy packet → paste into ChatGPT/Claude → paste
   reply back* effortless — not to pretend it isn't happening. Every generative
   stage is a **Packet Exchange** (§6): packet out, reply in, validate.
2. **Nothing becomes canonical until it validates.** Exactly like the CLI, a
   reply is parsed and checked locally before it can overwrite `input/` or
   `output/`. Invalid replies never destroy good data; failures reopen the same
   panel with problems listed inline.
3. **The CLI is the source of truth, the UI is a faithful front-end.** Every
   button maps to a documented CLI action (see
   [`cli-to-ui-map.md`](cli-to-ui-map.md)). Anything you can do in the terminal,
   you can do here; anything you do here, you could have typed.
4. **One straight pipeline, always legible.** The same five-stage rail
   (`Raw · Import · Build · Check · Audio`) is on screen at all times, with each
   batch's true state — mirroring `lingo status`.
5. **Local and trustworthy.** No network calls except to the local viewer
   server. Raw text and replies stay on the machine.

---

## 2. Where it lives

The viewer today is read-only (`Words · Sentences · Deliver · QA`). The Studio
adds **one new top-nav destination** and a couple of contextual entry points.

```
┌──────────────────────────────────────────────────────────────────────┐
│ ✦ LINGO     ◈ Words   ❝ Sentences   ✎ Studio   ⊙ Deliver   ◇ QA       │
└──────────────────────────────────────────────────────────────────────┘
```

- **Studio** (new tab, icon `✎`) — the generation workspace. Default landing
  when the deck has raw files that aren't fully built yet.
- **Entry points elsewhere:**
  - Empty Sentences/Words page → primary button **“Generate cards →”** jumps to
    Studio with the next pending batch focused.
  - Deliver/QA pages that find missing audio or failing checks → **“Fix in
    Studio”** deep-links to the offending batch at the right stage.

The Studio inherits the viewer's shell: 720px-centered column, sticky nav, dark
radial background, Barlow Condensed display type, amber→orange accent.

---

## 3. Information architecture

```
Studio
├── Pipeline rail            (always visible — the lingo status dashboard)
├── Batch list / selector    (left rail on wide screens, dropdown on mobile)
└── Stage panel              (the focused work area for the selected batch+stage)
    ├── Stage 0  New / Raw           paste · drop · pick a raw file
    ├── Stage 1  Import              Packet Exchange → reviewed sentences (YAML)
    ├── Stage 2  Build               Packet Exchange → cards (JSON)
    ├── Stage 3  Check               deterministic validation report
    ├── Stage 4  Audio               synthesize clips, pick backend/voice
    └── Done                         hand-off: view · package · export
```

The **Pipeline rail** is both a status display and the primary navigator: click
any stage chip to jump the stage panel there (when reachable).

---

## 4. The Pipeline rail (status dashboard)

A persistent header strip that is the GUI of `lingo status`. One row per batch.

```
WORKSPACE  hindi-practice            Hindi · 3 batches · 41 sentences      ⚙ Settings

  BATCH                 RAW   IMPORT   BUILD   CHECK   AUDIO
  introduce_yourself     ●      ●        ●       ●       ●     ✓ done
▸ chapter_02             ●      ●        ●       ●       ◐     12/17 audio   ▸ Resume
  chapter_03             ●      ●        ○       –       –     needs build   ▸ Build
  + New batch from raw text…
```

- **Dots** per stage: `●` done · `◐` partial · `○` pending/next · `–` not yet
  reachable. Colours follow the viewer palette: done = teal `#5eead4`, next/active
  = amber `#fbbf24`, blocked/pending = slate `#475569`.
- **Right column** echoes the CLI's human summary (`done`, `12/17 audio`,
  `needs build`) and a one-click **Resume / Build / Fix** that opens the correct
  stage. This is the GUI of the CLI's `Next` hint.
- **Filter toggle: “Problems only”** → GUI of `lingo status --problems`; collapses
  the list to batches with a failing check or missing audio.
- **`⚙ Settings`** opens the deck settings drawer (§8): display lead, audio
  backend/voice, port — the things that live in `config.toml`.
- Header line mirrors `lingo status`'s workspace summary (name, language, batch
  and sentence counts).

Clicking a batch row selects it; the stage panel below follows.

---

## 5. The stages

Each stage is a focused panel for the **selected batch**. A slim **stepper**
above the panel shows progress and lets you move between reachable stages:

```
   ① Raw ──● ② Import ──● ③ Build ──◐ ④ Check ──○ ⑤ Audio
                                         ▲ you are here
```

### Stage 0 — New / Raw  (`lingo import`'s source selection)

The entry to a new batch, or the source view of an existing one.

```
┌─ New batch ─────────────────────────────────────────────────────────┐
│  How do you want to add raw text?                                    │
│   ◉ Paste it      ○ Drop a file      ○ Pick from raw/                 │
│                                                                       │
│  ┌─ Raw text ──────────────────────────────────────────────────┐    │
│  │ I am a boy                                                    │    │
│  │ मैं लड़का हूँ                                                  │    │
│  │ maĩ laṛkā hū̃                                                  │    │
│  │                                                               │    │
│  │ I am a girl …                                                 │    │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  Batch id   [ introduce_yourself        ]  ⓘ auto from filename       │
│  Title      [ Introduce Yourself        ]  ⓘ optional                 │
│  Subtitle   [                           ]  ⓘ optional                 │
│                                                                       │
│  Saves to raw/introduce-yourself.md           [ Cancel ]  [ Save & Import → ]
└─────────────────────────────────────────────────────────────────────┘
```

- **Three input modes:**
  - **Paste** — a large textarea; on save, the viewer server writes it to
    `raw/<batch>.md`. This is the friendly path that has no CLI equivalent
    (today you create the file by hand), and the headline feature for “generate
    from raw input.”
  - **Drop a file** — drag-and-drop / file picker; copies into `raw/`.
  - **Pick from raw/** — lists existing un-imported raw files (GUI of `import`'s
    “next un-imported file” auto-pick); selecting one is the equivalent of
    `lingo import raw/<file>`.
- **Batch / Title / Subtitle** map to `--batch`, `--title`, `--subtitle`. Batch
  id auto-fills from the filename (slugified, matching the CLI's `slug()`), with
  an inline “auto” hint; editing it is allowed. Title auto-fills from a
  humanised id (matching the CLI's `human_title()`).
- **“Save & Import →”** writes the raw file and advances to Stage 1 with the
  packet already prepared.

> **Small-batch nudge:** an inline tip ("smaller batches are easier to review
> and cheaper to redo") shows when the pasted text exceeds ~25 lines, echoing
> the USER_GUIDE advice.

### Stage 1 — Import  (`lingo import` → reviewed source YAML)

A **Packet Exchange** (§6). Packet format: the import prompt; reply format:
YAML. On a valid reply the server writes `input/sentences/<batch>.yaml` and the
panel flips to a **review** of the reviewed sentences:

```
┌─ Imported · 8 sentences ────────────────────────────────────────────┐
│  मैं लड़का हूँ            maĩ laṛkā hū̃         I am a boy.            │
│  मैं लड़की हूँ            maĩ laṛkī hū̃         I am a girl.           │
│  …                                                                   │
│                                                  [ Re-import ]  [ Build → ]
└─────────────────────────────────────────────────────────────────────┘
```

- Each row shows target / romanisation / English / tags — the reviewed source
  that became canonical.
- **Re-import** re-opens the Packet Exchange (re-running `import` over the same
  raw file; stable IDs are preserved exactly as the CLI's fingerprint logic
  does — unchanged sentences keep their IDs).
- **Build →** advances to Stage 2.

### Stage 2 — Build  (`lingo build` → enriched card JSON)

A **Packet Exchange**. Packet: the build prompt seeded with the reviewed
sentences; reply format: JSON. On a valid reply (after passing
`check_card_batch`) the server writes `output/sentences/<batch>.json`.

The review surface here is richer — it reuses the existing **SentenceCard**
component so a built card looks exactly as it will in the Sentences tab:
sentence, romanisation, English + literal gloss, register, and the expandable
word-by-word breakdown.

```
┌─ Built · 8 cards ───────────────────────────────────────────────────┐
│  ❝ मैं लड़का हूँ                                            ▶ play     │
│    maĩ laṛkā hū̃ · I am a boy.  · lit. "I boy am"  · register: neutral│
│    ▾ words: मैं (I, pron) · लड़का (boy, noun·m) · हूँ (am, verb)        │
│  …                                                                   │
│                                                  [ Re-build ]  [ Check → ]
└─────────────────────────────────────────────────────────────────────┘
```

### Stage 3 — Check  (`lingo check`)

No model. A button runs deterministic validation for the batch and renders the
report. This is the GUI of `lingo check --batch <batch>`.

```
┌─ Check · chapter_02 ────────────────────────────────────────────────┐
│  ✓ 17 cards structurally valid                                       │
│  ✓ romanisation reconstructs every sentence                          │
│  ⚠ 5 cards missing audio          → resolved by Stage 5              │
│                                                       [ Re-run check ]│
└─────────────────────────────────────────────────────────────────────┘
```

- Errors (red) block progress and link to the offending card; warnings (amber)
  — notably “missing audio” before audio is generated — are expected and do not
  block. Mirrors the CLI's warning-vs-error distinction precisely.
- A clean check enables **Audio →**.

### Stage 4 — Audio  (`lingo audio`)

Synthesize the missing clips. Backend and voice controls are inline (and also in
deck Settings, §8):

```
┌─ Audio · chapter_02 ────────────────────────────────────────────────┐
│  Backend    ◉ gTTS (default)   ○ ElevenLabs                          │
│  Voice      [ Rachel · 9BWtsMIN… ▾ ]   ⓘ ElevenLabs only             │
│  ☐ Force re-synthesize clips that already exist                      │
│                                                                       │
│  5 of 17 clips missing                              [ Generate audio ]│
│  ▓▓▓▓▓▓▓▓░░░░░░░  synthesizing 4/5 …                                  │
└─────────────────────────────────────────────────────────────────────┘
```

- **Backend** = `--backend gtts|elevenlabs`. gTTS is labelled default and needs
  nothing; selecting ElevenLabs reveals the voice picker and a key-status line.
- **Voice picker** = the GUI of `audio voice select` / `audio voices` /
  `audio voice set`: a searchable dropdown listing voices for the configured
  `ELEVENLABS_API_KEY`. “Use for this run only” = `--for-run` / the
  `--voice <id>` one-shot; “Save to deck” = `audio voice set`.
- **Force** = `--force`.
- **Generate audio** streams progress (one tick per clip). On completion it
  re-runs check automatically so the batch goes green, then enables **Done**.
- If no `ELEVENLABS_API_KEY` is present and ElevenLabs is chosen, an inline
  banner explains it falls back to gTTS (mirroring the config `fallback` chain),
  with a link to Settings.

### Done — hand-off

When a batch is fully built, checked, and voiced:

```
┌─ chapter_02 is ready ───────────────────────────────────────────────┐
│  17 cards · 17 clips · checks green                                  │
│  [ View in Sentences ]   [ Package… ]   [ Export to Anki… ]          │
└─────────────────────────────────────────────────────────────────────┘
```

These deep-link into the existing Sentences tab / Deliver tab (`lingo package`,
`lingo export`). Generation hands off to the read/deliver experience that
already exists.

---

## 6. The Packet Exchange (the heart of the UI)

`import` and `build` share one component. This is where the model loop lives. It
is deliberately a **two-pane, three-step** affair so the manual hand-off never
feels hidden or broken.

```
┌─ Import · introduce_yourself ───────────────────────────────────────┐
│  ① Copy the packet      ② Paste into ChatGPT / Claude    ③ Paste reply
│                                                                       │
│  ┌─ PACKET (read-only) ───────────┐  ┌─ REPLY (paste here) ────────┐ │
│  │ You are helping build a Hindi  │  │                             │ │
│  │ study deck. For each source    │  │  ⌁ paste the model's YAML   │ │
│  │ block, return YAML with …      │  │     reply, then Apply       │ │
│  │ …                              │  │                             │ │
│  │ ── source ──                   │  │                             │ │
│  │ I am a boy / मैं लड़का हूँ …     │  │                             │ │
│  └────────────────────────────────┘  └─────────────────────────────┘ │
│  [ ⧉ Copy packet ]  ✓ copied         [ Open in editor ]  [ Apply → ] │
│                                                                       │
│  ⓘ Reply must be YAML matching the requested format. Lingo validates │
│    locally before anything is saved.                                 │
└─────────────────────────────────────────────────────────────────────┘
```

**Step ① — Copy.** The packet is rendered into the left (read-only) pane and
**auto-copied to the clipboard on open** (same behaviour as the CLI). A
prominent **⧉ Copy packet** button re-copies; it flips to “✓ copied”. A small
**“Download packet”** / **“View raw”** affordance covers `--print` (capture the
packet for a prompt experiment or to drive the model by hand).

**Step ② — Generate (off-app).** A short instruction and, optionally, quick
links to open chatgpt.com / claude.ai in a new tab. Lingo does not send
anything — the user pastes the packet themselves.

**Step ③ — Apply.** The right pane is the reply target. Two ways in, matching
the CLI:
- **Paste** the reply into the textarea → **Apply →**.
- **Open in editor** = the classic `$EDITOR` reply file loop, for people who
  prefer it; equivalent to `--apply <reply-file>` after editing.

On **Apply**, the reply is sent to the server, parsed, and validated. Outcomes:

| Outcome | UI |
| --- | --- |
| Valid | Panel advances to the stage's review surface; rail dot turns done. |
| Invalid (parse / schema / unknown source id / missing item) | The reply pane stays, gains a red **problems list** with line refs, and an explanatory header. Nothing was saved. Fix and re-Apply. |
| Empty reply | Inert (matches CLI's "no reply applied; canonical data unchanged"); a quiet note, no destruction. |

The validation messages are the *same* ones the application layer already
produces (`import.rs` / `build.rs`): "reply contains no source items",
"unknown source item …", "reply omitted source item …", "duplicate card for
source item …", "profile requires romanisation for every item", plus YAML/JSON
parse errors. They render verbatim so the GUI and CLI teach the same fixes.

**Format affordances** — the panel header shows the exact expected `format`
value the packet requested (YAML vs JSON), echoing the troubleshooting advice,
and strips stray markdown fences (```` ```yaml ````) before validating, with a
gentle note when it does.

---

## 7. Editing prompts (`lingo lang edit`)

A secondary **“Tune prompt”** link in each Packet Exchange header opens a
read view of the active prompt template and shows which layer it resolves from
(default → global → deck) — the GUI of `lingo lang which`. Editing itself opens
the template in `$EDITOR` (server shells out, like `lingo lang edit import`),
with a toggle for **global** vs **--deck** scope. This is intentionally a thin
wrapper: prompt authoring stays a text-file activity, the Studio just routes to
it.

---

## 8. Deck Settings drawer (`config.toml`)

Opened from the rail's `⚙ Settings`. Groups the per-deck config the pipeline
reads, each writing back to `config.toml`:

- **Display**
  - **Lead**: `Romanisation` ↔ `Target` toggle (`[display].lead`; also the
    viewer's `--lead`). Live-previews against a sample card.
  - **Show secondary line**: switch (`[display].show_secondary`).
- **Audio**
  - **Backend / fallback**: gTTS ↔ ElevenLabs, with fallback note.
  - **ElevenLabs voice**: id + display name (`audio voice show` / `set`),
    model, and a key-status indicator that reads whether `ELEVENLABS_API_KEY`
    is set (never shows the key; `api_key = "env:…"`).
- **Viewer**
  - **Port** (`--port`), **Open browser on start** (`--no-open` inverse) — these
    apply to the next `lingo viewer` launch and are informational here.

Settings changes are explicit (**Save**), never silent, and a toast confirms the
`config.toml` write.

---

## 9. Backend: endpoints the viewer server must add

Today `viewer_server.rs` is **GET-only** (`/api/session`, static, `/audio/…`).
The Studio needs a small, local, write-capable API. All endpoints are
`127.0.0.1`-only and call straight into the existing `lingo-application`
functions — no new business logic, no model calls.

| Method & path | Backs | Calls |
| --- | --- | --- |
| `GET  /api/studio/status` | Pipeline rail | `status` report (per-batch stage state) |
| `GET  /api/studio/raw` | Stage 0 “Pick from raw/” | `WorkspaceStore::list_raw` |
| `POST /api/studio/raw` | Stage 0 paste/drop | write `raw/<batch>.md` |
| `POST /api/studio/import/prepare` | Import packet | `prepare_import` (returns packet + run_id) |
| `POST /api/studio/import/apply` | Import Apply | `apply_import` (validate + write `input/`) |
| `POST /api/studio/build/prepare` | Build packet | `prepare_build` |
| `POST /api/studio/build/apply` | Build Apply | `apply_build` (validate + write `output/`) |
| `POST /api/studio/check` | Check stage | `check` report |
| `GET  /api/studio/audio/voices` | Voice picker | ElevenLabs voices (key-gated) |
| `POST /api/studio/audio` | Generate audio | `audio` synth (streams progress, SSE/chunked) |
| `GET/POST /api/studio/config` | Settings drawer | read/write `config.toml` |

Notes:
- Reuse the exact `Prepare*`/`Apply*` request/response types from
  `lingo-application` so validation parity is automatic. The HTTP layer is a
  thin adapter, same as the CLI commands.
- `apply` endpoints return the structured validation report on failure (HTTP
  422) so the Packet Exchange can render the inline problems list — they must
  **not** write canonical data on failure (the application layer already
  guarantees this).
- Audio progress wants streaming; a chunked/SSE response per clip keeps the
  progress bar honest. A non-streaming fallback (poll `status`) is acceptable
  for v1.
- Keep the GET-only safety posture for anything outside `/api/studio/*`; the new
  write routes must validate batch ids and never accept absolute/`..` paths
  (reuse `is_safe_relative_path`).

---

## 10. States, empties, and errors

- **No deck / not a workspace** → friendly empty state with **“Run `lingo init
  --lang <language>`”** guidance (the Studio can't create a deck itself; init is
  a terminal step).
- **No raw files** → Stage 0 opens directly in **Paste** mode with focus in the
  textarea: the fastest path from blank deck to first packet.
- **Clipboard unavailable** → the Copy button shows “Select all & copy manually”
  and the packet pane auto-selects on click (mirrors the CLI's "clipboard
  unavailable" note).
- **`$EDITOR` unavailable** → “Open in editor” is disabled with a tooltip; paste
  flow is unaffected.
- **Server/network error on Apply** → reply is preserved in the textarea, a
  retry banner appears; canonical data is never touched.
- **Profile changed mid-run** → the same guard the CLI has (`ProfileChanged`)
  surfaces as “This deck's language changed since the packet was generated;
  re-prepare the packet.”

Every panel has explicit **loading** (skeleton + “preparing packet…”), **busy**
(disabled actions during Apply/synthesis), and **success** (toast + rail dot
update) states.

---

## 11. Keyboard & accessibility

- `⌘/Ctrl + Enter` = Apply in any Packet Exchange.
- `⌘/Ctrl + C` while the packet pane is focused re-copies the packet.
- Stepper and rail are real buttons with `aria-current`; stage transitions move
  focus to the new panel heading.
- Progress and validation results use `aria-live="polite"` so screen readers
  announce “8 cards built” / “3 problems found.”
- Devanagari (and other scripts) render with the viewer's existing
  `Tiro Devanagari Hindi` stack; reply textarea is `dir="auto"`.
- All state is conveyed by text/labels, not colour alone (dots carry glyphs
  `● ◐ ○ –`, not just hue).

---

## 12. Visual language

Reuse the viewer's tokens (`apps/viewer/src/styles/partials/*`) verbatim:

| Token | Value | Use in Studio |
| --- | --- | --- |
| Background | `radial-gradient(#0d1829 → #020617)` | page |
| Panel | `linear-gradient(#131f35, #0f172a)`, border `rgba(51,65,85,.5)`, radius 12px | stage panels, Packet Exchange |
| Accent | amber `#fbbf24` → orange `#f97316` | active stage, primary buttons, focus rings |
| Done | teal `#5eead4` | completed stage dots |
| Pending/muted | slate `#475569` | not-yet-reachable, hints |
| Error | red `#f87171` | validation problems |
| Warning | amber `#f59e0b` | non-blocking check warnings |
| Display type | Barlow Condensed, uppercase, tracked | stage titles, rail labels |
| Body type | Plus Jakarta Sans | prose, hints |
| Script | Tiro Devanagari Hindi | target text |

Primary buttons = amber gradient fill; secondary = slate outline; the Apply and
Copy buttons get the amber treatment because they are the two actions that move
the loop forward.

---

## 13. Out of scope (deliberately)

- **Automatic model calls / API keys for generation.** Forbidden by design.
- **Editing canonical YAML/JSON in the browser.** Hand-editing `input/`/`output/`
  stays a text-editor activity (the USER_GUIDE supports it); the Studio re-runs
  `check` if you do.
- **Creating a deck (`lingo init`).** A one-time terminal step.
- **Word batches.** This flow is sentence-first (the refactor is sentence-led);
  the legacy Words tab remains read-only.

---

## 14. Build order (suggested)

1. Read-only **Pipeline rail** wired to a new `GET /api/studio/status` — ship the
   dashboard first; it's useful immediately and de-risks the status mapping.
2. **Packet Exchange** component + `import` prepare/apply endpoints → the core
   loop end-to-end for one stage.
3. Generalise Packet Exchange to **build**; add the SentenceCard review reuse.
4. **Check** + **Audio** stages (audio last; it's the only one needing
   streaming).
5. **Settings** drawer + **Stage 0** raw-paste convenience.
6. Deep-links from Sentences/Deliver/QA.

See [`cli-to-ui-map.md`](cli-to-ui-map.md) for the flag-by-flag contract each
control must honour, and [`demo/index.html`](demo/index.html) for the agreed
look and flow.
