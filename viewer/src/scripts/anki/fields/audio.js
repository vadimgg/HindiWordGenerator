/**
 * Anki Audio field builder — [sound:] syntax for word pronunciation.
 *
 * Responsible for: building the Anki Audio field value using AnkiConnect's
 * [sound:] tag syntax. Anki media files are stored flat (no subdirectories),
 * so the filename is namespaced with the audioBatch and word identifiers
 * to ensure uniqueness across all exported words.
 *
 * Dependencies: none.
 */
// Responsible for: building the Anki Audio field with [sound:] syntax for word pronunciation

/**
 * Builds the Anki Audio field value for a vocabulary word.
 *
 * Returns an empty string if the word has no audioBatch.
 * The filename uses a namespaced pattern to avoid collisions in Anki's flat media store.
 *
 * @param {object} word - Vocabulary word object.
 * @param {string} [word.audioBatch] - Batch ID (vocab filename without .json).
 * @param {string} [word.hindi] - Devanagari form.
 * @param {string} [word.romanisation] - Romanised transliteration.
 * @returns {string} Anki sound tag or empty string.
 */
export function buildAnkiAudio(word) {
  if (!word.audioBatch || !word.audio) return '';
  const filename = `${word.audioBatch}__${word.hindi}_${word.romanisation}__word.mp3`;
  return `[sound:${filename}]`;
}
