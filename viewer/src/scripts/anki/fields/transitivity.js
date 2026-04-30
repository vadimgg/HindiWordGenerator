/**
 * Anki Transitivity field builder.
 *
 * Responsible for: building the Anki Transitivity badge HTML field.
 *
 * Dependencies: none
 */
// Responsible for: building the Anki Transitivity HTML field

/**
 * Builds the Anki Transitivity field HTML for a word.
 *
 * Returns a neutral badge for transitive/intransitive, or empty string otherwise.
 *
 * @param {object} word               - Vocabulary word object.
 * @param {string} [word.transitivity] - 'transitive' or 'intransitive'.
 * @returns {string} HTML badge span, or empty string if no transitivity.
 */
export function buildAnkiTransitivity(word) {
  if (word.transitivity === 'transitive')   return '<span class="badge-neutral">trans.</span>';
  if (word.transitivity === 'intransitive') return '<span class="badge-neutral">intrans.</span>';
  return '';
}
