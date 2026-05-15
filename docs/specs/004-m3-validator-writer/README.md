# 004 - M3 Validator And Writer

## Goal

Add the Rust safety layer that validates candidate sentence batches and writes
accepted output atomically, before M4 introduces local-model generation.

## User-Visible Result

No new generation command yet. Existing planner commands continue to work, and
the viewer becomes ready to render future Rust output that uses `word_id`.

## Key Boundaries

- M3 validates and writes only through reusable internals/tests.
- M3 does not call Ollama.
- M3 does not expose `hindi sentences generate`.
- M3 does not write real `output/` during normal CLI use.
- M3 keeps legacy Python `word_index` support in the viewer only.

## Documents

- [spec.md](spec.md)
- [plan.md](plan.md)
- [architecture.md](architecture.md)
- [testing.md](testing.md)
- [cli.md](cli.md)
- [research.md](research.md)
- [tasks.md](tasks.md)
- [review.md](review.md)
