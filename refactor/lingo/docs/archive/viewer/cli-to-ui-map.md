# CLI → Studio UI mapping

Every flag and subcommand the generation pipeline exposes on the command line,
and the exact Studio control that must cover it. This is the contract: if a row
has no UI control, the Studio is incomplete. Source of truth is
`crates/lingo-cli/src/cli.rs`.

Legend: ✅ surfaced in Studio · 🖥 stays terminal-only (by design) · 🔢 default.

---

## `lingo import` — raw text → reviewed source YAML

| CLI | Type | Studio control |
| --- | --- | --- |
| `[RAW_FILE]` (positional) | path | ✅ Stage 0 **“Pick from raw/”** list; selecting a file targets it. Omitting = auto-pick next un-imported (the default list order). |
| `--batch <id>` | string | ✅ Stage 0 **Batch id** field (auto-filled from filename slug, editable). |
| `--title <text>` | string | ✅ Stage 0 **Title** field (auto-filled from humanised id). |
| `--subtitle <text>` | string | ✅ Stage 0 **Subtitle** field. |
| `--print` | flag | ✅ Packet Exchange **“Download packet” / “View raw”** (capture packet, do nothing else). |
| `--apply <reply-file>` | path | ✅ Packet Exchange **“Open in editor”** path and **Paste → Apply** (paste is the GUI-native equivalent of feeding a saved reply). |
| _(no flag — paste new raw text)_ | — | ✅ Stage 0 **Paste** / **Drop** modes write `raw/<batch>.md` then prepare import. (New convenience; CLI requires you to create the file first.) |

## `lingo build` — reviewed source → card JSON

| CLI | Type | Studio control |
| --- | --- | --- |
| `--batch <id>` | string | ✅ Selected via the Pipeline rail / batch selector; omitting = next batch needing a build (rail's **Build** quick-action). |
| `--print` | flag | ✅ Packet Exchange **“Download packet / View raw”**. |
| `--apply <reply-file>` | path | ✅ Packet Exchange **Paste → Apply** / **Open in editor**. |

## `lingo check` — deterministic validation

| CLI | Type | Studio control |
| --- | --- | --- |
| `--batch <id>` | string | ✅ Stage 3 runs against the selected batch; **Re-run check** button. |
| _(no batch)_ | — | ✅ Whole-deck check available from the rail’s **“Problems only”** view. |

## `lingo audio` — synthesize clips

| CLI | Type | Studio control |
| --- | --- | --- |
| `--batch <id>` | string | ✅ Stage 4 targets the selected batch. |
| `--backend gtts\|elevenlabs` | enum 🔢 gtts | ✅ Stage 4 **Backend** radio (gTTS labelled default). |
| `--voice <id>` | string | ✅ Voice picker → **“Use for this run only”** (one-shot id, no config write). |
| `--force` | flag | ✅ Stage 4 **“Force re-synthesize”** checkbox. |

### `lingo audio voices`

| CLI | Type | Studio control |
| --- | --- | --- |
| `--limit <n>` | usize 🔢 25 | ✅ Voice picker list (paginated/searchable); limit is implicit in the scroll list. |

### `lingo audio voice show`

| CLI | Studio control |
| --- | --- |
| `voice show` | ✅ Settings → **ElevenLabs voice** displays current id + name. |

### `lingo audio voice select`

| CLI | Type | Studio control |
| --- | --- | --- |
| `voice select` | — | ✅ Voice picker dropdown (replaces the fzf list). |
| `--limit <n>` | usize 🔢 50 | ✅ Implicit in the searchable list. |
| `--for-run` | flag | ✅ Picker’s **“Use for this run only”** toggle. |
| `--batch <id>` | string | ✅ Inherited from the selected batch. |
| `--force` | flag | ✅ Stage 4 **Force** checkbox. |

### `lingo audio voice set <id>`

| CLI | Studio control |
| --- | --- |
| `voice set <id>` | ✅ Voice picker **“Save to deck”** (writes `config.toml`). |

## `lingo status` — pipeline state

| CLI | Type | Studio control |
| --- | --- | --- |
| `status` | — | ✅ The **Pipeline rail** (always on screen) is its GUI. |
| `--batch <id>` | string | ✅ Click a batch row to focus it. |
| `--problems` | flag | ✅ Rail **“Problems only”** filter toggle. |
| _(Next hint)_ | — | ✅ Per-row **Resume / Build / Fix** quick-action. |

## `lingo viewer` — serve the app

| CLI | Type | Studio control |
| --- | --- | --- |
| `--port <n>` | u16 🔢 4321 | ✅ Settings → **Port** (applies to next launch; informational). |
| `--no-open` | flag | ✅ Settings → **“Open browser on start”** (inverse). |
| `--lead romanisation\|target` | enum | ✅ Settings → **Display lead** toggle (also `[display].lead`). |
| `--batch <id>` | string | ✅ Rail batch focus / filter. |

## `config.toml` (`[display]`, `[audio]`) — Settings drawer

| Config | Studio control |
| --- | --- |
| `[display].lead` | ✅ Display **Lead** toggle (Romanisation ↔ Target). |
| `[display].show_secondary` | ✅ **“Show secondary line”** switch. |
| `[audio].backend` / `fallback` | ✅ **Backend / fallback** selector + note. |
| `[audio.elevenlabs].voice` | ✅ **ElevenLabs voice** id + name. |
| `[audio.elevenlabs].model` | ✅ Read-only model field. |
| `[audio.elevenlabs].api_key = "env:…"` | ✅ **Key status** indicator only — never displays the key. |

## `lingo lang` — prompt/profile authoring

| CLI | Type | Studio control |
| --- | --- | --- |
| `lang which` | — | ✅ Packet Exchange **“Tune prompt”** shows the resolved layer (default/global/deck). |
| `lang edit import\|build` | — | ✅ **“Tune prompt”** → opens template in `$EDITOR` (server shells out). |
| `--global` / `--deck` | flag | ✅ Scope toggle in the Tune-prompt view. |
| `lang list` / `lang show <profile>` | — | 🖥 Terminal-only (profile introspection; out of scope for generation UI). |
| `--profile <id>` | string | 🖥 Terminal-only. |

## `lingo init` / `lingo doctor`

| CLI | Studio control |
| --- | --- |
| `init --lang <profile> [DIR]` | 🖥 Terminal-only — the Studio shows guidance when run outside a deck, but cannot scaffold one. |
| `doctor` | 🖥 Terminal-only — Studio surfaces the specific gaps it hits (no clipboard, no `$EDITOR`, no `ELEVENLABS_API_KEY`) contextually instead. |

## `lingo package` / `lingo export` — hand-off

These are *delivery*, not generation, but the Studio's **Done** panel links to
them (they already have a home in the Deliver tab).

| CLI | Type | Studio control |
| --- | --- | --- |
| `package --batch <id> --dest <dir>` | — | ✅ **Done → Package…** (deep-link to Deliver). |
| `export --batch <id>… / --all / --deck / --dest` | — | ✅ **Done → Export to Anki…** (deep-link to Deliver). |

---

### Coverage summary

Everything in the generation path — `import`, `build`, `check`, `audio` (incl.
all voice subcommands), `status`, the `[display]`/`[audio]` config, and prompt
tuning — has a Studio control. Only **deck creation** (`init`), **environment
diagnosis** (`doctor`), and **profile introspection** (`lang list/show`) remain
terminal-only, by design — none of them are part of turning raw text into cards.
