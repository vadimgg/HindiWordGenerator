/**
 * Shared Anki tag-building utility for vocabulary words.
 *
 * Responsible for: deriving the standard Anki tag array from a word object so
 * that the live AnkiConnect export path (export.js) and the .txt fallback
 * download (txtFallback.js) always produce identical tags.
 *
 * Used by: anki/export.js, anki/txtFallback.js.
 * No dependencies on other project modules.
 */
// Responsible for: shared Anki tag-building utility used by export.js and txtFallback.js

/**
 * Builds the Anki tag list for a single vocabulary word.
 *
 * When the word has an explicit anki_tags array, each tag is namespaced under
 * 'Hindi::' and returned directly. Otherwise falls back to derived slugs:
 *   - 'hindi'                       — fixed namespace tag
 *   - category slug                 — first word of pos, letters only
 *   - 'added:YYYYMMDD'              — today's date in compact ISO format
 *
 * @param {object}   word              - Vocabulary word object.
 * @param {string[]} [word.anki_tags]  - Explicit tag list from the vocab JSON.
 * @param {string}   [word.pos]        - Part of speech, e.g. "adjective", "noun".
 * @returns {string[]} Non-empty tag strings ready for the Anki API or .txt header.
 */
export function buildWordTags(word) {
  if (word.anki_tags?.length) {
    return word.anki_tags.map(t => `Hindi::${t}`);
  }
  const today   = new Date().toISOString().split('T')[0].replace(/-/g, '');
  const catSlug = ((word.pos || 'unknown').split(' ')[0]).toLowerCase().replace(/[^a-z]/g, '');
  return ['hindi', catSlug, `added:${today}`].filter(Boolean);
}
