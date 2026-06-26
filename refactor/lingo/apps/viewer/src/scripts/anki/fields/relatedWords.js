/**
 * Anki Related Words field builder.
 *
 * Responsible for: building the Anki RelatedWords HTML section field.
 *
 * Dependencies: anki/fields/utils.js.
 */
// Responsible for: building the Anki RelatedWords HTML field
import { esc, aSection } from './utils.js';

/**
 * Renders a single related word as a centred vertical .clang-item card.
 *
 * @param {object} rw - Related word object with { hindi, roman, english }.
 * @returns {string} HTML div string for the chip.
 */
function renderRelatedWord(rw) {
  return (
    `<div class="clang-item">` +
    `<div class="word-row">` +
    `<span class="word-text" lang="hi">${esc(rw.hindi)}</span>` +
    `<span class="word-rom">${esc(rw.roman)}</span>` +
    `</div>` +
    `<div class="mean-box"><span>${esc(rw.english)}</span></div>` +
    `</div>`
  );
}

/**
 * Builds the Anki RelatedWords field HTML for a word.
 *
 * Returns a styled section with vocabulary chips, or empty string if absent.
 *
 * @param {object}   word                - Vocabulary word object.
 * @param {object[]} [word.related_words] - Array of { hindi, roman, english } objects.
 * @returns {string} HTML section, or empty string if no related words.
 */
export function buildAnkiRelatedWords(word) {
  const relatedWords = word.related_words || [];
  if (!relatedWords.length) return '';
  const items = relatedWords.map(renderRelatedWord).join('');
  return aSection('Related Words', items, '#0ea5e9');
}
