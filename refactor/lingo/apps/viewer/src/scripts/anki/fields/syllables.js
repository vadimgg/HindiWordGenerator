/**
 * Anki Syllables field builder.
 *
 * Responsible for: building the Anki Syllables plain-text field.
 *
 * Dependencies: none
 */
// Responsible for: building the Anki Syllables plain-text field

/**
 * Returns the syllables string for the Anki answer header.
 *
 * @param {object} word - Vocabulary word object.
 * @returns {string} Syllables string or empty string.
 */
export function buildAnkiSyllables(word) {
  return word.syllables ?? '';
}
