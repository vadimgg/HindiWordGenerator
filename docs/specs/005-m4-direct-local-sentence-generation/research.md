# Research

## Files

### `docs/ROADMAP.md`

#### R001 - M4 Is One-Model Direct Generation

Status: confirmed  
Kind: scope  
Backlog: none  
Confidence: high

What we saw:
- M4 requires `hindi sentences generate --max-batches 1`.
- M4 uses `generation_prompt_sentences_enrichment.txt`.
- Source QA, model switching, and review/accept are in Later.

Why it matters:
- The spec should not add multi-model orchestration or Ollama lifecycle
  management.

Recommended action:
- Require only `[models].sentence_generation` and local HTTP calls.

### `generation_prompt_sentences_enrichment.txt`

#### R002 - Prompt Already Matches Trusted-Source Boundary

Status: confirmed  
Kind: reuse  
Backlog: none  
Confidence: high

What we saw:
- Prompt tells model not to output trusted source fields.
- Output is enrichment keyed by source row ID.

Why it matters:
- M4 can implement merge safely: source fields come from Rust, enrichment comes
  from model.

Recommended action:
- Prompt builder should send only source-row fields, and merge should ignore
  any trusted fields if model returns them anyway.

### `src/sentence_plan.rs`

#### R003 - Planner Needs A Generation View

Status: confirmed  
Kind: reuse  
Backlog: none  
Confidence: high

What we saw:
- M2 planner has pending counts and target filenames, but much of the useful
  source-row data is private.

Why it matters:
- M4 should not duplicate pending/target selection.

Recommended action:
- Expose a typed generation plan or extract shared planner internals so generate
  can reuse source rows and target paths.

### `src/sentence_validate.rs` and `src/accepted_writer.rs`

#### R004 - M3 Provides The Safety Gate

Status: confirmed  
Kind: reuse  
Backlog: none  
Confidence: high

What we saw:
- M3 validator and writer are currently library-like internals.

Why it matters:
- M4 should call them directly instead of reimplementing validation/write
  safety.

Recommended action:
- Generation should construct `SentenceBatch`, call `validate_sentence_batch`,
  and only then call `write_sentence_batch`.

## Data Drift Themes Caught

- Generation must reuse planner target selection.
- Generation must not trust source fields returned by the model.
- Run reports are diagnostics only.

## Research Decisions

- User does not need to start Ollama for spec creation.
- Implementation/smoke can use real Ollama if the user starts
  `ollama run translategemma:12b`.
