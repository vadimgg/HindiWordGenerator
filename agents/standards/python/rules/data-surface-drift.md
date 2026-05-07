# Data Surface Drift

For every changed data surface, identify the source of truth.

- `input/` is human/source material.
- `output/` is generated learner-facing card data and the source of truth for
  completed cards.
- `audio/` is generated media referenced by relative paths in output JSON.
- `manifest.json` is processing metadata, not the sole authority for completion.
- generation prompts own model behavior and expected card content.
- review prompts own QA reviewer behavior.
- `viewer/` reads generated output/audio and must not become card authority.

Flag competing authority drift, stale convenience data, hidden output rewrites,
and flows where a generated view can override a stronger source.
