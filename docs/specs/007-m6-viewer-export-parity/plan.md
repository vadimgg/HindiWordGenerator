# Plan

## Design

Add the smallest Rust surface around the existing viewer/export workflow:
`hindi viewer` launches the Astro viewer from `viewer/`, while `hindi export`
builds a deterministic sentence Anki import artifact from accepted JSON. Keep
viewer internals in the Astro app and keep Rust export focused on a rebuildable
file, not live AnkiConnect.

## Modules

| Module | Responsibility |
|---|---|
| `src/cli.rs` | Parse `viewer`, `viewer --help`, `export --help`, and `export --source ... --topic ...`. |
| `src/main.rs` | Dispatch viewer/export commands and map exit behavior. |
| `src/viewer.rs` | Build/run the viewer command, print URL, handle missing viewer dependencies clearly. |
| `src/export.rs` | Load accepted sentence output, filter by source/topic, build Anki import artifact. |
| `src/sentence_schema.rs` | Reuse sentence batch/card structures. |
| `viewer/` | Existing Astro app and check scripts; avoid redesign. |

## Operation Order

### `hindi viewer`

1. Discover project root.
2. Verify `viewer/package.json` exists.
3. Print the viewer URL (`http://localhost:4321`) and command.
4. Run `npm run dev` with current directory set to `viewer/`.
5. Forward the child process exit code.

### `hindi export --source ... --topic ...`

1. Discover project root.
2. Read accepted sentence batches from `output/sentences/*.json`.
3. Filter batches where `title == --source` and `subtitle == --topic`.
4. Flatten matching sentences and attach group/topic metadata.
5. Convert each sentence to Anki import fields.
6. Write a deterministic tab-separated artifact under `exports/`.
7. Print artifact path, exported count, and missing-audio count.

## Work Package Sequence

| WP | Purpose |
|---|---|
| WP01 | Review viewer/export contracts before implementation. |
| WP02 | Implement the `hindi viewer` command. |
| WP03 | Build sentence export selection and field mapping. |
| WP04 | Write the CLI export artifact under `exports/`. |
| WP05 | Add controlled end-to-end and viewer/export smoke checks. |
| WP06 | Review M6 parity and safety before PR. |

## Risks

| Risk | Mitigation |
|---|---|
| Rust export drifts from viewer export fields. | Mirror the current sentence field names and add fixture tests. |
| `hindi viewer` becomes a process manager. | Run the existing `npm run dev` directly; do not supervise/restart. |
| Browser opening causes platform noise. | Treat URL printing as required; browser opening can be deferred if brittle. |
| Export mutates accepted data. | Export writes only under `exports/`; never patches `output/`. |
| Real end-to-end run mutates broad project data. | Use temp/controlled fixture first; ask before real data smoke. |

## Validation

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run -- --help`
- `cargo run -- viewer --help`
- `cargo run -- export --help`
- `cd viewer && npm run check`
