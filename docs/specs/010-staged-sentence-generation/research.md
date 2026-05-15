# Research

## Files

### `src/sentence_generate.rs`

#### R001 - Single Prompt Generation Is Current Default

Status: confirmed  
Kind: design  
Backlog: none  
Confidence: high

What we saw:
- `sentence_generate` loads `generation_prompt_sentences_enrichment.txt`.
- It calls the configured model once per batch.
- It passes the response to `merge_enrichment`.

Why it matters:
- This is exactly the shape eval showed to be slower and less predictable than
  focused prompts.

Recommended action:
- Replace the single model call with staged calls while preserving planning,
  readiness, validation, accepted writer, and user-facing outcome.

### `src/sentence_enrichment.rs`

#### R002 - Merger Already Owns Trusted Source Boundary

Status: confirmed  
Kind: good-pattern  
Backlog: none  
Confidence: high

What we saw:
- `merge_enrichment` copies Hindi, romanisation, English, title/subtitle, and
  source_ref from `PlannedSentenceBatch`.
- Model output only supplies enrichment fields.
- It already filters non-word tokens before validation.

Why it matters:
- Staged generation should keep this trust boundary.

Recommended action:
- Reuse or evolve this module into a staged merger. Do not move trusted source
  ownership into prompt/model output.

### `src/eval_prompts/`

#### R003 - Focused Prompts Have Better Evidence Than Full Enrichment

Status: confirmed  
Kind: design  
Backlog: none  
Confidence: high

What we saw:
- Eval runs showed strong results for register v3, literal, English, and
  word-breakdown prompts.
- Full enrichment was useful as a stress test but slower and less consistent.

Why it matters:
- Generation should use the focused prompt lessons rather than the full
  enrichment prompt.

Recommended action:
- Use `sentence/register`, `sentence/literal`, and
  `sentence/word-breakdown-from-translation` as default generation stages.

### `src/run_report.rs`

#### R004 - Run Report Needs Stage-Level Detail

Status: confirmed  
Kind: improvement  
Backlog: none  
Confidence: high

What we saw:
- Current report has one prompt path/fingerprint and one validation summary.

Why it matters:
- Staged generation needs to say which stage was slow or failed.

Recommended action:
- Add `stages[]` with prompt ID/version/fingerprint, model, timing, and error.

## Data Drift Themes Caught

- Eval/generation prompt drift is the central risk. The implementation should
  either share prompt text or make prompt versions/fingerprints explicit enough
  that drift is visible in reports.

## Research Decisions

- Default generation should not use the full-enrichment prompt.
- Use translation-guided word breakdown as the default word stage because the
  source English is trusted and eval quality was strong.
- Keep one configured generation model for all stages in this spec.
