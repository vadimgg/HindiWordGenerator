# Reuse Before Helpers

Before adding a parser, path helper, batch helper, schema checker, output
scanner, display helper, or audio helper:

- look for existing helpers in `process.py`
- check whether `main.py` already has display/planning helpers
- check whether `generate.py` already owns the orchestration concern
- check whether `audio_generator.py` already owns the audio concern
- check whether `viewer/src/utils/` owns the viewer-only concern

Only add a new abstraction when it reduces real duplication or clarifies an
ownership boundary.
