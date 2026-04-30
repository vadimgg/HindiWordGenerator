/**
 * Anki Gender field builder.
 *
 * Responsible for: building the Anki Gender badge HTML field.
 *
 * Dependencies: none
 */
// Responsible for: building the Anki Gender HTML field

/**
 * Builds the Anki Gender field HTML for a word.
 *
 * Returns a masculine or feminine badge span, or empty string if gender is absent.
 *
 * @param {object} word          - Vocabulary word object.
 * @param {string} [word.gender] - 'masculine' or 'feminine'.
 * @returns {string} HTML badge span, or empty string if no gender.
 */
export function buildAnkiGender(word) {
  if (word.gender === 'masculine') return '<span class="badge-masc">m</span>';
  if (word.gender === 'feminine')  return '<span class="badge-fem">f</span>';
  return '';
}
