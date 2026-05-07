# Architecture Boundaries

- Keep `main.py` focused on argument parsing, command dispatch, and readable CLI
  output.
- Keep planning, dedupe, validation, paths, writes, and manifest updates in
  `process.py`.
- Keep LLM provider construction, prompt loading, retries, concurrency, and
  generation orchestration in `generate.py`.
- Keep audio synthesis and `audio` path enrichment in `audio_generator.py`.
- Keep viewer behavior in `viewer/`; the viewer reads generated data but does
  not become the source of truth.
- Future transcription code should get a clear owner module instead of being
  folded into generation or audio code by convenience.
- Do not add vague `utils.py`, `helpers.py`, or `common.py` files. Split by
  ownership or side-effect boundary when a split is needed.
