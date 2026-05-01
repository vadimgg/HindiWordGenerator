/**
 * Shared Anki media naming helpers.
 *
 * Responsible for: deriving the exact filenames used by Anki [sound:] fields
 * and by AnkiConnect media uploads. Anki stores media in a flat folder, so
 * names are batch-prefixed to avoid collisions.
 *
 * No dependencies on other project modules.
 */
// Responsible for: shared Anki media filenames and sound tags

function sanitiseSentenceEnglish(english = '') {
  return english.replace(/\s+/g, '_').replace(/[?!.,]/g, '');
}

/**
 * Builds the flat Anki media filename for a word audio file.
 *
 * @param {object} word - Word object with audioBatch, hindi, romanisation.
 * @returns {string} Media filename, or an empty string when required data is missing.
 */
export function wordMediaFilename(word) {
  if (!word.audioBatch || !word.hindi || !word.romanisation) return '';
  return `${word.audioBatch}__${word.hindi}_${word.romanisation}__word.mp3`;
}

/**
 * Builds the flat Anki media filename for a sentence audio file.
 *
 * @param {object} sentence - Sentence object with audioBatch and english.
 * @returns {string} Media filename, or an empty string when required data is missing.
 */
export function sentenceMediaFilename(sentence) {
  if (!sentence.audioBatch || !sentence.english) return '';
  return `${sentence.audioBatch}__${sanitiseSentenceEnglish(sentence.english)}__sentence.mp3`;
}

/**
 * Wraps a media filename in Anki's sound tag syntax.
 *
 * @param {string} filename - Flat Anki media filename.
 * @returns {string} Anki sound tag, or empty string when filename is empty.
 */
export function soundTag(filename) {
  return filename ? `[sound:${filename}]` : '';
}
