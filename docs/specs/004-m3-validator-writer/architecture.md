# Architecture

## Part 1 - What Changed And Why

M3 adds the safety layer between planned source rows and accepted sentence JSON.
The architectural risk is mixing trusted source data, model enrichment, schema
validation, and file writes in one future `generate` command. This spec keeps
those responsibilities separate before M4 adds model calls.

## Part 2 - Module Ownership

| Module | Owns | Must never |
|---|---|---|
| `src/sentence_schema.rs` | Rust structs for accepted sentence batches, parse/serialize helpers, field names. | Decide whether a candidate is valid for a source row or write files. |
| `src/sentence_validate.rs` | Validation rules and validation error reporting. | Read/write project files, call Ollama, or print user-facing output. |
| `src/accepted_writer.rs` | Atomic write mechanics for already-validated accepted JSON. | Re-validate business rules, overwrite existing accepted files, or write outside the requested target. |
| `src/sentence_plan.rs` | Planner read model and shared source identity/fingerprint data. | Write accepted output or accept model-generated trusted source fields. |
| `src/main.rs` / `src/cli.rs` | Existing command dispatch; no new write command for M3. | Hide writer behavior behind an undocumented user-facing command. |
| `viewer/**` | Display accepted sentence cards, resolving `token.word_id` and legacy `word_index`. | Change accepted JSON authority or mutate output files. |

## Part 3 - Public Entry Points

M3 does not add `hindi sentences generate`. The public Rust entry points are
library-like functions used by tests now and by M4 later:

```text
candidate JSON/string
  -> sentence_schema parse
  -> sentence_validate validate against expected source rows
  -> accepted_writer write validated batch to target
```

The existing user-facing command remains:

```bash
cargo run -- sentences plan --max-batches 1
```

It should remain read-only. If implementation extracts shared source identity
helpers from the planner, the rendered planner output should stay compatible
with M2.

## Part 4 - Shared Abstractions

| Abstraction | Used By | Contract | Must Not |
|---|---|---|---|
| Source identity/fingerprint helper | Planner, validator, future generator | Given Hindi, romanisation, and English, returns the canonical fingerprint defined in `docs/DESIGN.md`. | Have two competing normalization/hash implementations. |
| Sentence batch schema | Validator, writer, future generator | Parses/serializes the accepted sentence JSON shape. | Accept legacy `word_index` in new Rust candidates. |
| Validation report | Tests, future generator | Returns all useful batch errors with sentence/source context. | Print directly or write files. |
| Accepted writer | Tests, future generator | Writes one validated batch atomically to one target path. | Overwrite existing output or silently skip collisions. |

## Part 5 - Data And Drift Risks

### Persistent Files

| File | Written By | Read By | Rule |
|---|---|---|---|
| `output/sentences/*.json` | Future M4 through `accepted_writer`; M3 tests only write temp fixtures | Planner, viewer, future export | Accepted learner authority. Do not overwrite existing batch files. |
| `viewer/**` source files | Humans / this spec | Browser build and viewer users | Must support new `word_id` and legacy `word_index` until Python output is retired. |
| `docs/ROADMAP.md` | Humans / this spec | Humans and agents | Status and pending wording should match implementation reality. |
| `docs/specs/004-m3-validator-writer/**` | brief and humans | brief, humans, agents | Spec/task state for this work only. |

### Drift Scenario A - Planner And Validator Fingerprints Diverge

**How it happens.** Planner keeps the M2 helper while validator adds a second
normalization/hash implementation.

**What breaks.** A card planned as current can fail validation or vice versa.

**Detection.** Shared unit tests with whitespace and known SHA-256 vectors.

**Resolution.** Extract one helper and make both callers use it.

> **Review flag.** Reject duplicate fingerprint functions unless one is test
> fixture scaffolding.

### Drift Scenario B - Viewer Cannot Render Rust Tokens

**How it happens.** Validator emits `word_id`, but viewer still only looks for
legacy `word_index`.

**What breaks.** The first Rust-generated card passes validation but fails in
the preview/export flow.

**Detection.** Viewer fixture/build test covering `word_id` and `word_index`.

**Resolution.** Viewer resolves `word_id` first, then falls back to
`word_index`.

> **Review flag.** Reject code that converts Rust `word_id` back to
> `word_index` just to satisfy the viewer.

### Drift Scenario C - Writer Leaves Partial Output

**How it happens.** The writer writes directly to the accepted target before
serialization or validation is complete.

**What breaks.** `output/` contains corrupt or partial learner data.

**Detection.** Writer failure tests and collision tests in temp directories.

**Resolution.** Serialize first, write temp file, then rename.

> **Review flag.** Reject direct writes to `output/sentences/*.json` in M3.

## Part 6 - Code Review Checklist

| Area | Reject | Accept |
|---|---|---|
| Schema | Untyped string manipulation for candidate JSON. | Typed structs with focused parse/serialize tests. |
| Validator | File writes, printing, or CLI parsing. | Pure validation from candidate + expected source rows to report/result. |
| Legacy support | Rust validator accepts `word_index`. | Viewer alone supports `word_index` fallback. |
| Writer | Overwrites accepted output or writes before validation. | Collision refusal, temp write, rename. |
| Planner reuse | Duplicate fingerprint logic. | One shared fingerprint helper or a clear extraction path with tests. |
| Protected data | Tests modify real `input/`, `output/`, `audio/`, or `runs/`. | Tests use temp directories/fixtures only. |

## Appendix - Files Removed Or Moved

None planned.

## Appendix - Out-Of-Scope Residue

- M2 uses narrow hand-rolled JSON scanning for existing accepted output. M3 may
  introduce typed JSON for candidate batches, but a full replacement of M2
  scanning is only required if it reduces duplication safely.
