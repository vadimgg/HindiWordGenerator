# Viewer — Generation Studio docs

This folder specifies a new capability for the local viewer (`apps/viewer`,
served by `lingo viewer`): a **user-friendly, in-browser way to generate
sentence cards from raw input**, replacing the terminal `import → build → check
→ audio` loop with a guided UI — without losing any power the CLI gives you.

## The one constraint that shapes everything

Lingo **never calls a language model on its own**. There is no API key for
generation, no model server. The two generative steps — `import` and `build` —
are a manual three-beat loop around *your* ChatGPT/Claude window:

```
EMIT  →  GENERATE  →  APPLY
(copy packet)  (paste into your model, copy reply)  (paste reply back, validate)
```

The Studio does **not** remove that loop — it makes it pleasant. It copies the
packet for you, gives you a real paste target with live validation, and shows
inline what failed. Your accepted cards still only become "real" after passing
deterministic validation locally. See
[`generation-experience.md`](generation-experience.md) for the rationale baked
into every screen.

## What's here

| File | What it is |
| --- | --- |
| [`generation-experience.md`](generation-experience.md) | The full UI/UX spec: navigation, every stage, every state, validation, settings, and the backend endpoints the viewer server must add. |
| [`cli-to-ui-map.md`](cli-to-ui-map.md) | Exhaustive table mapping **every** `import` / `build` / `check` / `audio` / `status` / `viewer` CLI flag to its Studio UI control. Nothing the CLI can do is dropped. |
| [`demo/index.html`](demo/index.html) | A self-contained, clickable mock of the Studio. Open it in any browser — no build, no server. It walks the full Raw → Import → Build → Check → Audio flow with fake data. |

## Status: implemented (phase 2 done)

The Studio is now wired to real data, not the mock:

- **UI** lives in `apps/viewer` — `components/tabs/StudioTab.astro`,
  `scripts/ui/studio.js`, `scripts/studio/api.js`, `styles/partials/studio.css`.
- **Backend** lives in the Rust viewer server —
  `crates/lingo-cli/src/studio.rs` (orchestration over `lingo-application`) and
  `crates/lingo-cli/src/viewer_server.rs` (routing). Every Studio action behaves
  like one CLI invocation: it discovers a fresh workspace `Composition` and calls
  the same use cases (`prepare_import`/`apply_import`, `prepare_build`/
  `apply_build`, `check`, `synthesize_audio`, `status`) the CLI uses. No model is
  ever called — the manual packet loop is preserved.

Implemented routes (served at `127.0.0.1` only by `lingo viewer`):

```
GET  /api/studio/status            POST /api/studio/raw
GET  /api/studio/batch?batch=ID    POST /api/studio/import/prepare
GET  /api/studio/raw               POST /api/studio/import/apply
GET  /api/studio/voices            POST /api/studio/build/prepare
GET  /api/studio/config            POST /api/studio/build/apply
                                   POST /api/studio/check
                                   POST /api/studio/audio
                                   POST /api/studio/config
```

Run it for real:

```bash
cd <your-deck> && lingo viewer        # builds + serves the viewer, opens the browser
```

then click the **✎ Studio** tab.

## Try the mock

```bash
open docs/viewer/demo/index.html        # macOS
# or just double-click it in a file browser
```

The mock is a **fidelity reference**, not production code — it fakes the model
reply and the server. Use it to feel the flow and to agree on layout/wording
before building the real Astro components.
